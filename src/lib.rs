// zeta-quantum/src/lib.rs (updated integration)

pub mod api;
pub mod bms;
pub mod cache;
pub mod cost;
pub mod error;
pub mod flux_holonomy;
pub mod ir;
pub mod nonlocal_dynamics;
pub mod phi_ir;
pub mod qpu;
pub mod radiative_entropy;

pub use bms::{BMSObservable, EscapeRoute};
pub use error::{ZetaError, ZetaResult};
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
        // For now the optimizer is identity; return integrated obstruction S_X.
        let integrated_obstruction = crate::cost::integrated_obstruction(circ, &self.qpu);
        Ok((circ.clone(), integrated_obstruction))
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

    fn analyze_nonlocal_dynamics(&self, dt: f64, dissipator: f64) -> EntropicSubsystem {
        let rho_full = DMatrix::identity(4, 4);
        let h_nonlocal = DMatrix::from_fn(4, 4, |i, j| if i % 2 != j % 2 { 0.1 } else { 0.0 });
        let rho_evolved = evolve_nonlocal_lindblad(&rho_full, &h_nonlocal, dissipator, dt);
        reduce_to_subsystem(&rho_evolved, 1)
    }

    fn analyze_flux_sector(&self) -> FluxSector {
        let proj1 = DMatrix::from_diagonal_element(4, 4, 1.0);
        let proj2 = proj1.clone();
        let flux_op = DMatrix::from_element(4, 4, 0.05);
        let holonomy_mat = DMatrix::identity(4, 4) + flux_op.clone() * 0.2;
        compute_flux_sector(&proj1, &proj2, &flux_op, &holonomy_mat)
    }

    fn analyze_radiative_visibility(
        &self,
        bms: &BMSObservable,
        integrated_obstruction: f64,
    ) -> RadiativeVisibility {
        RadiativeVisibility::from_bms_and_entropy(bms, integrated_obstruction)
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
        let (optimized, integrated_obstruction) = self.quantize(circ)?;
        let (_full_entropy, bms_obs) =
            crate::bms::probe_gravitational_memory(&optimized, &self.qpu, bms_route);

        let subsystem = self.analyze_nonlocal_dynamics(dt, dissipator);
        let flux_sector = self.analyze_flux_sector();
        let radiative = self.analyze_radiative_visibility(&bms_obs, integrated_obstruction);

        Ok((
            optimized,
            integrated_obstruction,
            bms_obs,
            subsystem,
            flux_sector,
            radiative,
        ))
    }
}
