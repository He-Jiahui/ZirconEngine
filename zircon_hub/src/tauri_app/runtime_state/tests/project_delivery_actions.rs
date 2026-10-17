use std::{fs, path::PathBuf};

use crate::error::HubError;
use crate::projects::RecentProject;
use crate::settings::{HubConfig, HubLanguage};
use crate::state::{HubActionKind, HubActionStatus, TaskExecutionOutcome};

use super::{
    super::{action_tasks::BackgroundTaskContext, HubRuntimeSession},
    resolve_device_install_stage, BackgroundTask, PendingDeviceInstall,
};

#[test]
fn background_package_prepares_request_without_copying_or_recording_history() {
    let temp = temp_test_dir("zircon-hub-background-package-prepare");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);

    let pending = session
        .prepare_background_project_package()
        .expect("package preparation should not fail hard")
        .expect("valid project should prepare package request");

    assert_eq!(pending.project_name, "Game");
    assert_eq!(session.config.action_history.len(), 0);
    assert!(!session.config.settings.default_build_output_dir.exists());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_package_completion_records_success_after_copy_result() {
    let temp = temp_test_dir("zircon-hub-background-package-complete");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    let pending = session
        .prepare_background_project_package()
        .unwrap()
        .unwrap();
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_project_package(pending, result)
        .expect("package completion should record state");

    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::PackageProject);
    assert_eq!(record.status, HubActionStatus::Success);
    assert_eq!(session.task_status.label, "Package created");
    assert!(record
        .output_dir
        .as_ref()
        .unwrap()
        .join("zircon-package.toml")
        .is_file());
    assert!(record
        .command_line
        .iter()
        .any(|part| part == "package-project"));
    assert!(record.command_line.iter().any(|part| part == "--project"));
    assert!(record
        .command_line
        .iter()
        .any(|part| part == &project.to_string_lossy()));
    assert!(record.log_excerpt.contains("2 files"));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn package_action_uses_explicit_target_project_instead_of_selected_project() {
    let temp = temp_test_dir("zircon-hub-package-explicit-target");
    let selected_project = create_project_root(&temp, "Selected");
    let target_project = create_project_root(&temp, "Target");
    let mut session = session_with_projects(
        &temp,
        &[
            ("Selected", selected_project.clone()),
            ("Target", target_project.clone()),
        ],
        &selected_project,
    );

    session
        .apply_action(super::super::HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some(target_project.to_string_lossy().into_owned()),
            payload: None,
        })
        .expect("package action should accept a project target id");

    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::PackageProject);
    assert_eq!(record.status, HubActionStatus::Success);
    assert_eq!(record.target, "Target");
    let output_dir = record
        .output_dir
        .as_ref()
        .expect("target package should record output dir");
    assert!(output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("target-")));
    assert!(fs::read_to_string(output_dir.join("zircon-package.toml"))
        .unwrap()
        .contains("package_name = \"Target\""));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn package_completion_localizes_success_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-background-package-complete-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    let pending = session
        .prepare_background_project_package()
        .unwrap()
        .unwrap();
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_project_package(pending, result)
        .expect("package completion should record localized state");

    let package_dir = session.config.action_history[0]
        .output_dir
        .as_ref()
        .expect("package history should keep output dir")
        .to_string_lossy()
        .into_owned();
    let expected_detail = format!("Game -> {package_dir}（2 个文件）");
    let model = session.view_model();
    assert_eq!(model.task_summary.label, "包已创建");
    assert_eq!(model.task_summary.detail, expected_detail);
    assert_eq!(model.action_history[0].action, "打包项目");
    assert_eq!(model.action_history[0].detail, expected_detail);
    assert_eq!(
        model.action_history[0].log_excerpt,
        format!("已打包 Game 到 {package_dir}（2 个文件）")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_install_runs_package_then_device_copy_before_recording_history() {
    let temp = temp_test_dir("zircon-hub-background-install-complete");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    let pending = session
        .prepare_background_device_install()
        .unwrap()
        .unwrap();
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_device_install(pending, result)
        .expect("install completion should record package and install state");

    let install = &session.config.action_history[0];
    let package = &session.config.action_history[1];
    assert_eq!(install.action, HubActionKind::InstallProject);
    assert_eq!(package.action, HubActionKind::PackageProject);
    assert_eq!(session.task_status.label, "Installed to device");
    assert!(install
        .output_dir
        .as_ref()
        .unwrap()
        .join("zircon-package.toml")
        .is_file());
    assert!(package
        .command_line
        .iter()
        .any(|part| part == "package-project"));
    assert!(package.log_excerpt.contains("2 files"));
    assert!(install
        .command_line
        .iter()
        .any(|part| part == "install-device"));
    assert!(install.command_line.iter().any(|part| part == "--device"));
    assert!(install.log_excerpt.contains("3 files"));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn cancelled_install_stage_removes_the_unpublished_intermediate_package() {
    let temp = temp_test_dir("zircon-hub-install-cancelled-package-cleanup");
    let package_dir = temp.join("packages").join("game-42");
    fs::create_dir_all(&package_dir).unwrap();
    fs::write(package_dir.join("zircon-package.toml"), "files_copied = 1").unwrap();
    let package_report = crate::projects::ProjectPackageReport {
        package_dir: package_dir.clone(),
        manifest_path: package_dir.join("zircon-package.toml"),
        files_copied: 1,
    };

    let outcome = resolve_device_install_stage(package_report, Ok(TaskExecutionOutcome::Cancelled))
        .expect("successful rollback should preserve the cancellation outcome");

    assert!(matches!(outcome, TaskExecutionOutcome::Cancelled));
    assert!(!package_dir.exists());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn failed_install_stage_removes_the_unpublished_intermediate_package() {
    let temp = temp_test_dir("zircon-hub-install-failed-package-cleanup");
    let package_dir = temp.join("packages").join("game-42");
    fs::create_dir_all(&package_dir).unwrap();
    let package_report = crate::projects::ProjectPackageReport {
        package_dir: package_dir.clone(),
        manifest_path: package_dir.join("zircon-package.toml"),
        files_copied: 1,
    };

    let error =
        resolve_device_install_stage(package_report, Err(HubError::message("device copy failed")))
            .expect_err("install failure should remain a failure after rollback");

    assert_eq!(error.to_string(), "device copy failed");
    assert!(!package_dir.exists());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn install_completion_localizes_success_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-background-install-complete-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    let pending = session
        .prepare_background_device_install()
        .unwrap()
        .unwrap();
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_device_install(pending, result)
        .expect("install completion should record localized state");

    let install_dir = session.config.action_history[0]
        .output_dir
        .as_ref()
        .expect("install history should keep output dir")
        .to_string_lossy()
        .into_owned();
    let expected_detail = format!("Game -> {install_dir}（3 个文件）");
    let model = session.view_model();
    assert_eq!(model.task_summary.label, "已安装到设备");
    assert_eq!(model.task_summary.detail, expected_detail);
    assert_eq!(model.action_history[0].action, "安装到设备");
    assert_eq!(model.action_history[0].detail, expected_detail);
    assert_eq!(
        model.action_history[0].log_excerpt,
        format!("已安装 Game 到 {install_dir}（3 个文件）")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn package_failure_localizes_required_output_root_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-package-output-required-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    session.config.settings.default_build_output_dir = PathBuf::new();
    let pending = session
        .prepare_background_project_package()
        .unwrap()
        .expect("valid project should still prepare a package request");
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_project_package(pending, result)
        .expect("package failure should record recoverable state");

    assert_eq!(
        session.task_status.detail,
        "Package output root is required"
    );
    let model = session.view_model();
    assert_eq!(model.task_summary.label, "打包项目失败");
    assert_eq!(model.task_summary.detail, "需要包输出根目录");
    assert_eq!(model.action_history[0].detail, "需要包输出根目录");
    assert_eq!(
        model.action_history[0].recovery.as_deref(),
        Some("检查项目根目录是否存在，并确保包输出目录位于项目外")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn package_failure_localizes_output_inside_project_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-package-output-inside-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    session.config.settings.default_build_output_dir = project.join("build-output");
    let pending = session
        .prepare_background_project_package()
        .unwrap()
        .expect("valid project should still prepare a package request");
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_project_package(pending, result)
        .expect("package failure should record recoverable state");

    assert_eq!(
        session.task_status.detail,
        "Package output root must be outside the project directory"
    );
    let model = session.view_model();
    assert_eq!(model.task_summary.detail, "包输出根目录必须位于项目目录外");
    assert_eq!(
        model.action_history[0].detail,
        "包输出根目录必须位于项目目录外"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn install_failure_localizes_required_device_root_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-install-device-required-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    session.config.settings.default_device_install_dir = PathBuf::new();
    let pending = session
        .prepare_background_device_install()
        .unwrap()
        .expect("valid project should still prepare an install request");
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_device_install(pending, result)
        .expect("install failure should record recoverable state");

    assert_eq!(
        session.task_status.detail,
        "Device install directory is required"
    );
    let model = session.view_model();
    assert_eq!(model.task_summary.label, "安装到设备失败");
    assert_eq!(model.task_summary.detail, "需要设备安装目录");
    assert_eq!(model.action_history[0].detail, "需要设备安装目录");
    assert_eq!(
        model.action_history[0].recovery.as_deref(),
        Some("重试前检查包输出和已配置的本地设备安装目录")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn install_failure_localizes_device_root_inside_package_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-install-device-inside-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    let pending = session
        .prepare_background_device_install()
        .unwrap()
        .expect("valid project should still prepare an install request");
    let package_dir = pending
        .package_request
        .output_root
        .join("packages")
        .join(format!("game-{}", pending.package_request.created_unix_ms));
    let pending = PendingDeviceInstall {
        device_root: package_dir.join("device"),
        ..pending
    };
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_device_install(pending, result)
        .expect("install failure should record recoverable state");

    assert_eq!(
        session.task_status.detail,
        "Device install directory must be outside the package directory"
    );
    let model = session.view_model();
    assert_eq!(model.task_summary.detail, "设备安装目录必须位于包目录外");
    assert_eq!(
        model.action_history[0].detail,
        "设备安装目录必须位于包目录外"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn install_failure_localizes_duplicate_install_directory_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-install-duplicate-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    let pending = session
        .prepare_background_device_install()
        .unwrap()
        .expect("valid project should still prepare an install request");
    let package_dir = pending
        .package_request
        .output_root
        .join("packages")
        .join(format!("game-{}", pending.package_request.created_unix_ms));
    let install_dir = pending.device_root.join(
        package_dir
            .file_name()
            .expect("package dir should have an install name"),
    );
    fs::create_dir_all(&install_dir).unwrap();
    let result = completed_result(pending.run(&background_context()));

    session
        .complete_background_device_install(pending, result)
        .expect("install failure should record recoverable state");

    let expected_detail = format!(
        "Device install already exists: {}",
        install_dir.to_string_lossy()
    );
    assert_eq!(session.task_status.detail, expected_detail);
    let model = session.view_model();
    let expected_localized = format!("设备安装已存在：{}", install_dir.to_string_lossy());
    assert_eq!(model.task_summary.detail, expected_localized);
    assert_eq!(model.action_history[0].detail, expected_localized);

    fs::remove_dir_all(temp).unwrap();
}

fn session_with_project(
    temp: &std::path::Path,
    name: &str,
    project: &std::path::Path,
) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_build_output_dir = temp.join("out");
    config.settings.default_device_install_dir = temp.join("device");
    config.recent_projects = vec![RecentProject::fixture(name, project, 1)];
    config.runtime.selected_project_path = Some(project.to_path_buf());
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn background_context() -> BackgroundTaskContext {
    BackgroundTaskContext::for_test(1)
}

fn completed_result<T>(result: Result<TaskExecutionOutcome<T>, HubError>) -> Result<T, HubError> {
    result.and_then(|outcome| match outcome {
        TaskExecutionOutcome::Completed(value) => Ok(value),
        TaskExecutionOutcome::Cancelled => Err(HubError::message(
            "test background task was unexpectedly cancelled",
        )),
    })
}

fn session_with_projects(
    temp: &std::path::Path,
    projects: &[(&str, PathBuf)],
    selected_project: &std::path::Path,
) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_build_output_dir = temp.join("out");
    config.settings.default_device_install_dir = temp.join("device");
    config.recent_projects = projects
        .iter()
        .map(|(name, path)| RecentProject::fixture(*name, path, 1))
        .collect();
    config.runtime.selected_project_path = Some(selected_project.to_path_buf());
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
    let project = temp.join(name);
    fs::create_dir_all(project.join("Assets")).unwrap();
    fs::write(
        project.join("zircon-project.toml"),
        format!("name = \"{name}\"\n"),
    )
    .unwrap();
    fs::write(project.join("Assets").join("mesh.txt"), "mesh").unwrap();
    project
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
