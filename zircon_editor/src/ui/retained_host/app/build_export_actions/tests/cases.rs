use super::*;
use crate::core::jobs::test_job_system;
use std::path::{Path, PathBuf};
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportTargetPlatform};

#[test]
fn build_export_actions_parse_execute_profile() {
    match parse_build_export_action("workbench.build_export.execute.desktop_windows") {
        Some(BuildExportAction::Execute { profile_name }) => {
            assert_eq!(profile_name, "desktop_windows");
        }
        _ => panic!("execute action should parse"),
    }
    match parse_build_export_action("workbench.build_export.plan.desktop_windows") {
        Some(BuildExportAction::GeneratePlan { profile_name }) => {
            assert_eq!(profile_name, "desktop_windows");
        }
        _ => panic!("plan action should parse"),
    }
    match parse_build_export_action("workbench.build_export.cancel.desktop_windows") {
        Some(BuildExportAction::Cancel { profile_name }) => {
            assert_eq!(profile_name, "desktop_windows");
        }
        _ => panic!("cancel action should parse"),
    }
    match parse_build_export_action(
        "workbench.build_export.output.set.desktop_windows|D:/Builds/Zircon",
    ) {
        Some(BuildExportAction::SetOutput {
            profile_name,
            output_root,
        }) => {
            assert_eq!(profile_name, "desktop_windows");
            assert_eq!(output_root, "D:/Builds/Zircon");
        }
        _ => panic!("set-output action should parse"),
    }
    match parse_build_export_action("workbench.build_export.output.choose.desktop_windows") {
        Some(BuildExportAction::ChooseOutput { profile_name }) => {
            assert_eq!(profile_name, "desktop_windows");
        }
        _ => panic!("choose-output action should parse"),
    }
    match parse_build_export_action("workbench.build_export.output.clear.desktop_windows") {
        Some(BuildExportAction::ClearOutput { profile_name }) => {
            assert_eq!(profile_name, "desktop_windows");
        }
        _ => panic!("clear-output action should parse"),
    }
    match parse_build_export_action("workbench.build_export.output.reveal.desktop_windows") {
        Some(BuildExportAction::RevealOutput { profile_name }) => {
            assert_eq!(profile_name, "desktop_windows");
        }
        _ => panic!("reveal-output action should parse"),
    }
    assert!(parse_build_export_action("workbench.build_export.execute.").is_none());
    assert!(parse_build_export_action("workbench.build_export.unknown.desktop_windows").is_none());
}

#[test]
fn desktop_export_output_root_is_project_local_and_profile_scoped() {
    let root = Path::new("Project");
    assert_eq!(
        default_desktop_export_output_root(root, "desktop_linux"),
        PathBuf::from("Project")
            .join("Builds")
            .join("zircon")
            .join("desktop_linux")
    );
}

#[test]
fn build_export_profiles_include_mobile_and_browser_source_scaffolds() {
    let profiles = desktop_export_profiles();
    let android = profiles
        .iter()
        .find(|profile| profile.name == "mobile_android")
        .expect("mobile Android export profile is projected");
    let webgpu = profiles
        .iter()
        .find(|profile| profile.name == "browser_webgpu")
        .expect("WebGPU export profile is projected");
    let headless = profiles
        .iter()
        .find(|profile| profile.name == "headless_server")
        .expect("headless server export profile is projected");

    assert_eq!(android.target_platform, ExportTargetPlatform::Android);
    assert_eq!(webgpu.target_platform, ExportTargetPlatform::WebGpu);
    assert_eq!(headless.target_platform, ExportTargetPlatform::Headless);
    assert_eq!(headless.target_mode, RuntimeTargetMode::ServerRuntime);
    assert!(android.uses_strategy(ExportPackagingStrategy::SourceTemplate));
    assert!(android.uses_strategy(ExportPackagingStrategy::LibraryEmbed));
    assert!(!android.uses_strategy(ExportPackagingStrategy::NativeDynamic));
    assert!(!webgpu.uses_strategy(ExportPackagingStrategy::NativeDynamic));
    assert!(!headless.uses_strategy(ExportPackagingStrategy::NativeDynamic));
}

#[test]
fn desktop_export_job_queue_starts_and_cancels_pending_jobs() {
    let mut queue = DesktopExportJobQueue::new(test_job_system());
    let first = queue.enqueue(
        "desktop_windows",
        PathBuf::from("Project"),
        ProjectManifest::new(
            "Project",
            zircon_runtime::asset::AssetUri::parse("res://main.scene.toml")
                .expect("test asset URI is valid"),
            1,
        ),
        PathBuf::from("Builds/windows"),
    );
    let second = queue.enqueue(
        "desktop_linux",
        PathBuf::from("Project"),
        ProjectManifest::new(
            "Project",
            zircon_runtime::asset::AssetUri::parse("res://main.scene.toml")
                .expect("test asset URI is valid"),
            1,
        ),
        PathBuf::from("Builds/linux"),
    );
    assert_eq!(first.phase, DesktopExportJobPhase::Queued);
    assert_eq!(second.id, first.id + 1);
    assert!(queue.is_profile_busy("desktop_windows"));

    match queue.cancel_profile("desktop_linux") {
        DesktopExportCancellation::PendingCancelled(summary) => {
            assert_eq!(summary.profile_name, "desktop_linux");
            assert_eq!(summary.state, DesktopExportExecutionState::Cancelled);
        }
        other => panic!("expected pending cancellation, got {other:?}"),
    }
    assert!(!queue.is_profile_busy("desktop_linux"));
}

#[test]
fn desktop_export_job_snapshot_projects_stage_progress() {
    let mut target =
        crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData {
            profile_name: "desktop_windows".into(),
            status: "Ready".into(),
            diagnostics: "".into(),
            fatal: true,
            ..Default::default()
        };
    let snapshot = DesktopExportJobSnapshot {
        id: 7,
        profile_name: "desktop_windows".to_string(),
        output_root: PathBuf::from("Builds/windows"),
        phase: DesktopExportJobPhase::Running,
        progress: Some(DesktopExportProgressSnapshot {
            stage: "cargo-build".to_string(),
            percent: 72,
            message: "Running generated SourceTemplate Cargo build".to_string(),
        }),
    };

    apply_job_snapshot_to_target(&mut target, &snapshot);

    assert_eq!(target.status.as_str(), "Running");
    assert!(!target.fatal);
    assert!(target
        .diagnostics
        .as_str()
        .contains("Stage: 72% cargo-build"));
    assert!(target
        .diagnostics
        .as_str()
        .contains("Running generated SourceTemplate Cargo build"));
}
