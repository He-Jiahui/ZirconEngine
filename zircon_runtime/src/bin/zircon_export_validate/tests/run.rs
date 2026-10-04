use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{AssetUri, ProjectManifest};
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    ExportPackagingStrategy, ExportProfile, ExportTargetPlatform, ProjectPluginFeatureSelection,
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};

use super::super::args::ValidateArgs;
use super::{
    recorded_contents_artifact_path, run, should_write_stdout, write_outputs, ExportValidateReport,
};

#[test]
fn export_validate_closeout_stdout_policy_covers_report_and_explicit_stdout() {
    assert!(should_write_stdout(&validate_args(None, false)));
    assert!(!should_write_stdout(&validate_args(
        Some(PathBuf::from("out/report.json")),
        false,
    )));
    assert!(should_write_stdout(&validate_args(
        Some(PathBuf::from("out/report.json")),
        true,
    )));
}

#[test]
fn export_validate_closeout_records_relative_contents_artifact_as_absolute() {
    let relative = PathBuf::from("out/generated-contents.json");
    let recorded = PathBuf::from(
        recorded_contents_artifact_path(&relative)
            .expect("relative contents artifact path should resolve"),
    );

    assert!(recorded.is_absolute());
    assert_eq!(
        recorded,
        std::path::absolute(relative).expect("expected path should resolve")
    );
}

#[test]
fn export_validate_closeout_project_load_failure_retains_profile_and_exit_code() {
    let root = unique_temp_dir("fatal-report");
    let missing_project = root.join("missing-zircon-project.toml");
    let report_path = root.join("out").join("report.json");

    let exit_code = run([
        OsString::from("--project"),
        missing_project.into_os_string(),
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        report_path.clone().into_os_string(),
    ])
    .expect("manifest load failures should be encoded into the report");

    assert_eq!(exit_code, std::process::ExitCode::from(2));
    let report = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&report_path).expect("fatal report should be written"),
    )
    .expect("fatal report should be JSON");
    assert_eq!(report["profile"], "client");
    assert_eq!(report["fatal"], true);
    assert!(report["fatal_diagnostics"][0]
        .as_str()
        .expect("fatal diagnostic should be text")
        .contains("failed to load project manifest"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn export_validate_closeout_writes_contents_artifact_and_matching_report_metadata() {
    let root = unique_temp_dir("contents-artifact");
    let project_path = root.join("zircon-project.toml");
    let report_path = root.join("out").join("report.json");
    let artifact_path = root.join("out").join("generated-contents.json");
    source_template_manifest()
        .save(&project_path)
        .expect("project manifest should be written");

    let exit_code = run([
        OsString::from("--project"),
        project_path.into_os_string(),
        OsString::from("--profile"),
        OsString::from("server"),
        OsString::from("--report"),
        report_path.clone().into_os_string(),
        OsString::from("--contents-artifact"),
        artifact_path.clone().into_os_string(),
    ])
    .expect("valid project export should produce both outputs");

    assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
    let report = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&report_path).expect("report should be written"),
    )
    .expect("report should be JSON");
    let artifact = fs::read_to_string(&artifact_path).expect("artifact should be written");
    let artifact_json =
        serde_json::from_str::<serde_json::Value>(&artifact).expect("artifact should be JSON");

    assert_eq!(report["schema_version"], 2);
    assert_eq!(artifact_json["schema_version"], 1);
    assert_eq!(
        report["generated_contents_artifact_path"],
        artifact_path.display().to_string()
    );
    assert_eq!(
        report["generated_contents_artifact_byte_length"],
        artifact.len() as u64
    );
    assert_eq!(
        report["generated_contents_artifact_digest"],
        ExportValidateReport::sha256_digest(artifact.as_bytes())
    );
    assert!(artifact_json["generated_files"]
        .as_array()
        .expect("artifact should contain generated files")
        .iter()
        .any(|file| file["path"] == "src/main.rs" && file["contents"].is_string()));
    assert!(report["plan_summary"]["generated_files"]
        .as_array()
        .expect("report should summarize generated files")
        .iter()
        .all(|file| file.get("contents").is_none()));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn export_validate_uses_project_plugin_root_for_linked_feature_admission() {
    let root = unique_temp_dir("linked-feature-source");
    let project_path = root.join("zircon-project.toml");
    let report_path = root.join("report.json");
    let mut manifest = ProjectManifest::new(
        "Linked Feature CLI",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .required(true)
                        .with_provider_package_id("sound_timeline_animation_track")
                        .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
            ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::new("sound_timeline_animation_track"),
                true,
                false,
            ),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)];
    manifest
        .save(&project_path)
        .expect("project manifest fixture");
    fs::create_dir_all(root.join("zircon_plugins/sound_timeline_animation_track"))
        .expect("carrier directory fixture");

    let exit_code = run([
        OsString::from("--project"),
        project_path.into_os_string(),
        OsString::from("--profile"),
        OsString::from("client"),
        OsString::from("--report"),
        report_path.clone().into_os_string(),
    ])
    .expect("missing linked source must be encoded in report");
    assert_eq!(exit_code, std::process::ExitCode::from(2));
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(report_path).expect("report fixture"))
            .expect("report JSON");
    let diagnostics = report["fatal_diagnostics"]
        .as_array()
        .expect("fatal diagnostics array");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.as_str().is_some_and(|diagnostic| {
            diagnostic.contains("sound_timeline_animation_track/plugin.toml")
        })
    }));
    assert!(!diagnostics.iter().any(|diagnostic| {
        diagnostic
            .as_str()
            .is_some_and(|diagnostic| diagnostic.contains("requires a plugin source root"))
    }));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn export_validate_closeout_rejects_check_use_hardlink_replacement_without_truncation() {
    let root = unique_temp_dir("output-identity-replacement");
    let report_path = root.join("report.json");
    let artifact_path = root.join("artifact.json");
    fs::write(&report_path, "preserve-existing-output").expect("report fixture");
    fs::write(&artifact_path, "distinct-at-argument-check").expect("artifact fixture");
    fs::remove_file(&artifact_path).expect("replace artifact fixture");
    fs::hard_link(&report_path, &artifact_path).expect("replacement hard link");

    let error = write_outputs(
        Some((&report_path, "new-report")),
        Some((&artifact_path, "new-artifact")),
    )
    .expect_err("opened output identities must reject the replacement alias");

    assert!(matches!(
        error,
        super::ExportValidateError::OutputPathsAlias { .. }
    ));
    assert_eq!(
        fs::read_to_string(&report_path).expect("report remains readable"),
        "preserve-existing-output"
    );
    assert_eq!(
        fs::read_to_string(&artifact_path).expect("artifact alias remains readable"),
        "preserve-existing-output"
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn export_validate_closeout_rejects_broken_symlink_and_parent_component_aliases_at_open() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_dir("symlink-output-identity");
    let real = root.join("real");
    let nested = real.join("nested");
    fs::create_dir_all(&nested).expect("real output directories");
    let artifact_path = real.join("artifact.json");
    let broken_link = root.join("broken-report.json");
    symlink(&artifact_path, &broken_link).expect("broken output symlink");
    let error = write_outputs(
        Some((&broken_link, "report")),
        Some((&artifact_path, "artifact")),
    )
    .expect_err("broken symlink target must be compared after opening");
    assert!(matches!(
        error,
        super::ExportValidateError::OutputPathsAlias { .. }
    ));

    fs::remove_file(&broken_link).expect("remove broken symlink");
    let directory_link = root.join("linked");
    symlink(&nested, &directory_link).expect("directory symlink");
    let parent_component_path = directory_link.join("..").join("artifact.json");
    let error = write_outputs(
        Some((&parent_component_path, "report")),
        Some((&artifact_path, "artifact")),
    )
    .expect_err("symlink parent traversal must use opened file identity");
    assert!(matches!(
        error,
        super::ExportValidateError::OutputPathsAlias { .. }
    ));
    let _ = fs::remove_dir_all(root);
}

fn source_template_manifest() -> ProjectManifest {
    let mut manifest = ProjectManifest::new(
        "Export Validate CLI Artifact",
        AssetUri::parse("res://scenes/main.zscene").expect("scene URI should parse"),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "server",
        RuntimeTargetMode::ServerRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Server,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)];
    manifest
}

fn validate_args(report: Option<PathBuf>, stdout: bool) -> ValidateArgs {
    ValidateArgs {
        project: PathBuf::from("zircon-project.toml"),
        profile: "client".to_string(),
        report,
        contents_artifact: None,
        stage_output: None,
        pretty: false,
        stdout,
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should follow the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-validate-{label}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temporary test directory should be created");
    root
}
