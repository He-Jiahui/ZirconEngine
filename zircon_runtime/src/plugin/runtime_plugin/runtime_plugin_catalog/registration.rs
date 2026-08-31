mod constructors;
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) mod order;
mod plugin;
mod reports;
mod update;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) use update::CandidateRows;
pub use update::{
    RuntimePluginCatalogUpdate, RuntimePluginCatalogUpdateMetrics,
    RuntimePluginCatalogUpdateOutcome,
};
