use crate::{phi_ir::PhiCircuit, qpu::QPU};

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
