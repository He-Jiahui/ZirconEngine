use serde::{Deserialize, Serialize};

use crate::ui::template::{
    UiAssetFingerprint, UiAssetHeader, UiCompileCacheKey, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

pub const UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION: u32 = 1;
pub const UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION: u32 = 8;

/// 编译包的身份与版本边界：源格式、编译器格式和包格式独立演进。
/// 运行时生成它并用于持久缓存兼容性判断；`is_current_source_schema` 只检查源格式。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetHeader {
    pub asset: UiAssetHeader,
    pub source_schema_version: u32,
    pub compiler_schema_version: u32,
    pub package_schema_version: u32,
    pub descriptor_registry_revision: u64,
    pub component_contract_revision: UiAssetFingerprint,
    pub root_document_fingerprint: UiAssetFingerprint,
    pub compile_cache_key: UiCompileCacheKey,
}

impl UiCompiledAssetHeader {
    /// 该判定只比较源资产 schema；编译器和包封套版本仍由各自字段单独校验。
    pub fn is_current_source_schema(&self) -> bool {
        self.source_schema_version == UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }
}
