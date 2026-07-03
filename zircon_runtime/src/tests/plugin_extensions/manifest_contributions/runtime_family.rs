use super::*;

#[test]
fn sound_plugin_manifest_matches_catalog_beta_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "sound");
    let manifest_source = fs::read_to_string(plugins_root.join("sound").join("plugin.toml"))
        .expect("sound plugin manifest source");
    let encoded = toml::to_string(&manifest).expect("sound plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("sound plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("sound plugin should declare a runtime module");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id() == RuntimePluginId::Sound)
        .expect("sound plugin should be in the runtime catalog");
    let projected_manifest = descriptor.package_manifest();
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.sound".to_string()];

    assert_eq!(decoded, manifest);
    assert!(
        manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
        "sound plugin should explicitly declare SDK API version"
    );
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.sound"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_audio/src/lib.rs".to_string())
    }));
    assert_eq!(descriptor.category(), manifest.category);
    assert_eq!(descriptor.maturity(), manifest.maturity);
    assert_eq!(
        descriptor.target_modes(),
        manifest.supported_targets.as_slice()
    );
    assert_eq!(descriptor.capabilities(), manifest.capabilities.as_slice());
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.plugin.sound"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_audio/src/lib.rs".to_string())
    }));
    assert_eq!(projected_manifest.category, manifest.category);
    assert_eq!(projected_manifest.maturity, manifest.maturity);
    assert_eq!(
        projected_manifest.supported_targets,
        manifest.supported_targets
    );
    assert_eq!(projected_manifest.capabilities, manifest.capabilities);
}

#[test]
fn animation_plugin_toml_matches_catalog_beta_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "animation");
    let encoded = toml::to_string(&manifest).expect("animation plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("animation plugin manifest roundtrip");

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.animation"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_animation/src/lib.rs".to_string())
    }));
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.feature.animation.timeline_event_track"
            && status.status == CapabilityStatus::Partial
    }));

    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id() == "animation")
        .expect("animation catalog entry");
    assert_eq!(descriptor.category(), "runtime");
    assert_eq!(descriptor.maturity(), crate::plugin::PluginMaturity::Beta);
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.plugin.animation"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_animation/src/lib.rs".to_string())
    }));
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.feature.animation.timeline_event_track"
            && status.status == CapabilityStatus::Partial
    }));
}

#[test]
fn navigation_plugin_toml_matches_catalog_beta_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "navigation");
    let encoded = toml::to_string(&manifest).expect("navigation plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("navigation plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("navigation plugin should declare a runtime module");
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.navigation".to_string()];

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.navigation"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some(
                    "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
                )
    }));

    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id() == "navigation")
        .expect("navigation catalog entry");
    assert_eq!(descriptor.category(), "runtime");
    assert_eq!(descriptor.maturity(), crate::plugin::PluginMaturity::Beta);
    assert_eq!(
        descriptor.target_modes(),
        manifest.supported_targets.as_slice()
    );
    assert_eq!(descriptor.capabilities(), manifest.capabilities.as_slice());
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.plugin.navigation"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some(
                    "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
                )
    }));
}

#[test]
fn particles_plugin_toml_matches_catalog_optional_feature_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "particles");
    let manifest_source = fs::read_to_string(plugins_root.join("particles").join("plugin.toml"))
        .expect("particles plugin manifest source");
    let encoded = toml::to_string(&manifest).expect("particles plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("particles plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("particles plugin should declare a runtime module");
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.particles".to_string()];

    assert_eq!(decoded, manifest);
    assert!(
        manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
        "particles plugin should explicitly declare SDK API version"
    );
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(
        manifest.maturity,
        crate::plugin::PluginMaturity::Experimental
    );
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.particles"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some("Advanced optional VFX capability; not a Bevy default parity blocker.")
    }));
    assert_particles_optional_features(&manifest.optional_features);

    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id() == "particles")
        .expect("particles catalog entry");
    assert_eq!(descriptor.category(), "runtime");
    assert_eq!(
        descriptor.maturity(),
        crate::plugin::PluginMaturity::Experimental
    );
    assert_eq!(
        descriptor.target_modes(),
        manifest.supported_targets.as_slice()
    );
    assert_eq!(descriptor.capabilities(), manifest.capabilities.as_slice());
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.plugin.particles"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some("Advanced optional VFX capability; not a Bevy default parity blocker.")
    }));
    assert_particles_optional_features(descriptor.optional_features());
}

#[test]
fn texture_plugin_manifest_matches_catalog_stable_complete_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "texture");
    let manifest_source = fs::read_to_string(plugins_root.join("texture").join("plugin.toml"))
        .expect("texture plugin manifest source");
    let encoded = toml::to_string(&manifest).expect("texture plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("texture plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("texture plugin should declare a runtime module");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id() == RuntimePluginId::Texture)
        .expect("texture plugin should be in the runtime catalog");
    let projected_manifest = descriptor.package_manifest();
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.texture".to_string()];

    assert_eq!(decoded, manifest);
    assert!(
        manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
        "texture plugin should explicitly declare SDK API version"
    );
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Stable);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.texture" && status.status == CapabilityStatus::Complete
    }));
    assert_eq!(descriptor.category(), manifest.category);
    assert_eq!(descriptor.maturity(), manifest.maturity);
    assert_eq!(
        descriptor.target_modes(),
        manifest.supported_targets.as_slice()
    );
    assert_eq!(descriptor.capabilities(), manifest.capabilities.as_slice());
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.plugin.texture" && status.status == CapabilityStatus::Complete
    }));
    assert_eq!(projected_manifest.category, manifest.category);
    assert_eq!(projected_manifest.maturity, manifest.maturity);
    assert_eq!(
        projected_manifest.supported_targets,
        manifest.supported_targets
    );
    assert_eq!(projected_manifest.capabilities, manifest.capabilities);
}

#[test]
fn runtime_experimental_plugin_toml_matches_catalog_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    for (id, capabilities) in [
        (
            "ai",
            vec![
                "runtime.plugin.ai",
                "runtime.feature.ai.behavior_tree",
                "runtime.feature.ai.blackboard",
                "runtime.feature.ai.perception",
            ],
        ),
        (
            "physics",
            vec![
                "runtime.plugin.physics",
                "runtime.capability.physics.raycast",
                "runtime.capability.physics.overlap",
                "runtime.capability.physics.shape_cast",
                "runtime.capability.physics.trigger_events",
                "runtime.capability.physics.constraints",
                "runtime.capability.physics.skeletal_joints",
            ],
        ),
        (
            "zr_vm_language",
            vec![
                "runtime.plugin.zr_vm_language",
                "runtime.script.backend.zr_vm_project",
            ],
        ),
    ] {
        let manifest = read_plugin_manifest(&plugins_root, id);
        let encoded = toml::to_string(&manifest).expect("runtime plugin manifest toml");
        let decoded: PluginPackageManifest =
            toml::from_str(&encoded).expect("runtime plugin manifest roundtrip");

        assert_eq!(decoded, manifest);
        assert_eq!(manifest.category, "runtime");
        assert_eq!(
            manifest.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
        assert_runtime_partial_capability_statuses(&manifest.capability_statuses, &capabilities);

        let descriptor = RuntimePluginDescriptor::builtin_catalog()
            .into_iter()
            .find(|descriptor| descriptor.package_id() == id)
            .expect("runtime plugin catalog entry");
        assert_eq!(descriptor.category(), "runtime");
        assert_eq!(
            descriptor.maturity(),
            crate::plugin::PluginMaturity::Experimental
        );
        assert_runtime_partial_capability_statuses(descriptor.capability_statuses(), &capabilities);
    }
}
