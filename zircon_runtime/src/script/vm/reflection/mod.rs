mod catalog;
mod error;
mod schema;

pub use catalog::{
    VmReflectionCatalog, VmReflectionRegistrySnapshot, VM_REFLECTION_WORLD_EXTENSION_NAME,
};
pub use error::VmReflectionError;
pub use schema::VmReflectionSchema;

#[cfg(test)]
mod tests;
