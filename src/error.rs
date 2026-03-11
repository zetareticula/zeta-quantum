use thiserror::Error;

/// Domain errors for the zeta-quantum compiler/runtime.
#[derive(Debug, Error)]
pub enum ZetaError {
    #[error("disconnected qubits: no path from {start} to {end}")]
    DisconnectedQubits { start: u32, end: u32 },

    #[error("invalid calibration key: {0}")]
    InvalidCalibrationKey(String),

    #[error("invalid density matrix: {0}")]
    InvalidDensityMatrix(&'static str),

    #[error("numerical instability: {0}")]
    Numerical(&'static str),
}

pub type ZetaResult<T> = Result<T, ZetaError>;
