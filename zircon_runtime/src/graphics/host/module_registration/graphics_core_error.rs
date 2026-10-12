use crate::core::CoreError;

pub(super) fn graphics_core_error(service: &str, error: impl ToString) -> CoreError {
    CoreError::Initialization(service.to_string(), error.to_string())
}
