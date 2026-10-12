#[path = "error.rs"]
mod error;
#[path = "validate.rs"]
mod validate;

pub use error::ProjectNameError;
pub use validate::validate_project_name;
