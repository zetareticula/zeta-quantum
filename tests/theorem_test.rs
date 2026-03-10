use nalgebra::DMatrix;
use zeta_quantum::{compute_flux_sector, evolve_nonlocal_lindblad, von_neumann_entropy};

#[test]
fn theorem_1_nonlocal_supports_autonomous_dynamics() {
    let rho0 = DMatrix::identity(2, 2);
    let h = DMatrix::from_fn(2, 2, |i, j| if i != j { 0.1 } else { 0.0 });
    let rho1 = evolve_nonlocal_lindblad(&rho0, &h, 0.05, 0.1);
    let s0 = von_neumann_entropy(&rho0); // 0.693
    let s1 = von_neumann_entropy(&rho1);
    assert!(s1 < s0); // local entropy decrease
}

#[test]
fn theorem_2_flux_guarantees_entanglement() {
    let proj1 = DMatrix::identity(2, 2);
    let proj2 = DMatrix::identity(2, 2);
    let flux_op = DMatrix::from_element(2, 2, 0.1);
    let holonomy = DMatrix::identity(2, 2) + flux_op.clone() * 0.2;
    let f = compute_flux_sector(&proj1, &proj2, &flux_op, &holonomy);
    assert!(f.guarantees_long_range_entanglement());
}
