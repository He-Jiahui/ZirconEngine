use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::asset::AssetImporterDescriptor;
use crate::core::framework::project::ExportPackagingStrategy;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{
    PluginFeatureBundleManifest, PluginModuleManifest, PluginPackageKind, PluginPackageManifest,
    PluginShaderModuleSource, UiComponentDescriptor,
};
use serde::Serialize;

use super::{projection::ProjectionBuildStats, NativePluginLoadReport};

const MAX_SHADER_MODULES_PER_PACKAGE: usize = 64;
const MAX_SHADER_MODULE_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_SHADER_MODULE_SOURCE_TOTAL_BYTES: u64 = 16 * 1024 * 1024;

impl NativePluginLoadReport {
    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        self.projection().package_manifests().to_vec()
    }
}

pub(super) fn projected_package_manifests(
    report: &NativePluginLoadReport,
    stats: &mut ProjectionBuildStats,
) -> Vec<PluginPackageManifest> {
    let mut builder = ManifestProjectionBuilder::default();
    for candidate in &report.discovered {
        stats.manifest_sources_scanned += 1;
        builder.insert_discovered(candidate.package_manifest.clone(), stats);
    }
    for plugin in &report.loaded {
        for manifest in [
            plugin
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.package_manifest.clone()),
            plugin
                .runtime_entry_report
                .as_ref()
                .and_then(|entry| entry.package_manifest.clone()),
            plugin
                .editor_entry_report
                .as_ref()
                .and_then(|entry| entry.package_manifest.clone()),
        ]
        .into_iter()
        .flatten()
        {
            stats.manifest_sources_scanned += 1;
            builder.merge(manifest, stats);
        }
    }
    builder.finish()
}

pub(super) fn shader_module_sources_from_candidate(
    candidate: &crate::plugin::native_plugin_loader::NativePluginCandidate,
) -> (Vec<PluginShaderModuleSource>, Vec<String>) {
    let mut sources = Vec::new();
    let mut diagnostics = Vec::new();
    let mut seen_import_paths = HashSet::new();
    let declared_modules = &candidate.package_manifest.shader_permutation.shader_modules;
    if declared_modules.len() > MAX_SHADER_MODULES_PER_PACKAGE {
        diagnostics.push(format!(
            "native plugin {} declares {} shader modules, above the {} module limit",
            candidate.plugin_id,
            declared_modules.len(),
            MAX_SHADER_MODULES_PER_PACKAGE
        ));
    }
    let Some(package_root) = candidate.manifest_path.parent() else {
        diagnostics.push(format!(
            "native plugin {} has no package root for shader module resolution",
            candidate.plugin_id
        ));
        return (sources, diagnostics);
    };
    let canonical_package_root = match fs::canonicalize(package_root) {
        Ok(path) => path,
        Err(error) => {
            diagnostics.push(format!(
                "native plugin {} cannot resolve package root {} for shader modules: {error}",
                candidate.plugin_id,
                package_root.display()
            ));
            return (sources, diagnostics);
        }
    };
    let mut total_source_bytes = 0_u64;
    for module in declared_modules.iter().take(MAX_SHADER_MODULES_PER_PACKAGE) {
        let origin = format!(
            "native plugin {} shader module `{}` ({})",
            candidate.plugin_id, module.import_path, module.source
        );
        if !seen_import_paths.insert(module.import_path.clone()) {
            diagnostics.push(format!(
                "{origin} duplicates an import path in the same package"
            ));
            continue;
        }
        let source_path =
            match package_shader_module_source_path(&canonical_package_root, &module.source) {
                Ok(path) => path,
                Err(message) => {
                    diagnostics.push(format!("{origin}: {message}"));
                    continue;
                }
            };
        match fs::metadata(&source_path) {
            Ok(metadata) if metadata.len() <= MAX_SHADER_MODULE_SOURCE_BYTES => {}
            Ok(metadata) => {
                diagnostics.push(format!(
                    "{origin}: source {} is {} bytes, above the {} byte limit",
                    source_path.display(),
                    metadata.len(),
                    MAX_SHADER_MODULE_SOURCE_BYTES
                ));
                continue;
            }
            Err(error) => {
                diagnostics.push(format!(
                    "{origin}: cannot inspect source {}: {error}",
                    source_path.display()
                ));
                continue;
            }
        }
        let bytes = match fs::read(&source_path) {
            Ok(bytes) if bytes.len() as u64 <= MAX_SHADER_MODULE_SOURCE_BYTES => bytes,
            Ok(bytes) => {
                diagnostics.push(format!(
                    "{origin}: source {} is {} bytes, above the {} byte limit",
                    source_path.display(),
                    bytes.len(),
                    MAX_SHADER_MODULE_SOURCE_BYTES
                ));
                continue;
            }
            Err(error) => {
                diagnostics.push(format!(
                    "{origin}: cannot read source {}: {error}",
                    source_path.display()
                ));
                continue;
            }
        };
        let source_len = bytes.len() as u64;
        if total_source_bytes.saturating_add(source_len) > MAX_SHADER_MODULE_SOURCE_TOTAL_BYTES {
            diagnostics.push(format!(
                "{origin}: source {} would exceed the {} byte package shader-module budget",
                source_path.display(),
                MAX_SHADER_MODULE_SOURCE_TOTAL_BYTES
            ));
            continue;
        }
        let source = match String::from_utf8(bytes) {
            Ok(source) => source,
            Err(error) => {
                diagnostics.push(format!(
                    "{origin}: source {} is not UTF-8: {error}",
                    source_path.display()
                ));
                continue;
            }
        };
        total_source_bytes += source_len;
        sources.push(PluginShaderModuleSource::new(
            candidate.plugin_id.clone(),
            module.import_path.clone(),
            source,
            origin,
        ));
    }
    (sources, diagnostics)
}

fn package_shader_module_source_path(package_root: &Path, source: &str) -> Result<PathBuf, String> {
    let relative = Path::new(source);
    if source.is_empty()
        || source.contains('\\')
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::CurDir
                    | Component::ParentDir
            )
        })
    {
        return Err("source must be a package-relative path".to_string());
    }
    if !matches!(
        relative
            .extension()
            .and_then(|extension| extension.to_str()),
        Some("zshader" | "wgsl")
    ) {
        return Err("source must end with .zshader or .wgsl".to_string());
    }
    let canonical_source = fs::canonicalize(package_root.join(relative)).map_err(|error| {
        format!(
            "cannot resolve source {}: {error}",
            package_root.join(relative).display()
        )
    })?;
    if !canonical_source.starts_with(package_root) {
        return Err(format!(
            "source {} resolves outside package root {}",
            canonical_source.display(),
            package_root.display()
        ));
    }
    Ok(canonical_source)
}

pub(super) fn merge_package_manifest(
    manifests: &mut BTreeMap<String, PluginPackageManifest>,
    manifest: PluginPackageManifest,
) {
    let Some(existing) = manifests.remove(&manifest.id) else {
        manifests.insert(manifest.id.clone(), manifest);
        return;
    };

    let mut accumulator = ManifestAccumulator::new(existing);
    accumulator.merge(manifest);
    let merged = accumulator.finish();
    manifests.insert(merged.id.clone(), merged);
}

#[derive(Default)]
struct ManifestProjectionBuilder {
    manifests: Vec<ManifestAccumulator>,
    manifest_indices: HashMap<String, usize>,
}

impl ManifestProjectionBuilder {
    fn insert_discovered(
        &mut self,
        manifest: PluginPackageManifest,
        stats: &mut ProjectionBuildStats,
    ) {
        stats.manifest_package_index_lookups += 1;
        if let Some(&index) = self.manifest_indices.get(&manifest.id) {
            self.manifests[index] = ManifestAccumulator::new(manifest);
        } else {
            self.push_new(manifest);
        }
    }

    fn merge(&mut self, manifest: PluginPackageManifest, stats: &mut ProjectionBuildStats) {
        stats.manifest_package_index_lookups += 1;
        if let Some(&index) = self.manifest_indices.get(&manifest.id) {
            self.manifests[index].merge(manifest);
        } else {
            self.push_new(manifest);
        }
    }

    fn finish(self) -> Vec<PluginPackageManifest> {
        let mut manifests = self
            .manifests
            .into_iter()
            .map(ManifestAccumulator::finish)
            .collect::<Vec<_>>();
        manifests.sort_by(|left, right| left.id.cmp(&right.id));
        manifests
    }

    fn push_new(&mut self, manifest: PluginPackageManifest) {
        let index = self.manifests.len();
        self.manifest_indices.insert(manifest.id.clone(), index);
        self.manifests.push(ManifestAccumulator::new(manifest));
    }
}

struct ManifestAccumulator {
    manifest: PluginPackageManifest,
    capabilities: EqualityIndex<String>,
    modules: EqualityIndex<PluginModuleManifest>,
    components: EqualityIndex<ComponentTypeDescriptor>,
    ui_components: EqualityIndex<UiComponentDescriptor>,
    asset_importers: EqualityIndex<AssetImporterDescriptor>,
    optional_features: EqualityIndex<PluginFeatureBundleManifest>,
    feature_extensions: EqualityIndex<PluginFeatureBundleManifest>,
    default_packaging: EqualityIndex<ExportPackagingStrategy>,
}

impl ManifestAccumulator {
    fn new(manifest: PluginPackageManifest) -> Self {
        Self {
            capabilities: EqualityIndex::new(&manifest.capabilities),
            modules: EqualityIndex::new(&manifest.modules),
            components: EqualityIndex::new(&manifest.components),
            ui_components: EqualityIndex::new(&manifest.ui_components),
            asset_importers: EqualityIndex::new(&manifest.asset_importers),
            optional_features: EqualityIndex::new(&manifest.optional_features),
            feature_extensions: EqualityIndex::new(&manifest.feature_extensions),
            default_packaging: EqualityIndex::new(&manifest.default_packaging),
            manifest,
        }
    }

    fn merge(&mut self, manifest: PluginPackageManifest) {
        if !manifest.version.is_empty() {
            self.manifest.version = manifest.version;
        }
        if !manifest.display_name.is_empty() {
            self.manifest.display_name = manifest.display_name;
        }
        if !manifest.description.is_empty() {
            self.manifest.description = manifest.description;
        }
        if manifest.package_kind != PluginPackageKind::Standard {
            self.manifest.package_kind = manifest.package_kind;
        }
        append_unique(
            &mut self.manifest.capabilities,
            manifest.capabilities,
            &mut self.capabilities,
        );
        append_unique(
            &mut self.manifest.modules,
            manifest.modules,
            &mut self.modules,
        );
        append_unique(
            &mut self.manifest.components,
            manifest.components,
            &mut self.components,
        );
        append_unique(
            &mut self.manifest.ui_components,
            manifest.ui_components,
            &mut self.ui_components,
        );
        append_unique(
            &mut self.manifest.asset_importers,
            manifest.asset_importers,
            &mut self.asset_importers,
        );
        append_unique(
            &mut self.manifest.optional_features,
            manifest.optional_features,
            &mut self.optional_features,
        );
        append_unique(
            &mut self.manifest.feature_extensions,
            manifest.feature_extensions,
            &mut self.feature_extensions,
        );
        append_unique(
            &mut self.manifest.default_packaging,
            manifest.default_packaging,
            &mut self.default_packaging,
        );
    }

    fn finish(self) -> PluginPackageManifest {
        self.manifest
    }
}

struct EqualityIndex<T> {
    buckets: HashMap<String, Vec<T>>,
}

impl<T> EqualityIndex<T>
where
    T: Clone + PartialEq + Serialize,
{
    fn new(values: &[T]) -> Self {
        let mut index = Self {
            buckets: HashMap::new(),
        };
        for value in values {
            index.insert(value);
        }
        index
    }

    fn insert(&mut self, value: &T) -> bool {
        let bucket = self.buckets.entry(stable_key(value)).or_default();
        if bucket.iter().any(|existing| existing == value) {
            return false;
        }
        bucket.push(value.clone());
        true
    }
}

fn append_unique<T>(target: &mut Vec<T>, source: Vec<T>, seen: &mut EqualityIndex<T>)
where
    T: Clone + PartialEq + Serialize,
{
    for value in source {
        if seen.insert(&value) {
            target.push(value);
        }
    }
}

fn stable_key<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("plugin manifest values serialize")
}
