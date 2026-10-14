#[path = "catalog.rs"]
mod catalog;

pub use catalog::{
    AssetCatalogEntry, PROJECT_ASSET_SOURCE, SELECTED_PROJECT_ASSET_SOURCE, discover_asset_catalog,
    discover_asset_catalog_for_scope,
};
