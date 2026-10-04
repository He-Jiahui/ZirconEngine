use core::mem::size_of;

use crate::ZIRCON_RUNTIME_API_VERSION_V8;

use super::{validate_runtime_api_v8_shape, ZrRuntimeApiV8, ZrRuntimeApiV8ShapeError};

#[test]
fn runtime_api_v8_shape_accepts_the_frozen_exact_table() {
    assert_eq!(
        validate_runtime_api_v8_shape(&ZrRuntimeApiV8::empty()),
        Ok(())
    );
}

#[test]
fn runtime_api_v8_shape_rejects_a_different_api_family() {
    let mut api = ZrRuntimeApiV8::empty();
    api.abi_version = ZIRCON_RUNTIME_API_VERSION_V8 + 1;

    assert_eq!(
        validate_runtime_api_v8_shape(&api),
        Err(ZrRuntimeApiV8ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_API_VERSION_V8,
            actual: ZIRCON_RUNTIME_API_VERSION_V8 + 1,
        })
    );
}

#[test]
fn runtime_api_v8_shape_rejects_a_shorter_table() {
    let mut api = ZrRuntimeApiV8::empty();
    api.size_bytes = size_of::<ZrRuntimeApiV8>() - 1;

    assert_eq!(
        validate_runtime_api_v8_shape(&api),
        Err(ZrRuntimeApiV8ShapeError::TableSizeMismatch {
            expected: size_of::<ZrRuntimeApiV8>(),
            actual: size_of::<ZrRuntimeApiV8>() - 1,
        })
    );
}

#[test]
fn runtime_api_v8_shape_rejects_a_larger_table() {
    let mut api = ZrRuntimeApiV8::empty();
    api.size_bytes = size_of::<ZrRuntimeApiV8>() + 1;

    assert_eq!(
        validate_runtime_api_v8_shape(&api),
        Err(ZrRuntimeApiV8ShapeError::TableSizeMismatch {
            expected: size_of::<ZrRuntimeApiV8>(),
            actual: size_of::<ZrRuntimeApiV8>() + 1,
        })
    );
}
