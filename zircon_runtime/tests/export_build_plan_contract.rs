use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::{
    ExportBuildPlan, ExportPackagingStrategy, ExportProfile, ExportTargetPlatform,
    ProjectPluginManifest, ProjectPluginSelection, RuntimePluginId, RuntimeTargetMode,
};

#[test]
fn native_dynamic_strategy_only_loads_native_packaged_selections() {
    let mut manifest = ProjectManifest::new(
        "Hybrid Export Packaging Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Physics, true, false)
                .with_runtime_crate("zircon_plugin_physics_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::VirtualGeometry, true, false)
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
    let native_manifest = generated_file(&plan, "plugins/native_plugins.toml");

    assert_eq!(
        plan.native_dynamic_packages,
        vec!["virtual_geometry".to_string()]
    );
    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_physics_runtime".to_string()));
    assert!(native_manifest.contains("id = \"virtual_geometry\""));
    assert!(!native_manifest.contains("id = \"physics\""));
}

#[test]
fn source_template_without_library_embed_serializes_selection_without_linking_crate() {
    let mut manifest = ProjectManifest::new(
        "Source Template Only Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Physics,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_physics_runtime")],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::SourceTemplate])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let cargo_manifest = generated_file(&plan, "Cargo.toml");
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.linked_runtime_crates.is_empty());
    assert!(plugin_source.contains("id: \"physics\".to_string()"));
    assert!(!plugin_source.contains("zircon_plugin_physics_runtime::plugin_registration()"));
    assert!(!cargo_manifest.contains("zircon_plugin_physics_runtime"));
}

#[test]
fn native_dynamic_selection_requires_native_dynamic_profile_strategy() {
    let mut manifest = ProjectManifest::new(
        "Native Dynamic Strategy Gate Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::VirtualGeometry,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_virtual_geometry_runtime")
        .with_packaging(ExportPackagingStrategy::NativeDynamic)],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
    ])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let main_source = generated_file(&plan, "src/main.rs");

    assert!(plan.native_dynamic_packages.is_empty());
    assert!(missing_generated_file(&plan, "plugins/native_plugins.toml"));
    assert!(!main_source.contains("NativePluginLoader.load_from_load_manifest"));
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("virtual_geometry") && diagnostic.contains("NativeDynamic")
    }));
}

#[test]
fn library_embed_selection_without_source_or_library_profile_reports_unexported_plugin() {
    let mut manifest = ProjectManifest::new(
        "Library Embed Strategy Gate Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Physics,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_physics_runtime")],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "native-only",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "native-only").unwrap();

    assert!(plan.linked_runtime_crates.is_empty());
    assert!(plan.native_dynamic_packages.is_empty());
    assert!(plan.generated_files.is_empty());
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("physics") && diagnostic.contains("LibraryEmbed")
    }));
}

#[test]
fn materialize_report_carries_planner_diagnostics_even_without_generated_files() {
    let output_root = temp_dir("zircon_materialize_plan_diagnostics_output");
    let mut manifest = ProjectManifest::new(
        "Materialize Diagnostics Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Physics,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_physics_runtime")],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "native-only",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "native-only").unwrap();
    let report = plan.materialize(&output_root).unwrap();

    assert!(report.generated_files.is_empty());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("physics") && diagnostic.contains("LibraryEmbed")
    }));

    let _ = std::fs::remove_dir_all(output_root);
}

#[test]
fn library_embed_deduplicates_runtime_crate_dependencies_and_registration_calls() {
    let mut manifest = ProjectManifest::new(
        "Duplicate Runtime Crate Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Physics, true, false)
                .with_runtime_crate("zircon_plugin_physics_runtime"),
            ProjectPluginSelection {
                id: "physics_debug_alias".to_string(),
                enabled: true,
                required: false,
                target_modes: vec![RuntimeTargetMode::ClientRuntime],
                packaging: ExportPackagingStrategy::LibraryEmbed,
                runtime_crate: Some("zircon_plugin_physics_runtime".to_string()),
                editor_crate: None,
            },
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
    ])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let cargo_manifest = generated_file(&plan, "Cargo.toml");
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert_eq!(
        plan.linked_runtime_crates,
        vec!["zircon_plugin_physics_runtime".to_string()]
    );
    assert_eq!(
        occurrences(cargo_manifest, "zircon_plugin_physics_runtime ="),
        1
    );
    assert_eq!(
        occurrences(
            plugin_source,
            "zircon_plugin_physics_runtime::plugin_registration()"
        ),
        1
    );
}

#[test]
fn native_dynamic_deduplicates_loader_manifest_packages_by_plugin_id() {
    let mut manifest = ProjectManifest::new(
        "Duplicate Native Dynamic Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Physics, true, false)
                .with_runtime_crate("zircon_plugin_physics_runtime")
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Physics, true, false)
                .with_runtime_crate("zircon_plugin_physics_runtime")
                .with_packaging(ExportPackagingStrategy::NativeDynamic),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let native_manifest = generated_file(&plan, "plugins/native_plugins.toml");

    assert_eq!(plan.native_dynamic_packages, vec!["physics".to_string()]);
    assert_eq!(occurrences(native_manifest, "[[plugins]]"), 1);
    assert_eq!(occurrences(native_manifest, "id = \"physics\""), 1);
}

#[test]
fn native_dynamic_deduplicates_loader_manifest_packages_by_output_directory() {
    let mut manifest = ProjectManifest::new(
        "Sanitized Native Dynamic Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection {
                id: "physics.debug".to_string(),
                enabled: true,
                required: false,
                target_modes: vec![RuntimeTargetMode::ClientRuntime],
                packaging: ExportPackagingStrategy::NativeDynamic,
                runtime_crate: Some("zircon_plugin_physics_runtime".to_string()),
                editor_crate: None,
            },
            ProjectPluginSelection {
                id: "physics_debug".to_string(),
                enabled: true,
                required: false,
                target_modes: vec![RuntimeTargetMode::ClientRuntime],
                packaging: ExportPackagingStrategy::NativeDynamic,
                runtime_crate: Some("zircon_plugin_physics_runtime".to_string()),
                editor_crate: None,
            },
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let native_manifest = generated_file(&plan, "plugins/native_plugins.toml");

    assert_eq!(
        plan.native_dynamic_packages,
        vec!["physics.debug".to_string()]
    );
    assert_eq!(occurrences(native_manifest, "[[plugins]]"), 1);
    assert_eq!(
        occurrences(native_manifest, "path = \"plugins/physics_debug\""),
        1
    );
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("physics_debug") && diagnostic.contains("plugins/physics_debug")
    }));
}

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn missing_generated_file(plan: &ExportBuildPlan, path: &str) -> bool {
    plan.generated_files.iter().all(|file| file.path != path)
}

fn occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{stamp}"))
}
