use super::*;

#[test]
fn source_template_preserves_builtin_catalog_target_modes_after_manifest_completion() {
    let mut manifest = ProjectManifest::new(
        "Catalog Completion Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins =
        RuntimePluginCatalog::builtin().complete_project_manifest(&ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::VirtualGeometry,
                true,
                false,
            )],
        });
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let virtual_geometry = manifest
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == "virtual_geometry")
        .expect("catalog completion should preserve the virtual geometry selection");

    assert_eq!(
        virtual_geometry.target_modes,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert!(plugin_source.contains(
        "target_modes: vec![RuntimeTargetMode::ClientRuntime, RuntimeTargetMode::EditorHost]"
    ));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_virtual_geometry_runtime::plugin_registration)"
    ));
}

#[test]
fn source_template_completes_builtin_catalog_selection_before_projection() {
    let mut manifest = ProjectManifest::new(
        "Implicit Catalog Completion Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            false,
        )],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_virtual_geometry_runtime".to_string()));
    assert!(plugin_source.contains(
        "target_modes: vec![RuntimeTargetMode::ClientRuntime, RuntimeTargetMode::EditorHost]"
    ));
    assert!(plugin_source
        .contains("runtime_crate: Some(\"zircon_plugin_virtual_geometry_runtime\".to_string())"));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_virtual_geometry_runtime::plugin_registration)"
    ));
    assert!(cargo_manifest.contains("zircon_plugin_virtual_geometry_runtime"));
}

#[test]
fn source_template_links_rendering_default_owner_features() {
    let mut manifest = ProjectManifest::new(
        "Rendering Default Feature Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, true)
                .with_runtime_crate("zircon_plugin_sound_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Rendering, true, true),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    for crate_name in [
        "zircon_plugin_rendering_runtime",
        "zircon_plugin_rendering_post_process_runtime",
        "zircon_plugin_rendering_ssao_runtime",
        "zircon_plugin_rendering_reflection_probes_runtime",
        "zircon_plugin_rendering_baked_lighting_runtime",
    ] {
        assert!(
            plan.linked_runtime_crates.contains(&crate_name.to_string()),
            "{crate_name} should be linked when rendering is selected"
        );
        assert!(cargo_manifest.contains(crate_name), "{cargo_manifest}");
    }
    for call in [
        "ExportRuntimePluginFeatureRegistrationProvider::new(zircon_plugin_rendering_post_process_runtime::plugin_feature_registration)",
        "ExportRuntimePluginFeatureRegistrationProvider::new(zircon_plugin_rendering_ssao_runtime::plugin_feature_registration)",
        "ExportRuntimePluginFeatureRegistrationProvider::new(zircon_plugin_rendering_reflection_probes_runtime::plugin_feature_registration)",
        "ExportRuntimePluginFeatureRegistrationProvider::new(zircon_plugin_rendering_baked_lighting_runtime::plugin_feature_registration)",
    ] {
        assert!(plugin_source.contains(call), "{plugin_source}");
    }
    for opt_in_crate in [
        "zircon_plugin_rendering_decals_runtime",
        "zircon_plugin_rendering_ray_tracing_policy_runtime",
        "zircon_plugin_rendering_shader_graph_runtime",
        "zircon_plugin_rendering_vfx_graph_runtime",
    ] {
        assert!(!plan
            .linked_runtime_crates
            .contains(&opt_in_crate.to_string()));
        assert!(!cargo_manifest.contains(opt_in_crate));
        assert!(!plugin_source.contains(&format!("{opt_in_crate}::plugin_feature_registration")));
    }
    assert!(availability_contains(
        &plan.runtime_plugin_availability.linked,
        "rendering"
    ));
    assert!(
        plan.effective_fatal_diagnostics().is_empty(),
        "{:?}",
        plan.effective_fatal_diagnostics()
    );
}

#[test]
fn library_embed_links_advanced_runtime_render_plugins() {
    let mut manifest = ProjectManifest::new(
        "Advanced Render Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::VirtualGeometry, true, false)
                .with_runtime_crate("zircon_plugin_virtual_geometry_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::HybridGi, true, false)
                .with_runtime_crate("zircon_plugin_hybrid_gi_runtime"),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_virtual_geometry_runtime".to_string()));
    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_hybrid_gi_runtime".to_string()));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_virtual_geometry_runtime::plugin_registration)"
    ));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_hybrid_gi_runtime::plugin_registration)"
    ));
}

#[test]
fn source_template_with_native_dynamic_merges_native_loader_reports() {
    let mut manifest = ProjectManifest::new(
        "Hybrid Native Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_runtime_crate("zircon_plugin_sound_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::VirtualGeometry, true, true)
                .with_runtime_crate("zircon_plugin_virtual_geometry_runtime")
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let main_source = generated_file(&plan, "src/main.rs");
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(main_source
        .contains("zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root"));
    assert!(main_source.contains("zircon_app::discover_export_root()?"));
    assert!(main_source.contains("zircon_plugins::export_runtime_bootstrap_config()"));
    assert!(!main_source.contains("NativePluginLoader"));
    assert!(!main_source.contains("load_runtime_from_load_manifest"));
    assert!(!main_source.contains("registrations.extend"));
    assert!(!main_source.contains("feature_registrations.extend"));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_sound_runtime::plugin_registration)"
    ));
    assert!(!plugin_source.contains("zircon_plugin_virtual_geometry_runtime::plugin_registration"));
}
