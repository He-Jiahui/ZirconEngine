use std::collections::BTreeMap;

use crate::{
    plugin::PluginFeatureBundleManifest, plugin::PluginModuleKind, plugin::PluginPackageKind,
    plugin::PluginPackageManifest, plugin::RuntimePluginFeatureRegistrationReport,
    plugin::RuntimePluginRegistrationReport,
};

use super::{LoadedNativePlugin, NativePluginCandidate};

#[derive(Debug, Default)]
pub struct NativePluginLoadReport {
    pub discovered: Vec<NativePluginCandidate>,
    pub loaded: Vec<LoadedNativePlugin>,
    pub diagnostics: Vec<String>,
}

impl NativePluginLoadReport {
    pub fn has_failures(&self) -> bool {
        !self.diagnostics.is_empty()
    }

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

    pub fn runtime_plugin_registration_reports(&self) -> Vec<RuntimePluginRegistrationReport> {
        self.package_manifests()
            .into_iter()
            .filter(|manifest| {
                manifest.package_kind != PluginPackageKind::FeatureExtension
                    && has_runtime_module(manifest)
            })
            .map(|manifest| {
                let plugin_id = manifest.id.clone();
                let mut report = RuntimePluginRegistrationReport::from_native_package_manifest(
                    runtime_only_package_manifest(manifest),
                );
                report
                    .diagnostics
                    .extend(self.diagnostics_for_runtime_plugin(&plugin_id));
                report.diagnostics.sort();
                report.diagnostics.dedup();
                report
            })
            .collect()
    }

    pub fn runtime_plugin_feature_registration_reports(
        &self,
    ) -> Vec<RuntimePluginFeatureRegistrationReport> {
        self.package_manifests()
            .into_iter()
            .flat_map(|manifest| {
                let plugin_id = manifest.id.clone();
                runtime_feature_manifests(&manifest)
                    .into_iter()
                    .filter(has_runtime_feature_module)
                    .map(move |feature| {
                        let provider_package_id = if feature.owner_plugin_id == plugin_id {
                            None
                        } else {
                            Some(plugin_id.clone())
                        };
                        let mut report =
                            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                                feature,
                                provider_package_id,
                            );
                        report
                            .diagnostics
                            .extend(self.diagnostics_for_runtime_plugin(&plugin_id));
                        report.diagnostics.sort();
                        report.diagnostics.dedup();
                        report
                    })
            })
            .collect()
    }

    pub fn entry_diagnostics(&self) -> Vec<String> {
        sorted_deduped(
            self.loaded
                .iter()
                .flat_map(|plugin| {
                    plugin
                        .runtime_entry_report
                        .iter()
                        .chain(plugin.editor_entry_report.iter())
                        .flat_map(|report| {
                            report.diagnostics.iter().map(|message| {
                                format!("native plugin {}: {message}", plugin.plugin_id)
                            })
                        })
                })
                .collect(),
        )
    }

    pub fn descriptor_diagnostics(&self) -> Vec<String> {
        sorted_deduped(
            self.loaded
                .iter()
                .filter(|plugin| plugin.descriptor.is_none())
                .map(|plugin| {
                    format!(
                        "native plugin {} has no ABI descriptor attached",
                        plugin.plugin_id
                    )
                })
                .collect(),
        )
    }

    pub fn diagnostics_for_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_for_plugin_with_entry_kinds(
            plugin_id,
            &[PluginModuleKind::Runtime, PluginModuleKind::Editor],
        )
    }

    pub fn diagnostics_for_runtime_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_for_plugin_with_entry_kinds(plugin_id, &[PluginModuleKind::Runtime])
    }

    pub fn diagnostics_for_editor_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_for_plugin_with_entry_kinds(plugin_id, &[PluginModuleKind::Editor])
    }

    fn diagnostics_for_plugin_with_entry_kinds(
        &self,
        plugin_id: &str,
        module_kinds: &[PluginModuleKind],
    ) -> Vec<String> {
        let mut diagnostics = self
            .diagnostics
            .iter()
            .filter(|message| diagnostic_mentions_plugin(message, plugin_id))
            .cloned()
            .collect::<Vec<_>>();
        diagnostics.extend(
            self.loaded
                .iter()
                .filter(|plugin| plugin.plugin_id == plugin_id && plugin.descriptor.is_none())
                .map(|plugin| {
                    format!(
                        "native plugin {} has no ABI descriptor attached",
                        plugin.plugin_id
                    )
                }),
        );
        diagnostics.extend(
            self.loaded
                .iter()
                .filter(|plugin| plugin.plugin_id == plugin_id)
                .flat_map(|plugin| {
                    plugin
                        .runtime_entry_report
                        .iter()
                        .chain(plugin.editor_entry_report.iter())
                        .filter(|report| module_kinds.contains(&report.module_kind))
                        .flat_map(|report| {
                            report.diagnostics.iter().map(|message| {
                                format!("native plugin {}: {message}", plugin.plugin_id)
                            })
                        })
                }),
        );
        sorted_deduped(diagnostics)
    }
}

fn sorted_deduped(mut diagnostics: Vec<String>) -> Vec<String> {
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn has_runtime_module(manifest: &PluginPackageManifest) -> bool {
    manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Runtime)
}

fn has_runtime_feature_module(feature: &crate::plugin::PluginFeatureBundleManifest) -> bool {
    feature
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Runtime)
}

fn runtime_feature_manifests(manifest: &PluginPackageManifest) -> Vec<PluginFeatureBundleManifest> {
    let mut features = manifest.optional_features.clone();
    features.extend(manifest.feature_extensions.iter().cloned());
    features
}

fn runtime_only_package_manifest(mut manifest: PluginPackageManifest) -> PluginPackageManifest {
    manifest
        .modules
        .retain(|module| module.kind == PluginModuleKind::Runtime);
    manifest
}

fn diagnostic_mentions_plugin(message: &str, plugin_id: &str) -> bool {
    message.contains(&format!("native plugin {plugin_id} "))
        || message.contains(&format!("native plugin {plugin_id}:"))
}

fn merge_package_manifest(
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crate::{
        plugin::PluginFeatureBundleManifest, plugin::PluginFeatureDependency,
        plugin::PluginModuleKind, plugin::PluginModuleManifest, plugin::PluginPackageManifest,
    };

    use super::{merge_package_manifest, NativePluginCandidate, NativePluginLoadReport};

    #[test]
    fn native_manifest_merge_preserves_runtime_and_editor_entry_modules() {
        let mut manifests = BTreeMap::new();
        manifests.insert(
            "split_native".to_string(),
            PluginPackageManifest::new("split_native", "Split Native").with_runtime_module(
                PluginModuleManifest::runtime(
                    "split_native.runtime",
                    "zircon_plugin_split_native_runtime",
                ),
            ),
        );

        merge_package_manifest(
            &mut manifests,
            PluginPackageManifest::new("split_native", "Split Native").with_editor_module(
                PluginModuleManifest::editor(
                    "split_native.editor",
                    "zircon_plugin_split_native_editor",
                )
                .with_capabilities(["editor.extension.split_native".to_string()]),
            ),
        );

        let manifest = manifests.get("split_native").unwrap();
        assert!(manifest
            .modules
            .iter()
            .any(|module| module.kind == PluginModuleKind::Runtime));
        assert!(manifest
            .modules
            .iter()
            .any(|module| module.kind == PluginModuleKind::Editor));
        assert_eq!(manifest.modules.len(), 2);
    }

    #[test]
    fn native_manifest_merge_preserves_optional_feature_declarations() {
        let mut manifests = BTreeMap::new();
        manifests.insert(
            "split_native".to_string(),
            PluginPackageManifest::new("split_native", "Split Native").with_optional_feature(
                PluginFeatureBundleManifest::new(
                    "split_native.runtime_tools",
                    "Runtime Tools",
                    "split_native",
                )
                .with_dependency(PluginFeatureDependency::primary(
                    "split_native",
                    "runtime.plugin.split_native",
                )),
            ),
        );

        merge_package_manifest(
            &mut manifests,
            PluginPackageManifest::new("split_native", "Split Native").with_optional_feature(
                PluginFeatureBundleManifest::new(
                    "split_native.editor_tools",
                    "Editor Tools",
                    "split_native",
                )
                .with_dependency(PluginFeatureDependency::primary(
                    "split_native",
                    "runtime.plugin.split_native",
                )),
            ),
        );

        let manifest = manifests.get("split_native").unwrap();
        let feature_ids = manifest
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>();
        assert!(feature_ids.contains(&"split_native.runtime_tools"));
        assert!(feature_ids.contains(&"split_native.editor_tools"));
    }

    #[test]
    fn native_load_report_projects_optional_features_as_runtime_feature_registrations() {
        let feature = PluginFeatureBundleManifest::new(
            "split_native.runtime_tools",
            "Runtime Tools",
            "split_native",
        )
        .with_dependency(PluginFeatureDependency::primary(
            "split_native",
            "runtime.plugin.split_native",
        ))
        .with_capability("runtime.feature.split_native.runtime_tools")
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "split_native.runtime_tools.runtime",
                "zircon_plugin_split_native_runtime_tools_runtime",
            )
            .with_capabilities(["runtime.feature.split_native.runtime_tools"]),
        )
        .with_editor_module(PluginModuleManifest::editor(
            "split_native.runtime_tools.editor",
            "zircon_plugin_split_native_runtime_tools_editor",
        ));
        let report = NativePluginLoadReport {
            discovered: vec![NativePluginCandidate {
                plugin_id: "split_native".to_string(),
                package_manifest: PluginPackageManifest::new("split_native", "Split Native")
                    .with_runtime_module(
                        PluginModuleManifest::runtime(
                            "split_native.runtime",
                            "zircon_plugin_split_native_runtime",
                        )
                        .with_capabilities(["runtime.plugin.split_native"]),
                    )
                    .with_optional_feature(feature.clone()),
                manifest_path: PathBuf::from("split_native/plugin.toml"),
                library_path: PathBuf::from("split_native/native/libsplit_native.so"),
            }],
            loaded: Vec::new(),
            diagnostics: Vec::new(),
        };

        let feature_reports = report.runtime_plugin_feature_registration_reports();

        assert_eq!(feature_reports.len(), 1);
        assert_eq!(feature_reports[0].manifest, feature);
        assert_eq!(
            feature_reports[0]
                .project_selection
                .runtime_crate
                .as_deref(),
            Some("zircon_plugin_split_native_runtime_tools_runtime")
        );
        assert_eq!(feature_reports[0].extensions.modules().len(), 1);
        assert_eq!(
            feature_reports[0].extensions.modules()[0].name,
            "split_native.runtime_tools.runtime"
        );
    }

    #[test]
    fn native_load_report_projects_feature_extension_packages_as_runtime_feature_registrations() {
        let feature = PluginFeatureBundleManifest::new(
            "sound.timeline_animation_track",
            "Sound Timeline Animation Track",
            "sound",
        )
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
        .with_capability("runtime.feature.sound.timeline_animation_track")
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "sound.timeline_animation_track.runtime",
                "zircon_plugin_sound_timeline_animation_runtime",
            )
            .with_capabilities(["runtime.feature.sound.timeline_animation_track"]),
        );
        let report = NativePluginLoadReport {
            discovered: vec![NativePluginCandidate {
                plugin_id: "sound_timeline_animation_track".to_string(),
                package_manifest: PluginPackageManifest::new(
                    "sound_timeline_animation_track",
                    "Sound Timeline Animation Track Provider",
                )
                .as_feature_extension()
                .with_feature_extension(feature.clone()),
                manifest_path: PathBuf::from("sound_timeline_animation_track/plugin.toml"),
                library_path: PathBuf::from(
                    "sound_timeline_animation_track/native/libsound_timeline_animation_track.so",
                ),
            }],
            loaded: Vec::new(),
            diagnostics: Vec::new(),
        };

        assert!(report.runtime_plugin_registration_reports().is_empty());

        let feature_reports = report.runtime_plugin_feature_registration_reports();

        assert_eq!(feature_reports.len(), 1);
        assert_eq!(feature_reports[0].manifest, feature);
        assert_eq!(
            feature_reports[0].provider_package_id.as_deref(),
            Some("sound_timeline_animation_track")
        );
        assert_eq!(
            feature_reports[0]
                .project_selection
                .provider_package_id
                .as_deref(),
            Some("sound_timeline_animation_track")
        );
    }
}
