# zeta-quantum

Quantum extension of Zeta Reticula.  
Real-time, calibration-aware compilation with Affine Weyl IR + salience-driven fidelity quantization.

## Installation

```bash
cargo add zeta-quantum
```

## Run the API server

```bash
cargo run --bin zeta-quantum-server
```

The service listens on `http://0.0.0.0:8080`.

## API

- **POST `/optimize`**

Example payload:

```json
{
  "circuit": [
    {"type": "H", "targets": [0]},
    {"type": "CNOT", "targets": [0, 1]}
  ],
  "modality": "superconducting",
  "calibration": {"0-1": 0.013},
  "bms_route": "none"
}
```

## Build & test

```bash
cargo test
```

## Release / deploy

For local deployment, build the server binary:

```bash
cargo build --release --bin zeta-quantum-server
```

Then run `./target/release/zeta-quantum-server`.