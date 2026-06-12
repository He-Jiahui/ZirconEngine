use crate::script::VmError;
use zr_vm_rust_binding as zrvm;
use zr_vm_rust_binding_sys as zrvm_sys;

pub(super) fn map_zr_error(error: zrvm::Error) -> VmError {
    VmError::Operation(format!("zr_vm binding error: {error}"))
}

pub(super) fn zr_error(message: impl Into<String>) -> zrvm::Error {
    zrvm::Error::new(
        zrvm_sys::ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_INTERNAL_ERROR,
        message,
    )
}

pub(super) fn is_optional_export_missing(error: &zrvm::Error) -> bool {
    error.status == zrvm_sys::ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_NOT_FOUND
        || error.message.contains("not found")
        || error.message.contains("NOT_FOUND")
}
