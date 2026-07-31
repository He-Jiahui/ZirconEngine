use std::collections::{BTreeMap, HashMap};

use crate::asset::AssetImporterDescriptor;
use crate::core::framework::project::ExportPackagingStrategy;
use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::plugin::{
    PluginFeatureBundleManifest, PluginModuleManifest, PluginPackageKind, PluginPackageManifest,
    UiComponentDescriptor,
};
use serde::Serialize;

use super::{projection::ProjectionBuildStats, NativePluginLoadReport};

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
        builder.insert_discovered(candidate.package_manifest.clone());
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
            builder.merge(manifest);
        }
    }
    builder.finish()
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
    manifests: BTreeMap<String, ManifestAccumulator>,
}

impl ManifestProjectionBuilder {
    fn insert_discovered(&mut self, manifest: PluginPackageManifest) {
        self.manifests
            .insert(manifest.id.clone(), ManifestAccumulator::new(manifest));
    }

    fn merge(&mut self, manifest: PluginPackageManifest) {
        let Some(existing) = self.manifests.get_mut(&manifest.id) else {
            self.insert_discovered(manifest);
            return;
        };
        existing.merge(manifest);
    }

    fn finish(self) -> Vec<PluginPackageManifest> {
        self.manifests
            .into_values()
            .map(ManifestAccumulator::finish)
            .collect()
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
