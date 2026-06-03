use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::{AssetUri, ProjectManifest};
use crate::{
    plugin::ExportBuildPlan, plugin::ExportPackagingStrategy, plugin::ExportProfile,
    plugin::ExportTargetPlatform, plugin::ProjectPluginManifest, plugin::ProjectPluginSelection,
    plugin::RuntimePluginCatalog, RuntimePluginId, RuntimeTargetMode,
};

#[test]
fn source_template_generates_linked_external_runtime_plugin_registration_calls() {
    let mut manifest = ProjectManifest::new(
        "Plugin Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            true,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime")],
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
    let main_source = generated_file(&plan, "src/main.rs");

    assert!(plugin_source
        .contains("pub fn runtime_plugin_registrations() -> Vec<RuntimePluginRegistrationReport>"));
    assert!(plugin_source.contains("zircon_plugin_sound_runtime::plugin_registration()"));
    assert!(main_source
        .contains("EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations"));
    assert!(main_source.contains("zircon_plugins::runtime_plugin_registrations()"));
    assert!(plan
        .runtime_plugin_availability
        .linked
        .iter()
        .any(|entry| entry.id == "sound"));
}

#[test]
fn export_plan_treats_missing_required_profile_providers_as_fatal() {
    let mut manifest = ProjectManifest::new(
        "Missing Required Provider Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            true,
        )],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::SourceTemplate])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();

    assert!(availability_contains(
        &plan.runtime_plugin_availability.externalized_missing,
        "sound"
    ));
    assert!(availability_contains(
        &plan.runtime_plugin_availability.missing_required,
        "sound"
    ));
    assert!(plan.has_fatal_diagnostics());
    assert!(plan
        .effective_fatal_diagnostics()
        .iter()
        .any(|diagnostic| diagnostic
            .contains("required runtime plugin sound is unavailable for export profile client")));

    let output_root = temp_dir("zircon_missing_required_provider_export");
    let report = plan.materialize(&output_root).unwrap();
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("required runtime plugin sound is unavailable for export profile client")));
}

#[test]
fn source_template_links_physics_as_external_runtime_plugin() {
    let mut manifest = ProjectManifest::new(
        "Builtin Domain Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Physics,
            true,
            true,
        )
        .with_runtime_crate("zircon_plugin_physics_runtime")],
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

    assert_eq!(
        plan.linked_runtime_crates,
        vec!["zircon_plugin_physics_runtime".to_string()]
    );
    assert!(plugin_source.contains("id: \"physics\".to_string()"));
    assert!(plugin_source.contains("zircon_plugin_physics_runtime::plugin_registration()"));
    assert!(cargo_manifest.contains("zircon_plugin_physics_runtime"));
}

#[test]
fn export_plan_treats_animation_as_external_native_dynamic_package() {
    let mut manifest = ProjectManifest::new(
        "Builtin Native Domain Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Animation,
            true,
            true,
        )
        .with_runtime_crate("zircon_plugin_animation_runtime")
        .with_packaging(ExportPackagingStrategy::NativeDynamic)],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "native-only",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "native-only").unwrap();

    assert!(plan.linked_runtime_crates.is_empty());
    assert_eq!(plan.native_dynamic_packages, vec!["animation".to_string()]);
    assert!(availability_contains(
        &plan.runtime_plugin_availability.native_dynamic,
        "animation"
    ));
    assert!(generated_file(&plan, "plugins/native_plugins.toml").contains("id = \"animation\""));
    assert!(plan
        .generated_files
        .iter()
        .all(|file| file.path != "Cargo.toml"));
    assert!(plan.diagnostics.is_empty());
}

#[test]
fn source_template_keeps_editor_only_plugins_out_of_runtime_registrations() {
    let mut manifest = ProjectManifest::new(
        "Editor Only Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: "runtime_diagnostics".to_string(),
            enabled: true,
            required: false,
            target_modes: vec![RuntimeTargetMode::EditorHost],
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: Some("zircon_plugin_runtime_diagnostics_editor".to_string()),
            features: Vec::new(),
        }],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "editor",
        RuntimeTargetMode::EditorHost,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "editor").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan.linked_runtime_crates.is_empty());
    assert!(!plugin_source.contains("runtime_diagnostics_runtime::plugin_registration()"));
    assert!(!plugin_source.contains("runtime_diagnostics_editor::plugin_registration()"));
    assert!(!cargo_manifest.contains("zircon_plugin_runtime_diagnostics_editor"));
}

#[test]
fn source_template_links_runtime_backed_authoring_and_excludes_editor_only_authoring() {
    let mut manifest = ProjectManifest::new(
        "Authoring Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins =
        RuntimePluginCatalog::builtin().complete_project_manifest(&ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::Terrain, true, false),
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::Tilemap2d, true, false),
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::PrefabTools, true, false),
                ProjectPluginSelection {
                    id: "material_editor".to_string(),
                    enabled: true,
                    required: false,
                    target_modes: vec![RuntimeTargetMode::EditorHost],
                    packaging: ExportPackagingStrategy::LibraryEmbed,
                    runtime_crate: None,
                    editor_crate: Some("zircon_plugin_material_editor_editor".to_string()),
                    features: Vec::new(),
                },
                ProjectPluginSelection {
                    id: "timeline_sequence".to_string(),
                    enabled: true,
                    required: false,
                    target_modes: vec![RuntimeTargetMode::EditorHost],
                    packaging: ExportPackagingStrategy::LibraryEmbed,
                    runtime_crate: None,
                    editor_crate: Some("zircon_plugin_timeline_sequence_editor".to_string()),
                    features: Vec::new(),
                },
                ProjectPluginSelection {
                    id: "animation_graph".to_string(),
                    enabled: true,
                    required: false,
                    target_modes: vec![RuntimeTargetMode::EditorHost],
                    packaging: ExportPackagingStrategy::LibraryEmbed,
                    runtime_crate: None,
                    editor_crate: Some("zircon_plugin_animation_graph_editor".to_string()),
                    features: Vec::new(),
                },
            ],
        });
    manifest.export_profiles = vec![ExportProfile::new(
        "editor",
        RuntimeTargetMode::EditorHost,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "editor").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_terrain_runtime".to_string()));
    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_tilemap_2d_runtime".to_string()));
    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_prefab_tools_runtime".to_string()));
    assert!(!plan
        .linked_runtime_crates
        .iter()
        .any(|crate_name| crate_name.contains("material_editor")
            || crate_name.contains("timeline_sequence")
            || crate_name.contains("animation_graph")));
    assert!(plugin_source.contains("zircon_plugin_terrain_runtime::plugin_registration()"));
    assert!(plugin_source.contains("zircon_plugin_tilemap_2d_runtime::plugin_registration()"));
    assert!(plugin_source.contains("zircon_plugin_prefab_tools_runtime::plugin_registration()"));
    assert!(!plugin_source.contains("zircon_plugin_material_editor_editor::plugin_registration()"));
    assert!(
        !plugin_source.contains("zircon_plugin_timeline_sequence_editor::plugin_registration()")
    );
    assert!(!plugin_source.contains("zircon_plugin_animation_graph_editor::plugin_registration()"));
    assert!(cargo_manifest.contains("zircon_plugin_terrain_runtime"));
    assert!(cargo_manifest.contains("zircon_plugin_tilemap_2d_runtime"));
    assert!(cargo_manifest.contains("zircon_plugin_prefab_tools_runtime"));
    assert!(!cargo_manifest.contains("zircon_plugin_material_editor_editor"));
    assert!(!cargo_manifest.contains("zircon_plugin_timeline_sequence_editor"));
    assert!(!cargo_manifest.contains("zircon_plugin_animation_graph_editor"));
}

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
    assert!(plugin_source.contains("zircon_plugin_virtual_geometry_runtime::plugin_registration()"));
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
    assert!(plugin_source.contains("zircon_plugin_virtual_geometry_runtime::plugin_registration()"));
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
        "zircon_plugin_rendering_post_process_runtime::plugin_feature_registration()",
        "zircon_plugin_rendering_ssao_runtime::plugin_feature_registration()",
        "zircon_plugin_rendering_reflection_probes_runtime::plugin_feature_registration()",
        "zircon_plugin_rendering_baked_lighting_runtime::plugin_feature_registration()",
    ] {
        assert!(plugin_source.contains(call), "{plugin_source}");
    }
    for opt_in_crate in [
        "zircon_plugin_rendering_decals_runtime",
        "zircon_plugin_rendering_ray_tracing_policy_runtime",
        "zircon_plugin_rendering_shader_graph_runtime",
        "zircon_plugin_rendering_vfx_graph_runtime",
    ] {
        assert!(!plan.linked_runtime_crates.contains(&opt_in_crate.to_string()));
        assert!(!cargo_manifest.contains(opt_in_crate));
        assert!(!plugin_source.contains(&format!(
            "{opt_in_crate}::plugin_feature_registration()"
        )));
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
    assert!(plugin_source.contains("zircon_plugin_virtual_geometry_runtime::plugin_registration()"));
    assert!(plugin_source.contains("zircon_plugin_hybrid_gi_runtime::plugin_registration()"));
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

    assert!(
        main_source.contains("NativePluginLoader.load_runtime_from_load_manifest(export_root()?)")
    );
    assert!(main_source
        .contains("registrations.extend(native_report.runtime_plugin_registration_reports())"));
    assert!(main_source.contains(
        "feature_registrations.extend(native_report.runtime_plugin_feature_registration_reports())"
    ));
    assert!(plugin_source.contains("zircon_plugin_sound_runtime::plugin_registration()"));
    assert!(
        !plugin_source.contains("zircon_plugin_virtual_geometry_runtime::plugin_registration()")
    );
}

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn availability_contains(
    entries: &[crate::plugin::RuntimePluginAvailabilityEntry],
    plugin_id: &str,
) -> bool {
    entries.iter().any(|entry| entry.id == plugin_id)
}

fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{stamp}"))
}
