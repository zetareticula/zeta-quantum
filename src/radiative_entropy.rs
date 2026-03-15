//! Copyright (c) 2026 Zeta Reticula Inc
//! Licensed under the MIT License. See LICENSE for details.

use crate::bms::BMSObservable;

/// Exact replica of your definitions
#[derive(Debug, Clone, PartialEq)]
pub struct RadiativeCharge {
    pub q_f: f64,                    // Q_f = ∫ f m_B dΩ
    pub flux: f64,                   // Flux_f = ∫ N_AB du
    pub is_from_stress_energy: bool, // true in standard GR
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntropyCurrent {
    pub j_s: f64,         // local J^μ_S
    pub divergence: f64,  // ∇_μ J^μ_S ≥ 0
    pub is_noether: bool, // false in standard thermodynamics
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadiativeVisibility {
    pub in_standard_gr: bool, // false – entropy invisible
    pub required_option: Option<NewPhysicsOption>,
    pub memory_would_see_sx: Option<f64>, // Some(value) only if extended
    pub verdict: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewPhysicsOption {
    EntropyAsGaugeCharge, // Option 1 – σ field coupled to R
    EntanglementGeometry, // Option 2 – holographic Ryu–Takayanagi
    ExtendedBMSAlgebra,   // Option 3 – BMS × U(1)_S
}

impl RadiativeVisibility {
    /// Formal check of your 3 conditions (III)
    pub fn from_bms_and_entropy(bms: &BMSObservable, entropy_sx: f64) -> Self {
        let in_gr = false; // your IV: GR too coarse

        let (option, visible) =
            if bms.escape_route_used == crate::bms::EscapeRoute::ScalarMetricEngineering {
                (
                    Some(NewPhysicsOption::EntropyAsGaugeCharge),
                    Some(entropy_sx),
                )
            } else if bms.escape_route_used == crate::bms::EscapeRoute::HolographicReconstruction {
                (
                    Some(NewPhysicsOption::EntanglementGeometry),
                    Some(entropy_sx),
                )
            } else {
                (Some(NewPhysicsOption::ExtendedBMSAlgebra), None) // still requires algebra extension
            };

        let verdict = if in_gr {
            "Gravitational memory encodes energy flux only. Entropy remains invisible (standard GR)."
        } else {
            "Entropy now geometric → visible in extended memory (new physics required)."
        };

        RadiativeVisibility {
            in_standard_gr: in_gr,
            required_option: option,
            memory_would_see_sx: visible,
            verdict: verdict.into(),
        }
    }
}

/// Toy News tensor → flux (your I)
pub fn compute_news_flux(news_n_ab: f64, du: f64) -> f64 {
    news_n_ab * du
}
