use core::mem::{align_of, size_of};

use thiserror::Error;

use crate::ZIRCON_RUNTIME_ABI_VERSION_V1;

use super::api_table::ZrHostApiV1;

/// Reports why a copied V1 host callback table does not match the frozen shape.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeHostApiV1ShapeError {
    #[error("runtime host API V1 requires version {expected}, received version {actual}")]
    ApiVersionMismatch { expected: u32, actual: u32 },
    #[error("runtime host API V1 requires table size {expected}, received {actual}")]
    TableSizeMismatch { expected: usize, actual: usize },
}

/// Reports why an optional host-table pointer cannot be inspected safely enough to read metadata.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeHostApiV1PointerError {
    #[error(
        "runtime host API V1 pointer {address:#x} is not aligned to required alignment {expected_alignment}"
    )]
    MisalignedPointer {
        address: usize,
        expected_alignment: usize,
    },
    #[error(transparent)]
    Shape(#[from] ZrRuntimeHostApiV1ShapeError),
}

/// Validates the frozen V1 host-table version and exact table-size metadata.
///
/// The callback slots are optional in V1. Capability requirements and callback
/// dependencies belong to the later handshake receipt, not this shape check.
pub fn validate_runtime_host_api_v1_shape(
    host: &ZrHostApiV1,
) -> Result<(), ZrRuntimeHostApiV1ShapeError> {
    if host.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return Err(ZrRuntimeHostApiV1ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_ABI_VERSION_V1,
            actual: host.abi_version,
        });
    }

    let expected = size_of::<ZrHostApiV1>();
    if host.size_bytes != expected {
        return Err(ZrRuntimeHostApiV1ShapeError::TableSizeMismatch {
            expected,
            actual: host.size_bytes,
        });
    }

    Ok(())
}

/// Validates an optional raw V1 host-table pointer before reading its metadata.
///
/// A null pointer is the V8 entry point's established "no host callbacks"
/// form. A non-null pointer must be aligned and point to a readable
/// `ZrHostApiV1` for this call. Address readability cannot be established
/// safely in-process; untrusted callers require the planned process-isolated
/// ABI boundary.
///
/// # Safety
///
/// For a non-null pointer, `host` must be valid to read a `ZrHostApiV1` for
/// the duration of this call. The function checks alignment before creating a
/// reference but cannot make an unmapped or dangling address safe to read.
pub unsafe fn validate_runtime_host_api_v1_pointer(
    host: *const ZrHostApiV1,
) -> Result<(), ZrRuntimeHostApiV1PointerError> {
    if host.is_null() {
        return Ok(());
    }

    let address = host as usize;
    let expected_alignment = align_of::<ZrHostApiV1>();
    if address % expected_alignment != 0 {
        return Err(ZrRuntimeHostApiV1PointerError::MisalignedPointer {
            address,
            expected_alignment,
        });
    }

    let host = unsafe { &*host };
    validate_runtime_host_api_v1_shape(host)?;
    Ok(())
}
