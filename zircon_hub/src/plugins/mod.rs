#[path = "catalog.rs"]
mod catalog;

pub use catalog::{
    ENGINE_PLUGIN_SCOPE, PROJECT_PLUGIN_SCOPE, PluginCatalogEntry, discover_plugin_catalog,
    discover_plugin_catalog_with_project_roots,
};
