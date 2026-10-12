#[path = "construction.rs"]
mod construction;
#[path = "error.rs"]
mod error;
#[path = "serde.rs"]
mod serde;
#[path = "validation.rs"]
mod validation;
#[path = "value.rs"]
mod value;

pub use error::AssetRefError;
pub use value::AssetRef;
