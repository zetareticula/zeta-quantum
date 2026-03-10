#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use zeta_quantum::phi_ir::{PhiCircuit, PhiElement};
    use zeta_quantum::{Modality, QuantumQuantizer, QPU};

    fn quantizer_from_calib_json(
        modality: Modality,
        calib_json: &str,
        ts: &str,
    ) -> QuantumQuantizer {
        let calib: HashMap<String, f64> = serde_json::from_str(calib_json).unwrap();
        QuantumQuantizer::new(modality, &calib, ts.into())
    }

    #[test]
    fn superconducting_routing_is_optimal() {
        let mut q = QPU::new(Modality::Superconducting, "2026-03-05".into());
        // Small Sycamore-like graph
        q.add_coupling(0, 1, 0.005);
        q.add_coupling(1, 2, 0.004);
        q.add_coupling(0, 3, 0.100); // high-noise direct link
        q.add_coupling(3, 2, 0.006);

        let path = q.find_optimal_path(0, 2);
        assert_eq!(path, vec![0, 1, 2]); // must avoid high-noise edge
        assert!(q.gate_cost(0, 2) < 0.02);
    }

    #[test]
    fn quantize_produces_lower_sx_on_calibrated_hardware() {
        let calib = r#"{"0-1":0.005,"0-3":0.100,"1-2":0.004}"#;
        let mut q = quantizer_from_calib_json(Modality::Superconducting, calib, "test");

        let mut circ = PhiCircuit::default();
        circ.elements.push(PhiElement::cnot(0, 2));

        let (_, sx) = q.quantize(&circ).unwrap();
        assert!(sx < 0.02); // routed via 0-1-2
    }
}
