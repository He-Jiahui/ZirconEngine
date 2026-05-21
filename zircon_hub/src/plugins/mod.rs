mod catalog;

pub use catalog::{
    discover_plugin_catalog, discover_plugin_catalog_with_project_roots, PluginCatalogEntry,
    ENGINE_PLUGIN_SCOPE, PROJECT_PLUGIN_SCOPE,
};
