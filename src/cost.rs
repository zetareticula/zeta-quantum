//! Copyright (c) 2026 Zeta Reticula Inc
//! Licensed under the MIT License. See LICENSE for details.

use crate::{phi_ir::PhiCircuit, qpu::QPU};

/// Cost result for a circuit on a QPU
#[derive(Debug, Clone)]
pub struct CostResult {
    pub integrated_obstruction: f64, // Integrated obstruction S_X over the circuit
    pub sx_per_gate: Vec<f64>,       // SX cost per 2-qubit gate
}

/// Integrated obstruction S_X over a circuit and hardware model.
///
/// Today this is a hardware-routing obstruction proxy: sum of 2-qubit gate costs.
///
/// # Arguments
///
/// * `circ` - The circuit to evaluate
/// * `qpu` - The QPU to use for the evaluation
///
/// # Returns
///
/// * `f64` - The integrated obstruction
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qpu::{Modality, QPU};

    fn test_qpu() -> QPU {
        QPU::new(Modality::Superconducting, "test".into())
    }

    #[test]
    fn test_integrated_obstruction() {
        let circ = PhiCircuit::hadamard(0);
        let qpu = test_qpu();
        let result = integrated_obstruction(&circ, &qpu);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_calculate_cost() {
        let circ = PhiCircuit::hadamard(0);
        let qpu = test_qpu();
        let result = calculate_cost(&circ, &qpu);
        assert_eq!(result.integrated_obstruction, 0.0);
        assert_eq!(result.sx_per_gate.len(), 0);
    }
}
