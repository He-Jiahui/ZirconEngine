//! Read-only model/instance facts; semantic asset routing and preview stay with their owners.

#[path = "adapter.rs"]
mod adapter;
#[path = "error.rs"]
mod error;
#[path = "source.rs"]
mod source;

pub use adapter::StaticMeshSourceAdapter;
pub use error::StaticMeshSourceError;
pub use source::{StaticMeshInstanceObservation, StaticMeshSource};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
