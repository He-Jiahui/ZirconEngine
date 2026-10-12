#[path = "catalog.rs"]
mod catalog;
#[path = "error.rs"]
mod error;
#[path = "schema.rs"]
mod schema;

pub use catalog::{
    VmReflectionCatalog, VmReflectionRegistrySnapshot, VM_REFLECTION_WORLD_EXTENSION_NAME,
};
pub use error::VmReflectionError;
pub use schema::VmReflectionSchema;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
