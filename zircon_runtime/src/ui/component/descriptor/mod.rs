#[path = "validation.rs"]
mod validation;

pub use validation::{validate_component_descriptor, UiComponentDescriptorError};
