#[derive(Debug, Clone, PartialEq)]
pub enum WeylGen {
    Affine(u32), // s_0 – the affine reflection (translation sector shift)
    Simple(u32), // s_i – finite Weyl reflections
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhiElement {
    pub word: Vec<WeylGen>, // reduced or unreduced word in affine Weyl group
    pub targets: Vec<u32>,  // logical qubits this element acts on
}

#[derive(Debug, Clone)]
pub struct PhiCircuit {
    pub elements: Vec<PhiElement>,
}

impl PhiElement {
    // Naive constructors – map common gates to minimal Weyl words (extensible)
    pub fn h(qubit: u32) -> Self {
        PhiElement {
            word: vec![WeylGen::Simple(qubit)],
            targets: vec![qubit],
        }
    }
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
    pub fn x(qubit: u32) -> Self {
        PhiElement {
            word: vec![WeylGen::Simple(qubit); 2],
            targets: vec![qubit],
        }
    }
}

impl PhiCircuit {
    pub fn new() -> Self {
        PhiCircuit { elements: vec![] }
    }
    pub fn push(&mut self, e: PhiElement) {
        self.elements.push(e);
    }
}
