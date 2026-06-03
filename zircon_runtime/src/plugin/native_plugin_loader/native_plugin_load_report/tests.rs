use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleKind, PluginModuleManifest,
    PluginPackageManifest,
};

use super::{manifests::merge_package_manifest, NativePluginLoadReport};
use crate::plugin::native_plugin_loader::NativePluginCandidate;

#[test]
fn native_manifest_merge_preserves_runtime_and_editor_entry_modules() {
    let mut manifests = BTreeMap::new();
    manifests.insert(
        "split_native".to_string(),
        PluginPackageManifest::new("split_native", "Split Native")
            .with_capability("runtime.plugin.split_native")
            .with_runtime_module(PluginModuleManifest::runtime(
                "split_native.runtime",
                "zircon_plugin_split_native_runtime",
            )),
    );

    merge_package_manifest(
        &mut manifests,
        PluginPackageManifest::new("split_native", "Split Native")
            .with_capability("editor.extension.split_native")
            .with_editor_module(
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
    assert_eq!(
        manifest.capabilities,
        vec![
            "runtime.plugin.split_native".to_string(),
            "editor.extension.split_native".to_string()
        ]
    );
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
                .with_capability("runtime.plugin.split_native")
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
