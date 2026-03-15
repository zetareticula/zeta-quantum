//! Copyright (c) 2026 Zeta Reticula Inc
//! Licensed under the MIT License. See LICENSE for details.

use crate::{phi_ir::PhiCircuit, qpu::QPU};

#[derive(Debug, Clone)]
pub struct CostResult {
    pub integrated_obstruction: f64,
    pub sx_per_gate: Vec<f64>,
}

/// Integrated obstruction S_X over a circuit and hardware model.
///
/// Today this is a hardware-routing obstruction proxy: sum of 2-qubit gate costs.
pub fn integrated_obstruction(circ: &PhiCircuit, qpu: &QPU) -> f64 {
    let mut total = 0.0;
    for elem in &circ.elements {
        if elem.targets.len() == 2 {
            total += qpu.gate_cost(elem.targets[0], elem.targets[1]);
        }
    }
    total
}

/// Calculate cost for a circuit on a QPU
pub fn calculate_cost(circ: &PhiCircuit, qpu: &QPU) -> CostResult {
    let integrated_obstruction = integrated_obstruction(circ, qpu);
    let sx_per_gate = circ
        .elements
        .iter()
        .filter(|e| e.targets.len() == 2)
        .map(|e| qpu.gate_cost(e.targets[0], e.targets[1]))
        .collect();
    CostResult {
        integrated_obstruction,
        sx_per_gate,
    }
}
