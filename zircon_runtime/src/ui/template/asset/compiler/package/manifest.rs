use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetFingerprint, UiCompileCacheKey, UiCompiledAssetDependency,
    UiCompiledAssetDependencyManifest, UiLocalizationDependency, UiResourceDependency,
};

// 与同次编译的缓存键组成依赖清单，供包读取方追踪导入、资源和本地化失效；不在打包时重新加载依赖文件。
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

// 用注册资产描述身份、用已计算摘要描述版本；只记录两边都存在的引用，保持有序清单的可重复性。
fn dependency_entries(
    imports: &BTreeMap<String, UiAssetDocument>,
    fingerprints: &BTreeMap<String, UiAssetFingerprint>,
) -> Vec<UiCompiledAssetDependency> {
    let mut dependencies = Vec::with_capacity(imports.len().min(fingerprints.len()));
    for (reference, document) in imports {
        let Some(fingerprint) = fingerprints.get(reference) else {
            continue;
        };
        dependencies.push(UiCompiledAssetDependency {
            reference: reference.clone(),
            asset_id: document.asset.id.clone(),
            asset_kind: document.asset.kind,
            source_schema_version: document.asset.version,
            fingerprint: *fingerprint,
        });
    }
    dependencies
}

// 源码形状门槛约束已审查的容量路径；ignored 计时只比较合成行收集成本，包语义回归由外层资产包测试负责。
#[cfg(test)]
#[path = "tests/manifest_performance_tests.rs"]
mod performance_tests;
