use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::plugin::NativePluginLoader;

#[path = "native_plugin_loader/real_fixture.rs"]
mod real_fixture;

#[test]
fn native_loader_discovers_candidates_from_export_load_manifest() {
    let root = temp_export_root("native-load-manifest");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.discover_from_load_manifest(&root);

    assert!(
        report.diagnostics().is_empty(),
        "{:?}",
        report.diagnostics()
    );
    assert_eq!(report.discovered().len(), 1);
    assert_eq!(report.discovered()[0].plugin_id, "weather");
    assert_eq!(
        report.discovered()[0].manifest_path,
        plugin_root.join("plugin.toml")
    );
    assert!(
        report.discovered()[0]
            .library_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("zircon_plugin_weather_runtime")
    );
    assert_eq!(
        report.discovered()[0].library_path,
        plugin_root
            .join("native")
            .join(platform_library_file_name("zircon_plugin_weather_runtime"))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_refreshes_current_load_manifest_candidates_through_the_authority() {
    let root = temp_export_root("native-load-manifest-refresh");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let first = NativePluginLoader.discover_from_load_manifest(&root);
    assert_eq!(first.discovered()[0].plugin_id, "weather");

    let root_scan = NativePluginLoader.discover(root.join("plugins").join("native_plugins.toml"));
    assert!(root_scan.discovered().is_empty());
    assert!(
        root_scan
            .diagnostics()
            .iter()
            .any(|message| { message.contains("native plugin root does not exist") })
    );

    fs::write(
        plugin_root.join("plugin.toml"),
        runtime_plugin_manifest().replace("weather", "climate"),
    )
    .unwrap();
    let refreshed = NativePluginLoader.discover_from_load_manifest(&root);

    assert!(refreshed.discovered().is_empty());
    assert!(refreshed.diagnostics().iter().any(|message| {
        message.contains("native plugin climate load manifest id mismatch: entry id weather")
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_keeps_same_root_scan_and_load_manifest_generations_independent() {
    let root = temp_export_root("native-load-manifest-input-isolation");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    let load_manifest_path = root.join("plugins").join("native_plugins.toml");
    fs::write(
        &load_manifest_path,
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let root = Arc::new(root);
    let selection_root = Arc::clone(&root);
    let selection = thread::spawn(move || {
        NativePluginLoader.discover_from_load_manifest(selection_root.as_ref())
    });
    let root_scan = thread::spawn(move || NativePluginLoader.discover(load_manifest_path));

    let selection = selection.join().expect("selection discovery worker");
    let root_scan = root_scan.join().expect("root scan discovery worker");
    assert_eq!(selection.discovered().len(), 1);
    assert_eq!(selection.discovered()[0].plugin_id, "weather");
    assert!(root_scan.discovered().is_empty());
    assert!(
        root_scan
            .diagnostics()
            .iter()
            .any(|message| { message.contains("native plugin root does not exist") })
    );

    let _ = fs::remove_dir_all(root.as_ref());
}

#[test]
fn native_loader_rejects_load_manifest_entry_mismatches_before_loading() {
    let root = temp_export_root("native-load-manifest-mismatch");
    let declared_root = root.join("plugins").join("declared_weather");
    let actual_root = root.join("plugins").join("actual_weather");
    fs::create_dir_all(&declared_root).unwrap();
    fs::create_dir_all(&actual_root).unwrap();
    fs::write(actual_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "declared_weather"
path = "plugins/declared_weather"
manifest = "plugins/actual_weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.load_all_from_load_manifest(&root);

    assert!(report.discovered().is_empty());
    assert!(report.loaded().is_empty());
    assert!(report.runtime_plugin_registration_reports().is_empty());
    assert!(
        report.diagnostics().iter().any(|message| message.contains(
            "native plugin weather load manifest id mismatch: entry id declared_weather"
        ))
    );
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|message| message.contains("native plugin weather load manifest path mismatch"))
    );
    assert!(
        !report
            .diagnostics()
            .iter()
            .any(|message| message.contains("library-open"))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "release performance gate; run through the managed Plugins21 validator"]
fn native_loader_load_manifest_rejection_release_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 16;
    const MAX_REJECTED_TO_VALID_P95_PERCENT: u128 = 125;

    let valid_root = temp_export_root("native-load-manifest-release-valid");
    let valid_plugin_root = valid_root.join("plugins").join("weather");
    fs::create_dir_all(&valid_plugin_root).unwrap();
    fs::write(
        valid_plugin_root.join("plugin.toml"),
        runtime_plugin_manifest(),
    )
    .unwrap();
    fs::write(
        valid_root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let rejected_root = temp_export_root("native-load-manifest-release-rejected");
    let rejected_declared_root = rejected_root.join("plugins").join("declared_weather");
    let rejected_actual_root = rejected_root.join("plugins").join("actual_weather");
    fs::create_dir_all(&rejected_declared_root).unwrap();
    fs::create_dir_all(&rejected_actual_root).unwrap();
    fs::write(
        rejected_actual_root.join("plugin.toml"),
        runtime_plugin_manifest(),
    )
    .unwrap();
    fs::write(
        rejected_root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "declared_weather"
path = "plugins/declared_weather"
manifest = "plugins/actual_weather/plugin.toml"
"#,
    )
    .unwrap();

    let mut valid_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut rejected_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let measure_valid = || measure_load_manifest_admission(&valid_root, ITERATIONS, 1, 1);
        let measure_rejected = || measure_load_manifest_admission(&rejected_root, ITERATIONS, 0, 0);
        if pair_index % 2 == 0 {
            valid_samples_ns.push(measure_valid());
            rejected_samples_ns.push(measure_rejected());
        } else {
            rejected_samples_ns.push(measure_rejected());
            valid_samples_ns.push(measure_valid());
        }
    }

    let valid_p95_ns = nearest_rank_percentile(&valid_samples_ns, 95);
    let rejected_p95_ns = nearest_rank_percentile(&rejected_samples_ns, 95);
    assert!(
        rejected_p95_ns * 100 <= valid_p95_ns * MAX_REJECTED_TO_VALID_P95_PERCENT,
        "rejected manifest P95 {rejected_p95_ns}ns exceeded valid admission P95 {valid_p95_ns}ns by more than {MAX_REJECTED_TO_VALID_P95_PERCENT}%"
    );
    let valid_samples_csv = join_samples(&valid_samples_ns);
    let rejected_samples_csv = join_samples(&rejected_samples_ns);
    println!(
        "PERF-MVP-PLUGINS21-LOAD-MANIFEST-REJECTION sample_pairs={SAMPLE_PAIRS} iterations_per_sample={ITERATIONS} order=alternating_valid_first_even percentile_method=nearest_rank valid_load_eligible_per_sample={ITERATIONS} rejected_load_eligible_per_sample=0 load_eligible_reduction_percent=100 valid_p95_ns={valid_p95_ns} rejected_p95_ns={rejected_p95_ns} max_rejected_to_valid_p95_percent={MAX_REJECTED_TO_VALID_P95_PERCENT} valid_samples_ns={valid_samples_csv} rejected_samples_ns={rejected_samples_csv}"
    );

    let _ = fs::remove_dir_all(valid_root);
    let _ = fs::remove_dir_all(rejected_root);
}

fn measure_load_manifest_admission(
    root: &Path,
    iterations: usize,
    expected_discovered: usize,
    expected_registration_reports: usize,
) -> u128 {
    let started = Instant::now();
    for _ in 0..iterations {
        let report = NativePluginLoader.load_all_from_load_manifest(root);
        assert_eq!(report.discovered().len(), expected_discovered);
        assert_eq!(
            report.runtime_plugin_registration_reports().len(),
            expected_registration_reports
        );
        black_box(report.loaded().len());
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn native_loader_deduplicates_load_manifest_package_ids() {
    let root = temp_export_root("native-load-manifest-duplicate");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"

[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.discover_from_load_manifest(&root);

    assert_eq!(report.discovered().len(), 1);
    assert_eq!(report.discovered()[0].plugin_id, "weather");
    assert!(report.diagnostics().iter().any(|message| {
        message.contains("native plugin weather load manifest duplicate package id ignored")
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_routes_load_manifest_selection_through_the_metered_authority() {
    let source = include_str!("../../plugin/native_plugin_loader/discover_load_manifest.rs");

    assert!(source.contains("discovery_authority().discover_load_manifest"));
    assert!(source.contains("collect_load_manifest"));
    assert!(source.contains("metered_candidate_from_manifest_path"));
    assert!(source.contains("BoundedLoadManifestEntries"));
    assert!(!source.contains("fs::read_to_string"));
    assert!(!source.contains("push_candidate_from_manifest_path"));

    let candidate = include_str!("../../plugin/native_plugin_loader/candidate_from_manifest.rs");
    assert!(candidate.contains("#[cfg(test)]"));
    assert!(candidate.contains("fn test_candidate_from_manifest_path"));
    assert!(!candidate.contains("pub(super) fn candidate_from_manifest_path"));
}

#[test]
fn native_loader_discovers_editor_only_native_package() {
    let root = temp_export_root("native-editor-only");
    let plugin_root = root.join("native_window_hosting");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(
        plugin_root.join("plugin.toml"),
        editor_only_plugin_manifest(),
    )
    .unwrap();

    let report = NativePluginLoader.discover(&root);

    assert!(
        report.diagnostics().is_empty(),
        "{:?}",
        report.diagnostics()
    );
    assert_eq!(report.discovered().len(), 1);
    assert_eq!(report.discovered()[0].plugin_id, "native_window_hosting");
    assert!(
        report.discovered()[0]
            .library_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("zircon_plugin_native_window_hosting_editor")
    );
    assert!(
        report.runtime_plugin_registration_reports().is_empty(),
        "editor-only native packages must not enter runtime plugin registration"
    );
    let runtime_report = NativePluginLoader.load_discovered_runtime(&root);
    assert!(
        runtime_report.diagnostics().is_empty(),
        "runtime-only loading should skip editor-only packages without probing editor libraries: {:?}",
        runtime_report.diagnostics()
    );
    assert!(runtime_report.loaded().is_empty());
    assert!(
        runtime_report
            .runtime_plugin_registration_reports()
            .is_empty()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_discovers_feature_extension_package_from_feature_runtime_module() {
    let root = temp_export_root("native-feature-extension");
    let plugin_root = root.join("sound_timeline_animation_track");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(
        plugin_root.join("plugin.toml"),
        feature_extension_plugin_manifest(),
    )
    .unwrap();

    let report = NativePluginLoader.discover(&root);

    assert!(
        report.diagnostics().is_empty(),
        "{:?}",
        report.diagnostics()
    );
    assert_eq!(report.discovered().len(), 1);
    assert_eq!(
        report.discovered()[0].plugin_id,
        "sound_timeline_animation_track"
    );
    assert!(
        report.discovered()[0]
            .library_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("zircon_plugin_sound_timeline_animation_runtime")
    );
    assert!(report.runtime_plugin_registration_reports().is_empty());

    let feature_reports = report.runtime_plugin_feature_registration_reports();
    assert_eq!(feature_reports.len(), 1);
    assert_eq!(
        feature_reports[0].provider_package_id.as_deref(),
        Some("sound_timeline_animation_track")
    );
    assert_eq!(feature_reports[0].manifest.owner_plugin_id, "sound");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_loader_uses_target_module_crate_for_split_native_package_loading() {
    let root = temp_export_root("native-split-target-library");
    let plugin_root = root.join("split_tool");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(
        plugin_root.join("plugin.toml"),
        split_native_plugin_manifest(),
    )
    .unwrap();

    let runtime_report = NativePluginLoader.load_discovered_runtime(&root);
    assert!(runtime_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));
    assert!(!runtime_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));

    let editor_report = NativePluginLoader.load_discovered_editor(&root);
    assert!(editor_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));
    assert!(!editor_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));

    let full_report = NativePluginLoader.load_discovered_all(&root);
    assert!(full_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_runtime",
        ))
    }));
    assert!(full_report.diagnostics().iter().any(|message| {
        message.contains(&platform_library_file_name(
            "zircon_plugin_split_tool_editor",
        ))
    }));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn native_registration_reports_preserve_per_plugin_loader_diagnostics() {
    let root = temp_export_root("native-registration-diagnostics");
    let plugin_root = root.join("plugins").join("weather");
    fs::create_dir_all(&plugin_root).unwrap();
    fs::write(plugin_root.join("plugin.toml"), runtime_plugin_manifest()).unwrap();
    fs::write(
        root.join("plugins").join("native_plugins.toml"),
        r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"
"#,
    )
    .unwrap();

    let report = NativePluginLoader.load_all_from_load_manifest(&root);
    let registrations = report.runtime_plugin_registration_reports();

    assert_eq!(registrations.len(), 1);
    assert_eq!(registrations[0].package_manifest.id, "weather");
    assert!(registrations[0].diagnostics.iter().any(|message| {
        message.contains("native plugin weather library-open failed")
            && message.contains("expected native dist library, actual artifact missing")
    }));

    let _ = fs::remove_dir_all(root);
}

fn runtime_plugin_manifest() -> &'static str {
    r#"
id = "weather"
version = "0.1.0"
display_name = "Weather"

[[modules]]
name = "weather.runtime"
kind = "runtime"
crate_name = "zircon_plugin_weather_runtime"
"#
}

fn editor_only_plugin_manifest() -> &'static str {
    r#"
id = "native_window_hosting"
version = "0.1.0"
display_name = "Native Window Hosting"

[[modules]]
name = "native_window_hosting.editor"
kind = "editor"
crate_name = "zircon_plugin_native_window_hosting_editor"
"#
}

fn split_native_plugin_manifest() -> &'static str {
    r#"
id = "split_tool"
version = "0.1.0"
display_name = "Split Tool"

[[modules]]
name = "split_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_split_tool_runtime"

[[modules]]
name = "split_tool.editor"
kind = "editor"
crate_name = "zircon_plugin_split_tool_editor"
"#
}

fn feature_extension_plugin_manifest() -> &'static str {
    r#"
id = "sound_timeline_animation_track"
version = "0.1.0"
package_kind = "feature_extension"
display_name = "Sound Timeline Animation Track Provider"

[[feature_extensions]]
id = "sound.timeline_animation_track"
display_name = "Sound Timeline Animation Track"
owner_plugin_id = "sound"
capabilities = ["runtime.feature.sound.timeline_animation_track"]

[[feature_extensions.dependencies]]
plugin_id = "sound"
capability = "runtime.plugin.sound"
primary = true

[[feature_extensions.modules]]
name = "sound.timeline_animation_track.runtime"
kind = "runtime"
crate_name = "zircon_plugin_sound_timeline_animation_runtime"
target_modes = ["client_runtime"]
capabilities = ["runtime.feature.sound.timeline_animation_track"]
"#
}

fn native_dynamic_fixture_load_manifest() -> &'static str {
    r#"
[[plugins]]
id = "native_dynamic_fixture"
path = "plugins/native_dynamic_fixture"
manifest = "plugins/native_dynamic_fixture/plugin.toml"
package_report = "plugins/native_dynamic_fixture/native_dynamic_package.toml"

[plugins.abi]
abi_version = 3
descriptor_symbol = "zircon_native_plugin_descriptor_v3"
descriptor_contract = "NativePluginAbiV3"
runtime_entry_source = "NativePluginAbiV3.runtime_entry_name"
editor_entry_source = "NativePluginAbiV3.editor_entry_name"
host_function_table = "NativePluginHostFunctionTableV3"
entry_report_contract = "NativePluginEntryReportV3"
behavior_contract = "NativePluginBehaviorV4"
state_snapshot_contract = "NativePluginBehaviorV4.save_state/restore_state"
bridge_method_table = "NativePluginBridgeMethodTableV3"
"#
}

fn temp_export_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
}

#[test]
fn native_dynamic_fixture_build_uses_isolated_offline_workspace() {
    let build_root = temp_export_root("native-dynamic-fixture-isolated-workspace");
    let workspace_manifest = prepare_native_dynamic_fixture_workspace(&build_root);
    let workspace_root = workspace_manifest.parent().unwrap();

    assert!(workspace_manifest.starts_with(&build_root));
    assert!(workspace_root.join("native/Cargo.toml").is_file());
    assert!(workspace_root.join("native/src/lib.rs").is_file());
    assert!(workspace_root.join("plugin.toml").is_file());
    let manifest = fs::read_to_string(&workspace_manifest).unwrap();
    assert!(manifest.contains("members = [\"native\"]"));
    assert!(manifest.contains("zircon_plugin_sdk"));

    let target_dir = build_root.join("target");
    let command =
        native_dynamic_fixture_build_command(&workspace_manifest, &target_dir, &["abi_v2_only"]);
    let arguments = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let workspace_manifest_text = workspace_manifest.to_string_lossy().into_owned();
    let plugin_workspace_manifest = repo_root()
        .join("zircon_plugins/Cargo.toml")
        .to_string_lossy()
        .into_owned();
    assert!(arguments.iter().any(|argument| argument == "--offline"));
    assert!(arguments.windows(2).any(|arguments| {
        arguments[0] == "--manifest-path" && arguments[1] == workspace_manifest_text
    }));
    assert!(
        !arguments
            .iter()
            .any(|argument| argument == &plugin_workspace_manifest)
    );
    assert!(
        arguments
            .windows(2)
            .any(|arguments| { arguments[0] == "--features" && arguments[1] == "abi_v2_only" })
    );

    let _ = fs::remove_dir_all(build_root);
}

fn build_native_dynamic_fixture(target_root: &std::path::Path) -> PathBuf {
    build_native_dynamic_fixture_with_features(target_root, &[])
}

fn build_native_dynamic_fixture_with_features(
    target_root: &std::path::Path,
    features: &[&str],
) -> PathBuf {
    let workspace_manifest = prepare_native_dynamic_fixture_workspace(target_root);
    let fixture_target_root = target_root.join("target");
    let status =
        native_dynamic_fixture_build_command(&workspace_manifest, &fixture_target_root, features)
            .status()
            .unwrap();
    assert!(
        status.success(),
        "native dynamic fixture build failed: {status}"
    );
    fixture_target_root
        .join("debug")
        .join(platform_library_file_name(
            "zircon_plugin_native_dynamic_fixture_native",
        ))
}

fn prepare_native_dynamic_fixture_workspace(build_root: &Path) -> PathBuf {
    let workspace_root = build_root.join("fixture-workspace");
    let native_source = repo_root().join("zircon_plugins/native_dynamic_fixture/native");
    let native_destination = workspace_root.join("native");

    copy_directory_recursively(&native_source, &native_destination);
    fs::copy(
        repo_root().join("zircon_plugins/native_dynamic_fixture/plugin.toml"),
        workspace_root.join("plugin.toml"),
    )
    .unwrap();
    let plugin_sdk = toml_literal_path(&repo_root().join("zircon_plugins/plugin_sdk"));
    fs::write(
        workspace_root.join("Cargo.toml"),
        format!(
            r#"[workspace]
members = ["native"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
zircon_plugin_sdk = {{ path = '{plugin_sdk}', default-features = false }}
"#
        ),
    )
    .unwrap();
    workspace_root.join("Cargo.toml")
}

fn copy_directory_recursively(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory_recursively(&entry.path(), &destination_path);
        } else {
            fs::copy(entry.path(), destination_path).unwrap();
        }
    }
}

fn toml_literal_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace('\'', "''")
}

fn native_dynamic_fixture_build_command(
    workspace_manifest: &Path,
    target_root: &Path,
    features: &[&str],
) -> Command {
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(workspace_manifest)
        .arg("-p")
        .arg("zircon_plugin_native_dynamic_fixture_native")
        .arg("--offline")
        .arg("--target-dir")
        .arg(target_root)
        .arg("--quiet");
    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }
    command
}

fn platform_library_file_name(crate_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{crate_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{crate_name}.dylib")
    } else {
        format!("lib{crate_name}.so")
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
