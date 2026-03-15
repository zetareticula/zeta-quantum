//! Copyright (c) 2026 Zeta Reticula Inc
//! Licensed under the MIT License. See LICENSE for details.

/// Weyl generators for the affine Weyl group
#[derive(Debug, Clone, PartialEq)]
pub enum WeylGen {
    Affine(u32), // s_0 – the affine reflection (translation sector shift)
    Simple(u32), // s_i – finite Weyl reflections
}

#[derive(Debug, Clone, PartialEq)]
pub struct AffineWeylGroup;

/// The affine Weyl group representation
///
/// This is a mathematical structure that represents the symmetries of the quantum system
/// in the phi language. It is used to encode quantum gates as elements of the group.
///
/// The affine Weyl group is a group of transformations that preserve the structure of
/// the quantum system. It is used to encode quantum gates as elements of the group.
///
/// The affine Weyl group is represented as a sequence of generators, where each generator
/// corresponds to a specific type of transformation. The generators are:
/// - Affine: s_0 – the affine reflection (translation sector shift)
/// - Simple: s_i – finite Weyl reflections
///
/// The affine Weyl group is used to encode quantum gates as elements of the group.
/// The encoding is done by mapping the quantum gate to a sequence of generators.
/// The sequence of generators is then used to represent the quantum gate in the phi language.
///
/// The affine Weyl group is a mathematical structure that represents the symmetries of the quantum system
/// in the phi language. It is used to encode quantum gates as elements of the group.

/// A single element in the phi circuit, represented as a word in the affine Weyl group
#[derive(Debug, Clone, PartialEq)]
pub struct PhiElement {
    pub word: Vec<WeylGen>, // reduced or unreduced word in affine Weyl group
    pub targets: Vec<u32>,  // logical qubits this element acts on
}

/// A circuit in the phi language, represented as a sequence of phi elements
#[derive(Debug, Clone)]
pub struct PhiCircuit {
    /// The sequence of phi elements that make up the circuit
    pub elements: Vec<PhiElement>,
}

impl PhiElement {
    /// Create a Hadamard gate on a single qubit
    ///
    /// # Arguments
    /// * `qubit` - The qubit to apply the Hadamard gate to
    ///
    /// # Returns
    /// * `PhiElement` - The Hadamard gate as a phi element
    // Naive constructors – map common gates to minimal Weyl words (extensible)
    pub fn h(qubit: u32) -> Self {
        // Hadamard in affine Weyl representation: H = s_i (simple reflection)
        // This corresponds to the finite Weyl reflection on the qubit
        PhiElement {
            word: vec![WeylGen::Simple(qubit)],
            targets: vec![qubit],
        }
    }

    /// Create a CNOT gate between two qubits
    ///
    /// # Arguments
    /// * `control` - The control qubit
    /// * `target` - The target qubit
    ///
    /// # Returns
    /// * `PhiElement` - The CNOT gate as a phi element
    pub fn cnot(control: u32, target: u32) -> Self {
        // Example word for CNOT in A1 affine Weyl (real math later)
        PhiElement {
            word: vec![
                WeylGen::Simple(control),
                WeylGen::Affine(target),
                WeylGen::Simple(control),
            ],
            targets: vec![control, target],
        }
    }
    /// Create a Pauli-X gate on a single qubit
    ///
    /// # Arguments
    /// * `qubit` - The qubit to apply the Pauli-X gate to
    ///
    /// # Returns
    /// * `PhiElement` - The Pauli-X gate as a phi element
    pub fn x(qubit: u32) -> Self {
        PhiElement {
            word: vec![WeylGen::Simple(qubit); 2],
            targets: vec![qubit],
        }
    }
}

/// A circuit in the phi language, represented as a sequence of phi elements
impl PhiCircuit {
    /// Create a new empty phi circuit
    ///
    /// # Returns
    /// * `PhiCircuit` - An empty phi circuit
    pub fn new() -> Self {
        // Initialize with empty elements vector
        PhiCircuit { elements: vec![] }
    }
    /// Add a phi element to the circuit
    ///
    /// # Arguments
    /// * `e` - The phi element to add
    pub fn push(&mut self, e: PhiElement) {
        self.elements.push(e);
    }
}

/// Example usage and testing
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hadamard() {
        let h = PhiElement::h(0);
        assert_eq!(h.targets, vec![0]);
        assert_eq!(h.word, vec![WeylGen::Simple(0)]);
    }

    #[test]
    fn test_cnot() {
        let cnot = PhiElement::cnot(0, 1);
        assert_eq!(cnot.targets, vec![0, 1]);
        assert_eq!(
            cnot.word,
            vec![WeylGen::Simple(0), WeylGen::Affine(1), WeylGen::Simple(0),]
        );
    }

    #[test]
    fn test_x() {
        let x = PhiElement::x(0);
        assert_eq!(x.targets, vec![0]);
        assert_eq!(x.word, vec![WeylGen::Simple(0), WeylGen::Simple(0)]);
    }

    #[test]
    fn test_circuit() {
        let mut circuit = PhiCircuit::new();
        circuit.push(PhiElement::h(0));
        circuit.push(PhiElement::cnot(0, 1));
        circuit.push(PhiElement::x(0));

        assert_eq!(circuit.elements.len(), 3);
        assert_eq!(circuit.elements[0].targets, vec![0]);
        assert_eq!(circuit.elements[1].targets, vec![0, 1]);
        assert_eq!(circuit.elements[2].targets, vec![0]);
    }
}
