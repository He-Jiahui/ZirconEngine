#[path = "../../../abi/support/callbacks/capture.rs"]
mod capture;
#[path = "../../../abi/support/callbacks/failure.rs"]
mod failure;

pub(crate) use capture::capture_abi_callback;
pub(crate) use failure::failing_abi_callback;
