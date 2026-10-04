pub const UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION: u32 = 3;
pub const UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION: u32 = 1;

/// 源资产版本的接纳窗口；迁移器拒绝窗口外版本后再升级受支持文档。
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

    /// 迁移前先用含端点的支持窗口判定版本；未来版本与低于下限的版本均不能进入常规升级。
    pub const fn is_supported_source_schema(version: u32) -> bool {
        version >= UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION
            && version <= UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }

    /// 仅供已通过支持范围检查的版本使用；过低的无效版本也会返回 true。
    pub const fn requires_source_schema_migration(version: u32) -> bool {
        version < UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }
}
