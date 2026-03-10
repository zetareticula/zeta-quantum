// zeta-quantum/src/lib.rs (updated integration)

pub mod api;
pub mod bms;
pub mod flux_holonomy;
pub mod ir;
pub mod nonlocal_dynamics;
pub mod phi_ir;
pub mod qpu;
pub mod radiative_entropy;

pub use bms::{BMSObservable, EscapeRoute};
pub use flux_holonomy::{compute_flux_sector, FluxSector};
pub use nonlocal_dynamics::{
    evolve_nonlocal_lindblad, reduce_to_subsystem, von_neumann_entropy, EntropicSubsystem,
};
pub use phi_ir::{PhiCircuit, PhiElement, WeylGen};
pub use qpu::{Modality, QPU};
pub use radiative_entropy::RadiativeVisibility;

use nalgebra::DMatrix;
use std::collections::HashMap;

#[derive(Debug)]
pub struct QuantumQuantizer {
    pub qpu: QPU,
    pub calibration: HashMap<String, f64>,
}

impl QuantumQuantizer {
    pub fn new(
        modality: Modality,
        calibration: &HashMap<String, f64>,
        calibration_ts: String,
    ) -> Self {
        let mut qpu = QPU::new(modality, calibration_ts);

        // Minimal coupling map bootstrap: parse keys like "0-1" => err
        for (k, &err) in calibration {
            if let Some((a, b)) = k.split_once('-') {
                if let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>()) {
                    qpu.add_coupling(a, b, err);
                    qpu.add_coupling(b, a, err);
                }
            }
        }

        Self {
            qpu,
            calibration: calibration.clone(),
        }
    }

    pub fn quantize(&mut self, circ: &PhiCircuit) -> anyhow::Result<(PhiCircuit, f64)> {
        // For now the optimizer is identity; S_X integrates hardware-obstruction cost.
        let mut sx_total = 0.0;
        for elem in &circ.elements {
            if elem.targets.len() == 2 {
                sx_total += self.qpu.gate_cost(elem.targets[0], elem.targets[1]);
            }
        }
        Ok((circ.clone(), sx_total))
    }

    pub fn quantize_with_bms(
        &mut self,
        circ: &PhiCircuit,
        route: EscapeRoute,
    ) -> anyhow::Result<(PhiCircuit, f64, BMSObservable)> {
        let (optimized, sx) = self.quantize(circ)?;
        let (_entropy, bms_obs) =
            crate::bms::probe_gravitational_memory(&optimized, &self.qpu, route);
        Ok((optimized, sx, bms_obs))
    }

    /// v0.6.0: quantize + full subsystems/sectors
    pub fn quantize_full_analysis(
        &mut self,
        circ: &PhiCircuit,
        bms_route: crate::bms::EscapeRoute,
        dt: f64,
        dissipator: f64,
    ) -> anyhow::Result<(
        PhiCircuit,
        f64,
        crate::bms::BMSObservable,
        EntropicSubsystem,
        FluxSector,
        RadiativeVisibility,
    )> {
        let (optimized, sx) = self.quantize(circ)?;
        let (_full_entropy, bms_obs) =
            crate::bms::probe_gravitational_memory(&optimized, &self.qpu, bms_route);

        // Theorem 1: full subsystem impl (toy 2-qubit, subsystem A = first qubit)
        let rho_full = DMatrix::identity(4, 4); // initial |00><00| + nonlocal
        let h_nonlocal = DMatrix::from_fn(4, 4, |i, j| if i % 2 != j % 2 { 0.1 } else { 0.0 });
        let rho_evolved = evolve_nonlocal_lindblad(&rho_full, &h_nonlocal, dissipator, dt);
        let subsystem = reduce_to_subsystem(&rho_evolved, 1); // |A| = 1 qubit

        // Theorem 2: full flux sector (toy regions R1/R2 projectors)
        let proj1 = DMatrix::from_diagonal_element(4, 4, 1.0); // identity for simplicity
        let proj2 = proj1.clone();
        let flux_op = DMatrix::from_element(4, 4, 0.05);
        let holonomy_mat = DMatrix::identity(4, 4) + flux_op.clone() * 0.2; // toy exp(∮ A)
        let flux_sector = compute_flux_sector(&proj1, &proj2, &flux_op, &holonomy_mat);

        let radiative = RadiativeVisibility::from_bms_and_entropy(&bms_obs, sx);

        Ok((optimized, sx, bms_obs, subsystem, flux_sector, radiative))
    }
}
