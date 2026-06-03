use std::collections::BTreeMap;

use crate::plugin::{PluginPackageKind, PluginPackageManifest};

use super::NativePluginLoadReport;

impl NativePluginLoadReport {
    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        let mut manifests = self
            .discovered
            .iter()
            .map(|candidate| {
                (
                    candidate.package_manifest.id.clone(),
                    candidate.package_manifest.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for plugin in &self.loaded {
            if let Some(manifest) = plugin
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.package_manifest.clone())
            {
                merge_package_manifest(&mut manifests, manifest);
            }
            if let Some(manifest) = plugin
                .runtime_entry_report
                .as_ref()
                .and_then(|report| report.package_manifest.clone())
            {
                merge_package_manifest(&mut manifests, manifest);
            }
            if let Some(manifest) = plugin
                .editor_entry_report
                .as_ref()
                .and_then(|report| report.package_manifest.clone())
            {
                merge_package_manifest(&mut manifests, manifest);
            }
        }
        manifests.into_values().collect()
    }
}

pub(super) fn merge_package_manifest(
    manifests: &mut BTreeMap<String, PluginPackageManifest>,
    manifest: PluginPackageManifest,
) {
    let Some(existing) = manifests.get_mut(&manifest.id) else {
        manifests.insert(manifest.id.clone(), manifest);
        return;
    };

    if !manifest.version.is_empty() {
        existing.version = manifest.version;
    }
    if !manifest.display_name.is_empty() {
        existing.display_name = manifest.display_name;
    }
    if !manifest.description.is_empty() {
        existing.description = manifest.description;
    }
    if manifest.package_kind != PluginPackageKind::Standard {
        existing.package_kind = manifest.package_kind;
    }
    push_unique(&mut existing.capabilities, manifest.capabilities);
    push_unique(&mut existing.modules, manifest.modules);
    push_unique(&mut existing.components, manifest.components);
    push_unique(&mut existing.ui_components, manifest.ui_components);
    push_unique(&mut existing.asset_importers, manifest.asset_importers);
    push_unique(&mut existing.optional_features, manifest.optional_features);
    push_unique(
        &mut existing.feature_extensions,
        manifest.feature_extensions,
    );
    push_unique(&mut existing.default_packaging, manifest.default_packaging);
}

fn push_unique<T: PartialEq>(target: &mut Vec<T>, source: Vec<T>) {
    for value in source {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}
