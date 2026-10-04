use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiCompileCacheKey, UiCompiledAssetHeader,
    UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION, UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
};

// 包头同时保留源格式、编译器与包 schema；这些版本承担不同兼容边界，不能只凭资产版本判断可复用。
pub(super) fn compiled_asset_header_from_cache_key(
    asset: UiAssetHeader,
    cache_key: UiCompileCacheKey,
) -> UiCompiledAssetHeader {
    UiCompiledAssetHeader {
        source_schema_version: asset.version,
        compiler_schema_version: UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
        package_schema_version: UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
        descriptor_registry_revision: cache_key.descriptor_registry_revision,
        component_contract_revision: cache_key.component_contract_revision,
        root_document_fingerprint: cache_key.root_document,
        compile_cache_key: cache_key,
        asset,
    }
}
