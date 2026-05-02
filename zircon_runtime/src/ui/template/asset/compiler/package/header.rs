use crate::ui::template::UiCompileCacheKey;
use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiCompileCacheKey as InterfaceUiCompileCacheKey, UiCompiledAssetHeader,
};

pub const UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION: u32 = 1;
pub const UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION: u32 = 1;

pub(super) fn compiled_asset_header_from_cache_key(
    asset: UiAssetHeader,
    cache_key: UiCompileCacheKey,
) -> UiCompiledAssetHeader {
    let interface_cache_key = InterfaceUiCompileCacheKey {
        root_document: cache_key.root_document,
        widget_imports: cache_key.widget_imports.clone(),
        style_imports: cache_key.style_imports.clone(),
        descriptor_registry_revision: cache_key.descriptor_registry_revision,
        component_contract_revision: cache_key.component_contract_revision,
        resource_dependencies_revision: cache_key.resource_dependencies_revision,
    };
    UiCompiledAssetHeader {
        source_schema_version: asset.version,
        compiler_schema_version: UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
        package_schema_version: UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
        descriptor_registry_revision: cache_key.descriptor_registry_revision,
        component_contract_revision: cache_key.component_contract_revision,
        root_document_fingerprint: cache_key.root_document,
        compile_cache_key: interface_cache_key,
        asset,
    }
}
