//! Persistent, folder-backed project asset index and dependency graph.

mod asset_registry_diagnostic;
mod asset_registry_entry;
mod asset_registry_error;
mod asset_registry_filter;
mod asset_registry_index;
mod deletion;
pub(crate) mod dependency_extractors;
mod incremental;
mod inspection;
pub(crate) mod persistence;
mod query;
mod rebuild;
mod relocation;
mod targeted;

pub use asset_registry_diagnostic::AssetRegistryDiagnostic;
pub use asset_registry_entry::AssetRegistryEntry;
pub use asset_registry_error::AssetRegistryError;
pub use asset_registry_filter::AssetRegistryFilter;
pub use asset_registry_index::AssetRegistryIndex;
