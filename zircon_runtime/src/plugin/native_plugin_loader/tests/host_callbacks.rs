use crate::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleKind, PluginModuleManifest,
    PluginPackageManifest,
};

use super::{
    granted_capabilities_for_entry, native_capability_list_contains, NativePluginDescriptor,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
};

#[test]
fn native_host_capability_probe_streams_delimited_tokens_without_owned_list_projection() {
    assert!(native_capability_list_contains(
        "runtime.physics, runtime.render;runtime.audio\nruntime.net",
        "runtime.audio"
    ));
    assert!(!native_capability_list_contains(
        "runtime.physics,runtime.rendering",
        "runtime.render"
    ));
    assert!(!native_capability_list_contains(" , ;\n", ""));

    let source = include_str!("../host_callbacks.rs")
        .split_once("unsafe fn native_host_has_capability_v3_inner")
        .expect("native host capability callback should exist")
        .1
        .split_once("pub(super) unsafe extern \"C\" fn native_host_log_v3")
        .expect("native host log callback should follow capability callback")
        .0;
    assert!(source.contains("CStr::from_ptr(granted_capabilities)"));
    assert!(source.contains("native_capability_list_contains"));
    assert!(!source.contains("read_optional_c_string"));
    assert!(!source.contains("parse_native_string_list"));

    let grants = include_str!("../host_callbacks.rs")
        .split_once("pub(super) fn granted_capabilities_for_entry")
        .expect("entry capability grant projection should exist")
        .1
        .split_once("fn module_capabilities")
        .expect("module capability iterator should follow grant projection")
        .0;
    assert!(grants.contains("collect::<HashSet<_>>()"));
    assert!(grants.contains("requested.contains(capability)"));
    assert!(grants.contains("granted_capabilities.insert(capability.to_string())"));
    assert!(grants.contains("manifest.feature_extensions"));
    assert!(grants.contains("feature.dependencies"));
    assert!(!grants.contains("requested.iter().any"));
    assert!(!grants.contains("granted.iter().any"));
}

#[test]
fn feature_extension_runtime_entry_grants_module_and_dependency_capabilities() {
    let manifest = PluginPackageManifest::new("sound_feature", "Sound Feature")
        .as_feature_extension()
        .with_feature_extension(
            PluginFeatureBundleManifest::new("sound.feature", "Sound Feature", "sound")
                .with_dependency(PluginFeatureDependency::primary(
                    "sound",
                    "runtime.plugin.sound",
                ))
                .with_dependency(PluginFeatureDependency::required(
                    "physics",
                    "runtime.plugin.physics.unrequested",
                ))
                .with_runtime_module(
                    PluginModuleManifest::runtime("sound.feature.runtime", "sound_feature")
                        .with_capabilities([
                            "runtime.feature.sound.feature",
                            "runtime.feature.sound.unrequested",
                        ]),
                ),
        );
    let descriptor = NativePluginDescriptor {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        plugin_id: "sound_feature".to_string(),
        package_manifest: Some(manifest),
        runtime_entry_name: Some("sound_feature_runtime_entry_v3".to_string()),
        editor_entry_name: None,
        requested_capabilities: vec![
            "runtime.plugin.sound".to_string(),
            "runtime.feature.sound.feature".to_string(),
        ],
    };

    assert_eq!(
        granted_capabilities_for_entry(&descriptor, PluginModuleKind::Runtime),
        [
            "runtime.feature.sound.feature".to_string(),
            "runtime.plugin.sound".to_string(),
        ]
    );
    assert!(granted_capabilities_for_entry(&descriptor, PluginModuleKind::Editor).is_empty());
}
