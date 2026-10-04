use core::mem::{align_of, size_of};

use crate::ZIRCON_RUNTIME_ABI_VERSION_V1;

use super::{
    validate_runtime_host_api_v1_pointer, validate_runtime_host_api_v1_shape, ZrHostApiV1,
    ZrRuntimeHostApiV1PointerError, ZrRuntimeHostApiV1ShapeError,
};

#[test]
fn runtime_host_api_v1_shape_accepts_the_frozen_exact_table() {
    assert_eq!(
        validate_runtime_host_api_v1_shape(&ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1)),
        Ok(())
    );
}

#[test]
fn runtime_host_api_v1_shape_rejects_a_different_api_family() {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1);

    assert_eq!(
        validate_runtime_host_api_v1_shape(&host),
        Err(ZrRuntimeHostApiV1ShapeError::ApiVersionMismatch {
            expected: ZIRCON_RUNTIME_ABI_VERSION_V1,
            actual: ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        })
    );
}

#[test]
fn runtime_host_api_v1_shape_rejects_a_non_exact_table_size() {
    let mut host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    host.size_bytes = size_of::<ZrHostApiV1>() - 1;

    assert_eq!(
        validate_runtime_host_api_v1_shape(&host),
        Err(ZrRuntimeHostApiV1ShapeError::TableSizeMismatch {
            expected: size_of::<ZrHostApiV1>(),
            actual: size_of::<ZrHostApiV1>() - 1,
        })
    );
}

#[test]
fn runtime_host_api_v1_pointer_accepts_null_as_an_absent_optional_table() {
    assert_eq!(
        unsafe { validate_runtime_host_api_v1_pointer(core::ptr::null()) },
        Ok(())
    );
}

#[test]
fn runtime_host_api_v1_pointer_rejects_misalignment_before_dereference() {
    let storage = [0_u8; size_of::<ZrHostApiV1>() + align_of::<ZrHostApiV1>()];
    let pointer = unsafe { storage.as_ptr().add(1) }.cast::<ZrHostApiV1>();

    assert_ne!((pointer as usize) % align_of::<ZrHostApiV1>(), 0);
    assert_eq!(
        unsafe { validate_runtime_host_api_v1_pointer(pointer) },
        Err(ZrRuntimeHostApiV1PointerError::MisalignedPointer {
            address: pointer as usize,
            expected_alignment: align_of::<ZrHostApiV1>(),
        })
    );
}
