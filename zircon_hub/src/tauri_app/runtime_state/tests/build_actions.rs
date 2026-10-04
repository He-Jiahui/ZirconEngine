use std::{fs, path::PathBuf};

use crate::build::BuildExecutionReport;
use crate::engines::source_engine_id;
use crate::projects::{project_metadata_key, ProjectMetadata, RecentProject};
use crate::settings::{HubConfig, HubLanguage};
use crate::state::{HubActionKind, HubActionStatus, HubMessage, HubMessageId, ProjectMessageId};
use crate::tauri_app::HubActionRequest;

use super::super::HubRuntimeSession;

#[test]
fn background_build_prepares_command_without_running_or_recording_history() {
    let temp = temp_test_dir("zircon-hub-background-build-prepare");
    let source = create_source_engine_root(&temp);
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);
    start_background_build(&mut session);

    let pending = session
        .prepare_background_editor_runtime_build()
        .expect("background build preparation should not fail hard")
        .expect("valid project and engine should produce a pending build");

    assert!(session.task_status.running);
    assert!(session.task_status.cancellable);
    assert_eq!(session.task_status.label, "Building");
    assert!(pending
        .command()
        .command_line()
        .iter()
        .any(|part| part.contains("zircon_build.py")));
    let bound_engine = session
        .config
        .engines
        .iter()
        .find(|engine| Some(engine.id.as_str()) == session.config.active_engine_id.as_deref())
        .expect("prepared build should retain its bound Source Engine");
    let bound_build_script = bound_engine
        .source_dir
        .join("tools")
        .join("zircon_build.py")
        .to_string_lossy()
        .into_owned();
    let bound_output = bound_engine.output_dir.to_string_lossy().into_owned();
    let command_line = pending.command().command_line();
    assert!(command_line.iter().any(|part| part == &bound_build_script));
    assert!(pending
        .build_output_dir()
        .starts_with(&bound_engine.output_dir));
    let pending_output = pending.build_output_dir().to_string_lossy().into_owned();
    assert!(command_line.iter().any(|part| part == &pending_output));
    assert!(!command_line.iter().any(|part| part == &bound_output));
    assert_eq!(session.config.action_history.len(), 0);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_build_completion_records_success_after_external_result() {
    let temp = temp_test_dir("zircon-hub-background-build-complete");
    let source = create_source_engine_root(&temp);
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);
    let active_staged_engine_dir = session.config.engines[0].staged_engine_dir();
    fs::create_dir_all(&active_staged_engine_dir).unwrap();
    fs::write(
        active_staged_engine_dir.join("last-good.marker"),
        b"last-good",
    )
    .unwrap();
    start_background_build(&mut session);
    let pending = session
        .prepare_background_editor_runtime_build()
        .unwrap()
        .unwrap();
    super::staged_build::write_valid_test_fixture(
        &pending.staged_engine_dir,
        &pending.source_dir,
        &pending.profile,
    );
    finish_active_background_task(&mut session);

    session
        .complete_background_editor_runtime_build(
            pending,
            Ok(BuildExecutionReport {
                status_code: Some(0),
                stdout: "staged editor/runtime\n".to_string(),
                stderr: String::new(),
            }),
        )
        .expect("successful external result should complete build state");

    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::BuildEditorRuntime);
    assert_eq!(record.status, HubActionStatus::Success);
    assert_eq!(session.task_status.label, "Build complete");
    let active_engine = session
        .config
        .engines
        .iter()
        .find(|engine| session.config.active_engine_id.as_deref() == Some(engine.id.as_str()))
        .expect("build should keep active Source Engine");
    assert_eq!(active_engine.build_history.len(), 1);
    assert_eq!(active_engine.build_history[0].status, "success");
    assert!(!active_staged_engine_dir.join("last-good.marker").exists());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn zero_exit_without_a_staging_manifest_records_a_failed_build() {
    let temp = temp_test_dir("zircon-hub-background-build-missing-manifest");
    let source = create_source_engine_root(&temp);
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);
    let active_staged_engine_dir = session.config.engines[0].staged_engine_dir();
    fs::create_dir_all(&active_staged_engine_dir).unwrap();
    fs::write(
        active_staged_engine_dir.join("last-good.marker"),
        b"last-good",
    )
    .unwrap();
    start_background_build(&mut session);
    let pending = session
        .prepare_background_editor_runtime_build()
        .unwrap()
        .unwrap();
    super::staged_build::write_valid_test_fixture(
        &pending.staged_engine_dir,
        &pending.source_dir,
        &pending.profile,
    );
    fs::remove_file(pending.staged_engine_dir.join("staging_manifest.json")).unwrap();
    finish_active_background_task(&mut session);

    session
        .complete_background_editor_runtime_build(
            pending,
            Ok(BuildExecutionReport {
                status_code: Some(0),
                stdout: "staged editor/runtime\n".to_string(),
                stderr: String::new(),
            }),
        )
        .unwrap();

    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );
    assert_eq!(session.task_status.label, "Build artifacts invalid");
    assert_eq!(session.config.engines[0].build_history[0].status, "failed");
    assert!(session.config.engines[0].last_build_unix_ms.is_none());
    assert!(active_staged_engine_dir.join("last-good.marker").is_file());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn build_completion_stays_bound_to_the_engine_and_settings_captured_at_prepare() {
    let temp = temp_test_dir("zircon-hub-background-build-stable-owner");
    let source = create_source_engine_root(&temp);
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);
    let original_engine_id = session
        .config
        .active_engine_id
        .clone()
        .expect("fixture should register its source engine");
    let original_jobs = session.config.settings.jobs;
    let original_output = session.config.settings.default_build_output_dir.clone();
    start_background_build(&mut session);
    let pending = session
        .prepare_background_editor_runtime_build()
        .unwrap()
        .unwrap();
    super::staged_build::write_valid_test_fixture(
        &pending.staged_engine_dir,
        &pending.source_dir,
        &pending.profile,
    );

    let mut other_engine = session.config.engines[0].clone();
    other_engine.id = "other-engine".to_string();
    other_engine.display_name = "Other Engine".to_string();
    other_engine.build_history.clear();
    session.config.engines.push(other_engine);
    session.config.active_engine_id = Some("other-engine".to_string());
    session.config.settings.jobs = original_jobs.saturating_add(1);
    session.config.settings.default_build_output_dir = temp.join("other-output");
    finish_active_background_task(&mut session);

    session
        .complete_background_editor_runtime_build(
            pending,
            Ok(BuildExecutionReport {
                status_code: Some(0),
                stdout: "staged editor/runtime\n".to_string(),
                stderr: String::new(),
            }),
        )
        .unwrap();

    let original_engine = session
        .config
        .engines
        .iter()
        .find(|engine| engine.id == original_engine_id)
        .expect("prepared engine should remain in the registry");
    assert_eq!(original_engine.build_history.len(), 1);
    assert_eq!(original_engine.build_history[0].jobs, Some(original_jobs));
    assert_eq!(original_engine.build_history[0].output_dir, original_output);
    let other_engine = session
        .config
        .engines
        .iter()
        .find(|engine| engine.id == "other-engine")
        .unwrap();
    assert!(other_engine.build_history.is_empty());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn build_completion_localizes_success_history_detail() {
    let temp = temp_test_dir("zircon-hub-background-build-complete-localized");
    let source = create_source_engine_root(&temp);
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);
    session.config.settings.language = HubLanguage::Chinese;
    start_background_build(&mut session);
    let pending = session
        .prepare_background_editor_runtime_build()
        .unwrap()
        .unwrap();
    super::staged_build::write_valid_test_fixture(
        &pending.staged_engine_dir,
        &pending.source_dir,
        &pending.profile,
    );
    finish_active_background_task(&mut session);

    session
        .complete_background_editor_runtime_build(
            pending,
            Ok(BuildExecutionReport {
                status_code: Some(0),
                stdout: "staged editor/runtime\n".to_string(),
                stderr: String::new(),
            }),
        )
        .expect("successful external result should complete localized build state");

    let model = session.view_model();
    assert_eq!(model.task_summary.label, "构建完成");
    assert_eq!(model.action_history[0].action, "构建编辑器/运行时");
    assert_eq!(model.action_history[0].detail, "已暂存编辑器/运行时载荷");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn build_precondition_failure_localizes_unbound_source_engine_detail() {
    let temp = temp_test_dir("zircon-hub-build-unbound-engine-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project_without_engine(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;

    let pending = session
        .prepare_background_editor_runtime_build()
        .expect("missing project engine binding should be a recoverable task status");

    assert!(pending.is_none());
    assert_eq!(
        session.config.action_history[0].detail,
        HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::NoBoundSourceEngine),
            ["Game"]
        )
    );
    let model = session.view_model();
    assert_eq!(model.task_summary.label, "构建编辑器/运行时失败");
    assert_eq!(model.task_summary.detail, "项目未绑定源码引擎：Game");
    assert_eq!(model.action_history[0].detail, "项目未绑定源码引擎：Game");
    assert_eq!(
        model.task_summary.recovery.as_deref(),
        Some("构建前先选择一个已绑定源码引擎的有效项目")
    );

    fs::remove_dir_all(temp).unwrap();
}

fn session_with_project_and_engine(
    temp: &std::path::Path,
    name: &str,
    project: &std::path::Path,
    source: &std::path::Path,
) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let engine_id = source_engine_id(source);
    let mut config = HubConfig::default();
    config.settings.default_source_dir = source.to_path_buf();
    config.settings.default_build_output_dir = temp.join("out");
    config.recent_projects = vec![RecentProject::fixture(name, project, 1)];
    config.runtime.selected_project_path = Some(project.to_path_buf());
    config.project_metadata.insert(
        project_metadata_key(project),
        ProjectMetadata {
            engine_id: Some(engine_id.clone()),
            ..ProjectMetadata::default()
        },
    );
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn session_with_project_without_engine(
    temp: &std::path::Path,
    name: &str,
    project: &std::path::Path,
) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_build_output_dir = temp.join("out");
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

fn create_source_engine_root(temp: &std::path::Path) -> PathBuf {
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(source.join("tools")).unwrap();
    fs::create_dir_all(source.join("zircon_runtime")).unwrap();
    fs::write(
        source.join("Cargo.toml"),
        "[workspace]\nmembers = [\"zircon_runtime\"]\n",
    )
    .unwrap();
    fs::write(source.join("tools").join("zircon_build.py"), "").unwrap();
    source
}

fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
    let project = temp.join(name);
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("zircon-project.toml"),
        format!("name = \"{name}\"\n"),
    )
    .unwrap();
    project
}

fn start_background_build(session: &mut HubRuntimeSession) {
    session
        .start_background_action_status(&HubActionRequest {
            action_id: "build-project".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("start background build status");
}

fn finish_active_background_task(session: &mut HubRuntimeSession) {
    let task_id = session
        .active_background_task_id()
        .expect("background build should remain active until completion");
    session.finish_background_task(task_id);
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
