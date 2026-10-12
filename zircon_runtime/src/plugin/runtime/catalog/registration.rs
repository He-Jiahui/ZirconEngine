#[path = "registration/constructors.rs"]
mod constructors;
#[path = "registration/order.rs"]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) mod order;
#[path = "registration/plugin.rs"]
mod plugin;
#[path = "registration/reports.rs"]
mod reports;
#[path = "registration/update.rs"]
mod update;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) use update::CandidateRows;
pub use update::{
    RuntimePluginCatalogUpdate, RuntimePluginCatalogUpdateMetrics,
    RuntimePluginCatalogUpdateOutcome,
};
