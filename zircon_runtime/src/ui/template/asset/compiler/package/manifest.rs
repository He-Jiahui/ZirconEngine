use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetFingerprint, UiCompileCacheKey, UiCompiledAssetDependency,
    UiCompiledAssetDependencyManifest, UiLocalizationDependency, UiResourceDependency,
};

pub(super) fn compiled_asset_dependency_manifest_from_imports(
    _document: &UiAssetDocument,
    cache_key: &UiCompileCacheKey,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
    resource_dependencies: &[UiResourceDependency],
    localization_dependencies: &[UiLocalizationDependency],
) -> UiCompiledAssetDependencyManifest {
    UiCompiledAssetDependencyManifest {
        widget_imports: dependency_entries(widget_imports, &cache_key.widget_imports),
        style_imports: dependency_entries(style_imports, &cache_key.style_imports),
        resource_dependencies: resource_dependencies.to_vec(),
        localization_dependencies: localization_dependencies.to_vec(),
    }
}

fn dependency_entries(
    imports: &BTreeMap<String, UiAssetDocument>,
    fingerprints: &BTreeMap<String, UiAssetFingerprint>,
) -> Vec<UiCompiledAssetDependency> {
    imports
        .iter()
        .filter_map(|(reference, document)| {
            fingerprints
                .get(reference)
                .map(|fingerprint| UiCompiledAssetDependency {
                    reference: reference.clone(),
                    asset_id: document.asset.id.clone(),
                    asset_kind: document.asset.kind,
                    source_schema_version: document.asset.version,
                    fingerprint: *fingerprint,
                })
        })
        .collect()
}
