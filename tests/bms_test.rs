//! Copyright (c) 2026 Zeta Reticula Inc
//! Licensed under the MIT License. See LICENSE for details.

#[cfg(test)]
mod bms_tests {
    use std::collections::HashMap;

    use zeta_quantum::phi_ir::{PhiCircuit, PhiElement};
    use zeta_quantum::{EscapeRoute, Modality, QuantumQuantizer};

    #[test]
    fn generic_gravity_cannot_decode_detailed_sx() {
        let mut calib = HashMap::new();
        calib.insert("0-1".to_string(), 0.005);
        calib.insert("1-2".to_string(), 0.004);

        let mut circ = PhiCircuit::default();
        circ.elements.push(PhiElement::cnot(0, 2)); // triggers Affine gen → r > 0

        let mut quant =
            QuantumQuantizer::new(Modality::Superconducting, &calib, "2026-03-05".into());
        let (_, _, obs) = quant.quantize_with_bms(&circ, EscapeRoute::None).unwrap();

        assert_eq!(obs.decoded_sx, None); // your verdict
        assert!(obs.phase_transition_detected); // detects jump, not value
    }

    #[test]
    fn scalar_engineering_restores_full_sx() {
        let mut calib = HashMap::new();
        calib.insert("0-1".to_string(), 0.005);
        calib.insert("1-2".to_string(), 0.004);

        let mut circ = PhiCircuit::default();
        circ.elements.push(PhiElement::cnot(0, 2));

        let mut quant =
            QuantumQuantizer::new(Modality::Superconducting, &calib, "2026-03-05".into());
        let (_, _, obs) = quant
            .quantize_with_bms(&circ, EscapeRoute::ScalarMetricEngineering)
            .unwrap();
        assert!(obs.decoded_sx.is_some());
    }
}
