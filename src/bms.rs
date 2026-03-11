use crate::phi_ir::PhiCircuit;
use crate::qpu::QPU;

/// Exact replica of your entropy definition
pub type RedundancyDim = usize; // r([γ]) = dim R_X([γ]) ⊂ H^0(M_H, O)

#[derive(Debug, Clone, PartialEq)]
pub struct CohomologicalEntropy {
    pub r_gamma: RedundancyDim, // intrinsic count
    pub sx: f64,                // Integrated Obstruction (our compiler value)
}

/// What gravitational memory actually sees (non-injective projection)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct BMSObservable {
    pub delta_c_ab: f64, // integrated stress-energy / total energy budget
    pub phase_transition_detected: bool, // fillable ↔ non-fillable jump
    pub decoded_sx: Option<f64>, // None = impossible generically
    pub escape_route_used: EscapeRoute,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum EscapeRoute {
    None,                      // generic case – information collapses
    DistinctMultipoles,        // (1) distinguishable multipole patterns
    ScalarMetricEngineering,   // (2) extra scalar Φ channel
    HolographicReconstruction, // (3) boundary entanglement dual
}

impl BMSObservable {
    /// Formal projection map: H_micro → O_BMS (your exact statement)
    pub fn from_entropy(entropy: &CohomologicalEntropy, route: EscapeRoute) -> Self {
        let delta_c_ab = entropy.sx; // universal coupling – only total budget

        let phase_jump = entropy.r_gamma > 0; // toy fillable ↔ non-fillable trigger on any nontrivial redundancy

        let decoded = match route {
            EscapeRoute::DistinctMultipoles => {
                // unlikely in standard GR – finite resolution
                if entropy.r_gamma % 2 == 0 {
                    Some(entropy.sx * 0.7)
                } else {
                    None
                }
            }
            EscapeRoute::ScalarMetricEngineering => Some(entropy.sx), // extra Φ channel
            EscapeRoute::HolographicReconstruction => Some(entropy.sx), // needs dictionary
            EscapeRoute::None => None,
        };

        BMSObservable {
            delta_c_ab,
            phase_transition_detected: phase_jump,
            decoded_sx: decoded,
            escape_route_used: route,
        }
    }
}

/// Main integration point – called after every quantize()
pub fn probe_gravitational_memory(
    circ: &PhiCircuit,
    qpu: &QPU,
    route: EscapeRoute,
) -> (CohomologicalEntropy, BMSObservable) {
    // Compute full intrinsic S_X + redundancy dim (toy model: count Affine gens)
    let mut r_total = 0;

    for elem in &circ.elements {
        let r = elem
            .word
            .iter()
            .filter(|g| matches!(g, crate::phi_ir::WeylGen::Affine(_)))
            .count();
        r_total += r;
    }

    let sx_total = crate::cost::integrated_obstruction(circ, qpu);

    let entropy = CohomologicalEntropy {
        r_gamma: r_total,
        sx: sx_total,
    };
    let observable = BMSObservable::from_entropy(&entropy, route);

    (entropy, observable)
}
