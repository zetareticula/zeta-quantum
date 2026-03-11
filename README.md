# zeta-quantum

[![CI](https://github.com/zetareticula/zeta-quantum/actions/workflows/ci.yml/badge.svg)](https://github.com/zetareticula/zeta-quantum/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/zeta-quantum.svg)](https://crates.io/crates/zeta-quantum)
[![docs.rs](https://docs.rs/zeta-quantum/badge.svg)](https://docs.rs/zeta-quantum)
[![License](https://img.shields.io/github/license/zetareticula/zeta-quantum.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2021-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![Last commit](https://img.shields.io/github/last-commit/zetareticula/zeta-quantum.svg)](https://github.com/zetareticula/zeta-quantum/commits/main)

Quantum extension of Zeta Reticula.  
Real-time, calibration-aware compilation with Affine Weyl IR + salience-driven fidelity quantization.

`zeta-quantum` is a Rust library + HTTP service that:

- Compiles circuits into **Phi-IR** (Affine Weyl group words)
- Estimates **integrated obstruction** (`S_X`) using a modality-aware QPU model
- Projects obstruction into **BMS-observable** channels (with optional escape routes)
- Computes toy implementations of:
  - **Theorem 1**: nonlocal Lindblad evolution + subsystem reduction
  - **Theorem 2**: flux/holonomy sector + entanglement witness
  - **Radiative visibility**: when entropy could become geometric (new physics options)

## Status

- Library crate: `zeta-quantum`
- Server binary: `zeta-quantum-server` (Axum)

## Installation

```bash
cargo add zeta-quantum
```

## Quickstart (library)

```rust
use std::collections::HashMap;
use zeta_quantum::{EscapeRoute, Modality, PhiCircuit, PhiElement, QuantumQuantizer};

let mut calibration = HashMap::new();
calibration.insert("0-1".to_string(), 0.005);
calibration.insert("1-2".to_string(), 0.004);

let mut q = QuantumQuantizer::new(Modality::Superconducting, &calibration, "demo".into());

let mut circ = PhiCircuit::default();
circ.elements.push(PhiElement::h(0));
circ.elements.push(PhiElement::cnot(0, 2));

let (_optimized, integrated_obstruction, bms) = q.quantize_with_bms(&circ, EscapeRoute::None)?;
println!("S_X={integrated_obstruction}, decoded={:?}", bms.decoded_sx);
# Ok::<(), anyhow::Error>(())
```

## Architecture

### Data flow

```mermaid
flowchart LR
  A[Circuit JSON / PhiCircuit] --> B[Phi-IR
  (Affine Weyl words)]
  B --> C[QPU model
  (modality + calibration graph)]
  C --> D[Cost / integrated obstruction
  S_X]
  D --> E[BMS projection
  (observable + escape route)]
  D --> F[Theorem 1
  nonlocal dynamics]
  D --> G[Theorem 2
  flux/holonomy]
  E --> H[Radiative visibility]
```

### Module map

```mermaid
flowchart TB
  subgraph Core
    phi_ir[phi_ir.rs
    PhiCircuit/PhiElement/WeylGen]
    qpu[qpu.rs
    QPU/Modality + Dijkstra]
    cost[cost.rs
    integrated_obstruction]
    cache[cache.rs
    LRU path cache]
    err[error.rs
    ZetaError]
  end
  subgraph Theory
    bms[bms.rs
    BMSObservable/EscapeRoute]
    nld[nonlocal_dynamics.rs
    entropy + Lindblad step]
    flux[flux_holonomy.rs
    FluxSector]
    rad[radiative_entropy.rs
    RadiativeVisibility]
  end
  lib[lib.rs
  QuantumQuantizer]
  api[api.rs
  Axum + OpenAPI]

  phi_ir --> lib
  qpu --> lib
  cost --> lib
  cache --> qpu
  bms --> lib
  nld --> lib
  flux --> lib
  rad --> lib
  lib --> api
```

## Run the API server

```bash
cargo run --bin zeta-quantum-server
```

The service listens on `http://0.0.0.0:8080`.

### OpenAPI

- `GET /openapi.json` (advertised on startup; may be wired via your router configuration)

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

### Notes

- `modality`:
  - `superconducting`
  - `iontrap`
  - `neutralatom`
- `calibration` keys use the format `"<qA>-<qB>" -> error_rate`.
- `bms_route`:
  - `none`
  - `scalar`
  - `holographic`

## Configuration

- `cargo add zeta-quantum` to add the library to your project
- `cargo run --bin zeta-quantum-server` to run the API server

## Development workflow

- `cargo test` to run tests
- `cargo build --release --bin zeta-quantum-server` to build the server binary
- `./target/release/zeta-quantum-server` to run the server

## Performance

- `QPU::find_optimal_path` is Dijkstra-based.
- A global **LRU cache** memoizes shortest paths across requests to speed repeated routing.

## Numerical safety (current model)

- `von_neumann_entropy` uses eigenvalues of the symmetrized density matrix and clamps small negative eigenvalues.
- Flux/holonomy computations apply an epsilon threshold to suppress floating-point noise.

## Release / deploy

For local deployment, build the server binary:

```bash
cargo build --release --bin zeta-quantum-server
```

Then run `./target/release/zeta-quantum-server`.

## License

See `LICENSE`.