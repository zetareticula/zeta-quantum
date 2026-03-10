#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum WeylGen {
    Affine(u32), // sector shift / translation
    Simple(u32), // finite reflection
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PhiElement {
    pub word: Vec<WeylGen>,
    pub targets: Vec<u32>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct PhiCircuit {
    pub elements: Vec<PhiElement>,
}

impl PhiElement {
    pub fn h(q: u32) -> Self {
        Self {
            word: vec![WeylGen::Simple(q)],
            targets: vec![q],
        }
    }
    pub fn x(q: u32) -> Self {
        Self {
            word: vec![WeylGen::Simple(q), WeylGen::Simple(q)],
            targets: vec![q],
        }
    }
    pub fn cnot(c: u32, t: u32) -> Self {
        Self {
            word: vec![WeylGen::Simple(c), WeylGen::Affine(t), WeylGen::Simple(c)],
            targets: vec![c, t],
        }
    }
}
