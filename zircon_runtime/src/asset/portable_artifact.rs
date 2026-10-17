//! Portable cooked resource decoding and the shared semantic block loaders.
#[path = "artifact/cache_payload.rs"]
mod cache_payload;
#[path = "artifact/raw.rs"]
mod raw;
pub use raw::{
    read_raw_asset, write_raw_asset, RawAssetCatalog, RawAssetCatalogEntry, RawAssetError,
    RawSceneCatalogEntry, RAW_ASSET_MAX_BYTES, RAW_CATALOG_MAX_BYTES,
};

#[path = "artifact/render_manifest/mod.rs"]
mod render_manifest;
pub use render_manifest::*;
