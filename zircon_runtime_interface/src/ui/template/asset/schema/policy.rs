pub const UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION: u32 = 3;
pub const UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiAssetSchemaVersionPolicy;

impl UiAssetSchemaVersionPolicy {
    pub const fn current_source_schema_version() -> u32 {
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }

    pub const fn minimum_supported_source_schema_version() -> u32 {
        UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION
    }

    pub const fn is_future_source_schema(version: u32) -> bool {
        version > UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }

    pub const fn is_supported_source_schema(version: u32) -> bool {
        version >= UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION
            && version <= UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }

    pub const fn requires_source_schema_migration(version: u32) -> bool {
        version < UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }
}
