mod flat_nodes;
mod legacy_template;
mod migrator;
mod policy;

pub(crate) use flat_nodes::load_flat_prototype_toml_str;
pub use migrator::UiAssetSchemaMigrator;
pub use policy::{
    UiAssetSchemaVersionPolicy, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
    UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION,
};
