mod descriptors;
mod errors;

pub(crate) use descriptors::{software_null_retry_descriptor, unsupported_native_descriptor};
pub(super) use errors::assert_not_available_error;
