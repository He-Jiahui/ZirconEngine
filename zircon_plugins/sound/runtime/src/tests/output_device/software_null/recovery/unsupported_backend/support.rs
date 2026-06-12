mod descriptors;
mod errors;

pub(super) use descriptors::{software_null_retry_descriptor, unsupported_native_descriptor};
pub(super) use errors::assert_not_available_error;
