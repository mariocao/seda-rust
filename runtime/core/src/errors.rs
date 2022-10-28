use std::{array::TryFromSliceError, str::Utf8Error};

use error_stack::Report;
use thiserror::Error;
use wasmer::{CompileError, ExportError};
use wasmer_wasi::WasiStateCreationError;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("{0:?}")]
    StringBytesConversion(Report<Utf8Error>),
    #[error("{0}")]
    NumBytesConversion(Report<TryFromSliceError>),

    #[error("{0}")]
    WasmCompileError(Report<CompileError>),

    #[error("{0}")]
    WasiStateCreationError(Report<WasiStateCreationError>),

    #[error("{0}")]
    FunctionNotFound(Report<ExportError>),

    #[error("Error while running: {0}")]
    ExecutionError(Report<wasmer::RuntimeError>),
}

impl From<Report<Utf8Error>> for RuntimeError {
    fn from(r: Report<Utf8Error>) -> Self {
        Self::StringBytesConversion(r)
    }
}

impl From<Report<TryFromSliceError>> for RuntimeError {
    fn from(r: Report<TryFromSliceError>) -> Self {
        Self::NumBytesConversion(r)
    }
}

impl From<CompileError> for RuntimeError {
    fn from(r: CompileError) -> Self {
        Self::WasmCompileError(r.into())
    }
}

impl From<WasiStateCreationError> for RuntimeError {
    fn from(r: WasiStateCreationError) -> Self {
        Self::WasiStateCreationError(r.into())
    }
}

impl From<ExportError> for RuntimeError {
    fn from(r: ExportError) -> Self {
        Self::FunctionNotFound(r.into())
    }
}

impl From<wasmer::RuntimeError> for RuntimeError {
    fn from(r: wasmer::RuntimeError) -> Self {
        Self::ExecutionError(r.into())
    }
}

pub type Result<T, E = RuntimeError> = core::result::Result<T, E>;