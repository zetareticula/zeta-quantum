//! Copyright (c) 2026 Zeta Reticula Inc
//! Licensed under the MIT License. See LICENSE for details.

/// Weyl generators for the phi_ir circuit
///
/// These represent the basic operations that can be applied to qubits
///
/// # Examples
///
/// ```
/// use zeta_quantum::phi_ir::WeylGen;
///
/// let h = WeylGen::Simple(0);
/// let x = WeylGen::Simple(1);
/// let cnot = WeylGen::Affine(2);
/// ```

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum WeylGen {
    Affine(u32), // sector shift / translation
    Simple(u32), // finite reflection
}

/// A single element in the phi_ir circuit
///
/// # Examples
///
/// ```
/// use zeta_quantum::phi_ir::{PhiElement, WeylGen};
///
/// let h = PhiElement::h(0);
/// let x = PhiElement::x(1);
/// let cnot = PhiElement::cnot(0, 1);
/// ```
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PhiElement {
    pub word: Vec<WeylGen>, // The sequence of Weyl generators
    pub targets: Vec<u32>,  // The qubits the element acts on
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PhiCircuit {
    pub elements: Vec<PhiElement>, // The sequence of phi_ir elements
}

/// Methods for creating common phi_ir elements
impl PhiElement {
    /// Create a Hadamard gate on a single qubit
    pub fn h(q: u32) -> Self {
        Self {
            word: vec![WeylGen::Simple(q)], // Apply H gate to qubit q
            targets: vec![q],               // Acting on qubit q
        }
    }
    pub fn x(q: u32) -> Self {
        Self {
            word: vec![WeylGen::Simple(q), WeylGen::Simple(q)], // Apply X gate to qubit q
            targets: vec![q],                                   // Acting on qubit q
        }
    }
    pub fn cnot(c: u32, t: u32) -> Self {
        Self {
            word: vec![WeylGen::Simple(c), WeylGen::Affine(t), WeylGen::Simple(c)], // Apply CNOT gate to qubits c and t
            targets: vec![c, t], // Acting on qubits c and t
        }
    }
}

impl PhiCircuit {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
}

/// Methods for adding elements to the circuit
impl PhiCircuit {
    /// Add a single element to the circuit
    pub fn add_element(&mut self, element: PhiElement) {
        self.elements.push(element);
    }
}

/// Methods for creating circuits from common patterns
impl PhiCircuit {
    /// Create a circuit with a single Hadamard gate
    pub fn hadamard(q: u32) -> Self {
        Self {
            elements: vec![PhiElement::h(q)],
        }
    }

    /// Create a circuit with a single X gate
    pub fn x(q: u32) -> Self {
        Self {
            elements: vec![PhiElement::x(q)],
        }
    }

    /// Create a circuit with a single CNOT gate
    pub fn cnot(c: u32, t: u32) -> Self {
        Self {
            elements: vec![PhiElement::cnot(c, t)],
        }
    }
}
