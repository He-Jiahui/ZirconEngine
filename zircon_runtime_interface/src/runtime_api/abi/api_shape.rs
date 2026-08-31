use core::mem::size_of;

use thiserror::Error;

use crate::ZIRCON_RUNTIME_API_VERSION_V8;

use super::api_table::ZrRuntimeApiV8;

/// Reports why a copied runtime API table does not match the frozen V8 shape.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeApiV8ShapeError {
    #[error("runtime API V8 requires version {expected}, received version {actual}")]
    ApiVersionMismatch { expected: u32, actual: u32 },
    #[error("runtime API V8 requires table size {expected}, received {actual}")]
    TableSizeMismatch { expected: usize, actual: usize },
}

/// Validates the frozen V8 family and exact table-size metadata.
///
/// The caller must already own a safely copied table or have independently
/// validated its foreign pointer. This does not validate provider identity,
/// BuildSet membership, required slots, or any function pointer.
pub fn validate_runtime_api_v8_shape(api: &ZrRuntimeApiV8) -> Result<(), ZrRuntimeApiV8ShapeError> {
    if api.abi_version != ZIRCON_RUNTIME_API_VERSION_V8 {
        return Err(ZrRuntimeApiV8ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_API_VERSION_V8,
            actual: api.abi_version,
        });
    }

    let expected = size_of::<ZrRuntimeApiV8>();
    if api.size_bytes != expected {
        return Err(ZrRuntimeApiV8ShapeError::TableSizeMismatch {
            expected,
            actual: api.size_bytes,
        });
    }

    Ok(())
}
