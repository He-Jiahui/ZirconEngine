use serde::{Deserialize, Serialize};

use crate::ui::template::{
    UiAssetFingerprint, UiAssetHeader, UiCompileCacheKey, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

pub const UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION: u32 = 1;
pub const UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION: u32 = 1;

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
    pub fn is_current_source_schema(&self) -> bool {
        self.source_schema_version == UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    }
}
