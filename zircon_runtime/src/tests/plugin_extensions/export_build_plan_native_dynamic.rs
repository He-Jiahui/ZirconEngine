use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::{AssetUri, ProjectManifest};
use crate::{
    plugin::ExportBuildPlan, plugin::ExportPackagingStrategy, plugin::ExportProfile,
    plugin::ExportTargetPlatform, plugin::ProjectPluginManifest, plugin::ProjectPluginSelection,
    RuntimePluginId, RuntimeTargetMode,
};

#[test]
fn export_plan_rejects_malformed_native_dynamic_project_plugin_id_before_package_selection() {
    let package_id = "sound/../escape";
    let mut manifest = ProjectManifest::new(
        "Native Dynamic Invalid Package Id Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: package_id.to_string(),
            enabled: true,
            required: true,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging: ExportPackagingStrategy::NativeDynamic,
            runtime_crate: Some("zircon_plugin_sound_runtime".to_string()),
            editor_crate: None,
            features: Vec::new(),
        }],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();

    assert!(plan.native_dynamic_packages.is_empty());
    assert!(plan
        .generated_files
        .iter()
        .all(|file| file.path != "plugins/native_plugins.toml"));
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("project plugin selection id `sound/../escape`")
            && diagnostic.contains("contain only lowercase ASCII letters")
    }));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("project plugin selection id `sound/../escape`")
            && diagnostic.contains("contain only lowercase ASCII letters")
    }));
}

#[test]
fn native_dynamic_generates_loader_manifest_without_source_template() {
    let plan = native_dynamic_plan();
    let native_manifest = generated_file(&plan, "plugins/native_plugins.toml");

    assert_eq!(plan.native_dynamic_packages, vec!["sound".to_string()]);
    assert!(plan.linked_runtime_crates.is_empty());
    assert!(native_manifest.contains("[[plugins]]"));
    assert!(native_manifest.contains("id = \"sound\""));
    assert!(native_manifest.contains("path = \"plugins/sound\""));
    assert!(native_manifest.contains("manifest = \"plugins/sound/plugin.toml\""));
    assert!(plan
        .generated_files
        .iter()
        .all(|file| file.path != "Cargo.toml"));
}

#[test]
fn native_dynamic_materialization_copies_runtime_package_without_source_crates() {
    let plugin_root = temp_dir("zircon_native_dynamic_plugin_root");
    let output_root = temp_dir("zircon_native_dynamic_output_root");
    let package_root = plugin_root.join("sound");
    fs::create_dir_all(package_root.join("runtime/src")).unwrap();
    fs::create_dir_all(package_root.join("editor/src")).unwrap();
    fs::create_dir_all(package_root.join("native/src")).unwrap();
    fs::create_dir_all(package_root.join("native")).unwrap();
    fs::create_dir_all(package_root.join("assets")).unwrap();
    fs::write(package_root.join("plugin.toml"), sound_plugin_manifest()).unwrap();
    fs::write(package_root.join("runtime/Cargo.toml"), "[package]\n").unwrap();
    fs::write(
        package_root.join("runtime/src/lib.rs"),
        "pub fn linked() {}\n",
    )
    .unwrap();
    fs::write(
        package_root.join("editor/src/lib.rs"),
        "pub fn editor() {}\n",
    )
    .unwrap();
    fs::write(package_root.join("native/Cargo.toml"), "[package]\n").unwrap();
    fs::write(
        package_root.join("native/src/lib.rs"),
        "pub fn native() {}\n",
    )
    .unwrap();
    fs::write(package_root.join("native/sound.dll"), "dynamic-library").unwrap();
    fs::write(package_root.join("assets/material.toml"), "name = \"mat\"").unwrap();

    let plan = native_dynamic_plan();
    let report = plan
        .materialize_with_native_packages(&plugin_root, &output_root)
        .unwrap();
    let copied = output_root.join("plugins/sound");

    assert!(report.copied_packages.contains(&copied));
    assert!(copied.join("plugin.toml").exists());
    assert!(copied.join("native/sound.dll").exists());
    assert!(copied.join("assets/material.toml").exists());
    assert!(!copied.join("runtime/Cargo.toml").exists());
    assert!(!copied.join("runtime/src/lib.rs").exists());
    assert!(!copied.join("editor/src/lib.rs").exists());
    assert!(!copied.join("native/Cargo.toml").exists());
    assert!(!copied.join("native/src/lib.rs").exists());

    let _ = fs::remove_dir_all(plugin_root);
    let _ = fs::remove_dir_all(output_root);
}

#[test]
fn native_dynamic_materialization_sanitizes_package_directory_names() {
    let plugin_root = temp_dir("zircon_native_dynamic_unsafe_plugin_root");
    let output_root = temp_dir("zircon_native_dynamic_unsafe_output_root");
    let package_id = "sound/../escape";
    let package_root = plugin_root.join("unsafe_package");
    fs::create_dir_all(package_root.join("native")).unwrap();
    fs::write(
        package_root.join("plugin.toml"),
        format!(
            "id = {package_id:?}\nversion = \"0.1.0\"\ndisplay_name = \"Unsafe Sound\"\n\n[[modules]]\nname = \"sound.runtime\"\nkind = \"runtime\"\ncrate_name = \"zircon_plugin_sound_runtime\"\ntarget_modes = [\"client_runtime\"]\n"
        ),
    )
    .unwrap();
    fs::write(package_root.join("native/sound.dll"), "dynamic-library").unwrap();
    let mut plan = native_dynamic_plan();
    plan.native_dynamic_packages = vec![package_id.to_string()];
    plan.generated_files.clear();

    let report = plan
        .materialize_with_native_packages(&plugin_root, &output_root)
        .unwrap();
    let copied = output_root.join("plugins/sound____escape");

    assert!(report.copied_packages.contains(&copied));
    assert!(copied.join("plugin.toml").exists());
    assert!(copied.join("native/sound.dll").exists());
    assert!(!output_root.join("escape/plugin.toml").exists());

    let _ = fs::remove_dir_all(plugin_root);
    let _ = fs::remove_dir_all(output_root);
}

#[test]
fn native_dynamic_materialization_does_not_directly_resolve_package_id_outside_plugin_root() {
    let plugin_root = temp_dir("zircon_native_dynamic_direct_escape_plugin_root");
    let output_root = temp_dir("zircon_native_dynamic_direct_escape_output_root");
    let external_root = temp_dir("zircon_native_dynamic_direct_escape_external_package");
    let external_name = external_root
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let package_id = format!("../{external_name}");
    fs::create_dir_all(external_root.join("native")).unwrap();
    fs::write(
        external_root.join("plugin.toml"),
        format!(
            "id = {package_id:?}\nversion = \"0.1.0\"\ndisplay_name = \"External Native\"\n\n[[modules]]\nname = \"external.runtime\"\nkind = \"runtime\"\ncrate_name = \"zircon_plugin_external_runtime\"\ntarget_modes = [\"client_runtime\"]\n"
        ),
    )
    .unwrap();
    fs::write(external_root.join("native/external.dll"), "dynamic-library").unwrap();
    let mut plan = native_dynamic_plan();
    plan.native_dynamic_packages = vec![package_id.clone()];
    plan.generated_files.clear();
    let report = plan
        .materialize_with_native_packages(&plugin_root, &output_root)
        .unwrap();

    assert!(report.copied_packages.is_empty());
    assert!(report
        .diagnostics
        .iter()
        .any(|message| { message.contains("was selected but no plugin.toml was found under") }));
    assert!(!output_root
        .join("plugins")
        .join(format!("___{external_name}"))
        .join("plugin.toml")
        .exists());

    let _ = fs::remove_dir_all(plugin_root);
    let _ = fs::remove_dir_all(output_root);
    let _ = fs::remove_dir_all(external_root);
}

#[test]
fn native_dynamic_materialization_reports_source_only_native_package() {
    let plugin_root = temp_dir("zircon_native_dynamic_source_only_plugin_root");
    let output_root = temp_dir("zircon_native_dynamic_source_only_output_root");
    let package_root = plugin_root.join("sound");
    fs::create_dir_all(package_root.join("native/src")).unwrap();
    fs::write(package_root.join("plugin.toml"), sound_plugin_manifest()).unwrap();
    fs::write(package_root.join("native/Cargo.toml"), "[package]\n").unwrap();
    fs::write(
        package_root.join("native/src/lib.rs"),
        "pub fn native() {}\n",
    )
    .unwrap();

    let report = native_dynamic_plan()
        .materialize_with_native_packages(&plugin_root, &output_root)
        .unwrap();
    let copied = output_root.join("plugins/sound");

    assert!(copied.join("plugin.toml").exists());
    assert!(!copied.join("native/Cargo.toml").exists());
    assert!(!copied.join("native/src/lib.rs").exists());
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("no dynamic library artifacts")));

    let _ = fs::remove_dir_all(plugin_root);
    let _ = fs::remove_dir_all(output_root);
}

#[test]
fn native_dynamic_materialization_reports_missing_native_directory() {
    let plugin_root = temp_dir("zircon_native_dynamic_missing_native_plugin_root");
    let output_root = temp_dir("zircon_native_dynamic_missing_native_output_root");
    let package_root = plugin_root.join("sound");
    fs::create_dir_all(&package_root).unwrap();
    fs::write(package_root.join("plugin.toml"), sound_plugin_manifest()).unwrap();

    let report = native_dynamic_plan()
        .materialize_with_native_packages(&plugin_root, &output_root)
        .unwrap();
    let copied = output_root.join("plugins/sound");

    assert!(copied.join("plugin.toml").exists());
    assert!(report
        .diagnostics
        .iter()
        .any(|message| message.contains("no native artifact directory")));

    let _ = fs::remove_dir_all(plugin_root);
    let _ = fs::remove_dir_all(output_root);
}

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn native_dynamic_plan() -> ExportBuildPlan {
    let mut manifest = ProjectManifest::new(
        "Native Dynamic Materialize Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime")
        .with_packaging(ExportPackagingStrategy::NativeDynamic)],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];
    ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap()
}

fn sound_plugin_manifest() -> &'static str {
    r#"
id = "sound"
version = "0.1.0"
display_name = "Sound"

[[modules]]
name = "sound.runtime"
kind = "runtime"
crate_name = "zircon_plugin_sound_runtime"
target_modes = ["client_runtime"]
"#
}

fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{stamp}"))
}
