use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::engines::active_source_engine;
use crate::error::HubError;
use crate::projects::RecentProject;
use crate::state::{
    DeliveryMessageId, EngineMessageId, HubMessage, HubMessageId, ProcessMessageId, ShellMessageId,
    TaskOperationKind, TaskStatus, TASK_PROGRESS_PREPARED_PERCENT,
};

use super::{recent_project_display_name, HubRuntimeSession};
use crate::tauri_app::action_id::HubActionId;
use crate::tauri_app::HubActionRequest;
use crate::tauri_app::HubViewModel;

pub(in crate::tauri_app) trait BackgroundTask: Send + 'static {
    type Output: Send + 'static;

    fn run(&self) -> Result<Self::Output, HubError>;
}

pub(in crate::tauri_app) type EmitState<'a> = &'a dyn Fn(&HubViewModel);

pub(in crate::tauri_app) fn lock_session(
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
) -> MutexGuard<'_, HubRuntimeSession> {
    session_handle.lock().unwrap_or_else(|poisoned| {
        eprintln!("zircon_hub: Hub runtime session lock was poisoned; recovering last state");
        poisoned.into_inner()
    })
}

pub(in crate::tauri_app) fn execute_background_task<T: BackgroundTask>(
    request: &HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
    prepare: fn(&mut HubRuntimeSession) -> Result<Option<T>, HubError>,
    complete: fn(&mut HubRuntimeSession, T, Result<T::Output, HubError>) -> Result<(), HubError>,
) {
    let pending = {
        let mut session = lock_session(session_handle);
        if let Err(error) = session.apply_request_project_target(request) {
            let _ = session.record_background_action_error(request, error);
            let view_model = session.view_model();
            drop(session);
            emit_state(&view_model);
            return;
        }
        let pending = match prepare(&mut session) {
            Ok(pending) => pending,
            Err(error) => {
                let _ = session.record_background_action_error(request, error);
                let view_model = session.view_model();
                drop(session);
                emit_state(&view_model);
                return;
            }
        };
        let view_model = session.view_model();
        drop(session);
        emit_state(&view_model);
        pending
    };

    let Some(pending) = pending else {
        let view_model = lock_session(session_handle).view_model();
        emit_state(&view_model);
        return;
    };

    let result = pending.run();
    let mut session = lock_session(session_handle);
    let view_model = match complete(&mut session, pending, result) {
        Ok(()) => session.view_model(),
        Err(error) => {
            let _ = session.record_background_action_error(request, error);
            session.view_model()
        }
    };
    drop(session);
    emit_state(&view_model);
}

pub(in crate::tauri_app) fn dispatch_background_request(
    request: &HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
) {
    match BackgroundHubAction::from_request(request) {
        Some(BackgroundHubAction::BuildProject) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_editor_runtime_build,
            HubRuntimeSession::complete_background_editor_runtime_build,
        ),
        Some(BackgroundHubAction::PackageProject) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_project_package,
            HubRuntimeSession::complete_background_project_package,
        ),
        Some(BackgroundHubAction::InstallDevice) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_device_install,
            HubRuntimeSession::complete_background_device_install,
        ),
        Some(BackgroundHubAction::OpenEditor) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_editor_launch,
            HubRuntimeSession::complete_background_editor_launch,
        ),
        None => {
            let mut session = lock_session(session_handle);
            let view_model = match session.apply_action(request.clone()) {
                Ok(view_model) => view_model,
                Err(error) => {
                    let _ = session.record_background_action_error(request, error);
                    session.view_model()
                }
            };
            drop(session);
            emit_state(&view_model);
        }
    }
}

pub(in crate::tauri_app) fn run_background_worker_loop(
    first_request: HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
) {
    let mut request = first_request;
    loop {
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            dispatch_background_request(&request, session_handle, emit_state);
        }));
        if let Err(payload) = outcome {
            let detail = panic_detail(payload.as_ref());
            eprintln!(
                "zircon_hub: background worker panicked while running {}: {detail}",
                request.action_id
            );
            let mut session = lock_session(session_handle);
            session.record_background_worker_panic(&request, &detail);
            let view_model = session.view_model();
            drop(session);
            emit_state(&view_model);
        }

        let next_request = lock_session(session_handle).take_next_background_action();
        match next_request {
            Some(next_request) => request = next_request,
            None => return,
        }
    }
}

fn panic_detail(payload: &(dyn Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackgroundHubAction {
    BuildProject,
    PackageProject,
    InstallDevice,
    OpenEditor,
}

impl BackgroundHubAction {
    fn from_request(request: &HubActionRequest) -> Option<Self> {
        HubActionId::from_str(&request.action_id).and_then(Self::from_action_id)
    }

    fn from_action_id(action: HubActionId) -> Option<Self> {
        match action {
            HubActionId::BuildProject => Some(Self::BuildProject),
            HubActionId::PackageProject => Some(Self::PackageProject),
            HubActionId::InstallDevice => Some(Self::InstallDevice),
            HubActionId::OpenEditor => Some(Self::OpenEditor),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::BuildProject => "Building",
            Self::PackageProject => "Packaging",
            Self::InstallDevice => "Installing",
            Self::OpenEditor => "Opening Editor",
        }
    }

    fn detail(self) -> HubMessage {
        match self {
            Self::BuildProject => {
                HubMessage::new(HubMessageId::Engine(EngineMessageId::RunningBuildScript))
            }
            Self::PackageProject => HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::CopyingProjectToPackage,
            )),
            Self::InstallDevice => HubMessage::new(HubMessageId::Delivery(
                DeliveryMessageId::PreparingPackageInstall,
            )),
            Self::OpenEditor => HubMessage::new(HubMessageId::Process(
                ProcessMessageId::LaunchingEditorProcess,
            )),
        }
    }

    fn operation(self) -> TaskOperationKind {
        match self {
            Self::BuildProject => TaskOperationKind::Build,
            Self::PackageProject | Self::InstallDevice => TaskOperationKind::Project,
            Self::OpenEditor => TaskOperationKind::Process,
        }
    }

    fn fallback_target(self) -> &'static str {
        match self {
            Self::BuildProject => "Source Engine",
            Self::PackageProject => "Project",
            Self::InstallDevice => "Device Install",
            Self::OpenEditor => "Editor",
        }
    }
}

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn should_run_action_in_background(
        request: &HubActionRequest,
    ) -> bool {
        BackgroundHubAction::from_request(request).is_some()
    }

    pub(in crate::tauri_app) fn start_background_action_status(
        &mut self,
        request: &HubActionRequest,
    ) -> Result<(), HubError> {
        let Some(action) = BackgroundHubAction::from_request(request) else {
            return Ok(());
        };
        let target = self.background_action_target(action, request);
        self.background_task_counter += 1;
        self.task_status = TaskStatus::running_operation(
            action.label(),
            action.detail(),
            action.operation(),
            target,
        )
        .with_task_id(self.background_task_counter);
        self.persist(None)
    }

    pub(in crate::tauri_app) fn start_background_action_or_record_error(
        &mut self,
        request: &HubActionRequest,
    ) -> Result<bool, HubError> {
        if BackgroundHubAction::from_request(request).is_none() {
            return Ok(false);
        }

        if self.background_worker_active {
            self.background_action_queue.push_back(request.clone());
            return Ok(false);
        }

        if let Err(error) = self.apply_request_project_target(request) {
            self.record_background_action_error(request, error)?;
            return Ok(false);
        }

        if let Err(error) = self.start_background_action_status(request) {
            self.record_background_action_error(request, error)?;
            return Ok(false);
        }

        self.background_worker_active = true;
        Ok(true)
    }

    pub(in crate::tauri_app) fn take_next_background_action(&mut self) -> Option<HubActionRequest> {
        let next_request = self.background_action_queue.pop_front();
        self.background_worker_active = next_request.is_some();
        next_request
    }

    pub(in crate::tauri_app) fn mark_background_action_prepared(&mut self) {
        if self.task_status.running {
            self.task_status
                .set_progress_percent(TASK_PROGRESS_PREPARED_PERCENT);
        }
    }

    pub(in crate::tauri_app) fn record_background_action_error(
        &mut self,
        request: &HubActionRequest,
        error: HubError,
    ) -> Result<(), HubError> {
        let action = BackgroundHubAction::from_request(request);
        let target = action
            .map(|action| self.background_action_target(action, request))
            .unwrap_or_else(|| {
                request
                    .target_id
                    .clone()
                    .unwrap_or_else(|| "Hub action".to_string())
            });
        let label = action
            .map(|action| format!("{} failed", action.label()))
            .unwrap_or_else(|| "Action failed".to_string());
        let operation = action
            .map(BackgroundHubAction::operation)
            .unwrap_or(TaskOperationKind::Hub);
        let (detail, recovery) = error.into_status_messages();
        self.task_status = TaskStatus::error(
            label,
            detail,
            recovery.unwrap_or_else(|| {
                HubMessage::new(HubMessageId::Shell(ShellMessageId::ReviewActionTarget))
            }),
        )
        .with_operation(operation, target)
        .with_task_id(self.task_status.task_id);
        self.persist(None)
    }

    pub(in crate::tauri_app) fn record_background_worker_panic(
        &mut self,
        request: &HubActionRequest,
        detail: &str,
    ) {
        let _ = self.record_background_action_error(
            request,
            HubError::status(
                HubMessage::with_params(
                    HubMessageId::Shell(ShellMessageId::BackgroundTaskPanicked),
                    [detail],
                ),
                Some(HubMessage::new(HubMessageId::Shell(
                    ShellMessageId::ReviewActionTarget,
                ))),
            ),
        );
    }

    fn background_action_target(
        &self,
        action: BackgroundHubAction,
        request: &HubActionRequest,
    ) -> String {
        if let Some(target) = self.project_target_label_from_request(request) {
            return target;
        }

        match action {
            BackgroundHubAction::BuildProject => active_source_engine(
                &self.config.engines,
                self.config.active_engine_id.as_deref(),
            )
            .map(|engine| engine.display_name.clone())
            .or_else(|| path_label(self.config.settings.default_source_dir.as_os_str()))
            .unwrap_or_else(|| action.fallback_target().to_string()),
            BackgroundHubAction::PackageProject | BackgroundHubAction::InstallDevice => self
                .selected_project_label_for_status()
                .unwrap_or_else(|| action.fallback_target().to_string()),
            BackgroundHubAction::OpenEditor => self
                .selected_project_label_for_status()
                .unwrap_or_else(|| action.fallback_target().to_string()),
        }
    }

    fn selected_project_label_for_status(&self) -> Option<String> {
        self.selected_project_path
            .as_ref()
            .and_then(|path| {
                self.config
                    .recent_projects
                    .iter()
                    .find(|project| crate::projects::project_paths_match(&project.path, path))
            })
            .map(recent_project_display_name)
            .or_else(|| {
                self.selected_project_path
                    .as_ref()
                    .and_then(|path| path_label(path.as_os_str()))
            })
            .or_else(|| {
                self.config
                    .recent_projects
                    .iter()
                    .max_by_key(|project: &&RecentProject| project.last_opened_unix_ms)
                    .map(recent_project_display_name)
            })
    }
}

fn path_label(path: &std::ffi::OsStr) -> Option<String> {
    (!path.is_empty()).then(|| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::{
        error::HubError,
        projects::RecentProject,
        settings::HubConfig,
        state::{
            TaskOperationKind, TaskSeverity, TASK_PROGRESS_PREPARED_PERCENT,
            TASK_PROGRESS_STARTED_PERCENT,
        },
    };

    use super::super::HubRuntimeSession;
    use crate::tauri_app::HubActionRequest;

    #[test]
    fn background_action_status_marks_build_running_without_executing_it() {
        let temp = temp_test_dir("zircon-hub-background-build-status");
        let project = temp.join("Game");
        fs::create_dir_all(&project).unwrap();
        let mut session = session_with_project(&temp, "Game", &project);

        session
            .start_background_action_status(&HubActionRequest {
                action_id: "build-project".to_string(),
                target_id: None,
                payload: None,
            })
            .unwrap();

        assert!(session.task_status.running);
        assert_eq!(session.task_status.label, "Building");
        assert_eq!(
            session.task_status.operation,
            Some(TaskOperationKind::Build)
        );
        assert_eq!(
            session.task_status.progress_percent,
            TASK_PROGRESS_STARTED_PERCENT
        );
        assert_eq!(session.task_status.task_id, 1);
        assert_eq!(session.config.action_history.len(), 0);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_action_progress_advances_after_preparation() {
        let temp = temp_test_dir("zircon-hub-background-progress-prepare");
        let project = temp.join("Game");
        fs::create_dir_all(&project).unwrap();
        let mut session = session_with_project(&temp, "Game", &project);

        session
            .start_background_action_status(&HubActionRequest {
                action_id: "package-project".to_string(),
                target_id: None,
                payload: None,
            })
            .unwrap();
        session.mark_background_action_prepared();

        assert!(session.task_status.running);
        assert_eq!(
            session.task_status.progress_percent,
            TASK_PROGRESS_PREPARED_PERCENT
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_action_error_is_recoverable_visible_status() {
        let temp = temp_test_dir("zircon-hub-background-error-status");
        let project = temp.join("Game");
        fs::create_dir_all(&project).unwrap();
        let mut session = session_with_project(&temp, "Game", &project);

        session
            .record_background_action_error(
                &HubActionRequest {
                    action_id: "package-project".to_string(),
                    target_id: None,
                    payload: None,
                },
                HubError::message("worker failed"),
            )
            .unwrap();

        assert!(!session.task_status.running);
        assert_eq!(session.task_status.severity, TaskSeverity::Error);
        assert_eq!(session.task_status.label, "Packaging failed");
        assert_eq!(session.task_status.target.as_deref(), Some("Game"));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_action_status_uses_explicit_project_target_label() {
        let temp = temp_test_dir("zircon-hub-background-target-status");
        let selected = temp.join("Selected");
        let target = temp.join("Target");
        fs::create_dir_all(&selected).unwrap();
        fs::create_dir_all(&target).unwrap();
        let mut session = session_with_projects(
            &temp,
            &[("Selected", selected.clone()), ("Target", target.clone())],
            &selected,
        );
        let request = HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some(target.to_string_lossy().into_owned()),
            payload: None,
        };

        session.apply_request_project_target(&request).unwrap();
        session.start_background_action_status(&request).unwrap();

        assert_eq!(
            session.selected_project_path.as_deref(),
            Some(target.as_path())
        );
        assert_eq!(session.task_status.target.as_deref(), Some("Target"));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_action_start_records_invalid_project_target_without_spawning() {
        let temp = temp_test_dir("zircon-hub-background-invalid-target");
        let selected = temp.join("Selected");
        fs::create_dir_all(&selected).unwrap();
        let mut session = session_with_project(&temp, "Selected", &selected);
        let request = HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some("missing-project".to_string()),
            payload: None,
        };

        let should_spawn = session
            .start_background_action_or_record_error(&request)
            .expect("invalid background action target should become visible Hub state");

        assert!(!should_spawn);
        assert!(!session.background_worker_active);
        assert!(session.background_action_queue.is_empty());
        assert!(!session.task_status.running);
        assert_eq!(session.task_status.severity, TaskSeverity::Error);
        assert_eq!(session.task_status.label, "Packaging failed");
        assert_eq!(
            session.task_status.detail,
            "Unknown recent project target for package-project: missing-project"
        );
        assert_eq!(
            session.selected_project_path.as_deref(),
            Some(selected.as_path())
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_actions_queue_while_worker_is_active_and_dequeue_fifo() {
        let temp = temp_test_dir("zircon-hub-background-action-queue");
        let project = temp.join("Game");
        fs::create_dir_all(&project).unwrap();
        let mut session = session_with_project(&temp, "Game", &project);
        let build_request = background_request("build-project");
        let package_request = background_request("package-project");
        let install_request = background_request("install-device");

        let should_spawn_first = session
            .start_background_action_or_record_error(&build_request)
            .expect("first background action should start the worker");
        let running_status = session.task_status.clone();
        let should_spawn_second = session
            .start_background_action_or_record_error(&package_request)
            .expect("second background action should be queued");
        let should_spawn_third = session
            .start_background_action_or_record_error(&install_request)
            .expect("third background action should be queued");

        assert!(should_spawn_first);
        assert!(!should_spawn_second);
        assert!(!should_spawn_third);
        assert!(session.background_worker_active);
        assert_eq!(session.background_action_queue.len(), 2);
        assert_eq!(
            session.task_status, running_status,
            "queued actions must not overwrite the currently running status"
        );
        let model = session.view_model();
        assert_eq!(model.task_summary.task_id, running_status.task_id);
        assert_eq!(model.task_summary.queued, 2);

        let next_request = session
            .take_next_background_action()
            .expect("package action should be next");
        assert_eq!(next_request.action_id, "package-project");
        assert!(session.background_worker_active);

        let next_request = session
            .take_next_background_action()
            .expect("install action should follow package action");
        assert_eq!(next_request.action_id, "install-device");
        assert!(session.background_worker_active);

        assert!(session.take_next_background_action().is_none());
        assert!(!session.background_worker_active);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_action_start_localizes_invalid_project_target_failure() {
        let temp = temp_test_dir("zircon-hub-background-invalid-target-localized");
        let selected = temp.join("Selected");
        fs::create_dir_all(&selected).unwrap();
        let mut session = session_with_project(&temp, "Selected", &selected);
        session.config.settings.language = crate::settings::HubLanguage::Chinese;
        let request = HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some("missing-project".to_string()),
            payload: None,
        };

        let should_spawn = session
            .start_background_action_or_record_error(&request)
            .expect("invalid background action target should return a localized ViewModel");

        let model = session.view_model();
        assert!(!should_spawn);
        assert_eq!(model.task_summary.label, "打包失败");
        assert_eq!(
            model.task_summary.detail,
            "未知最近项目目标（package-project）：missing-project"
        );
        assert_eq!(
            model.task_summary.recovery.as_deref(),
            Some("检查操作目标后重试")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    fn session_with_project(
        temp: &std::path::Path,
        name: &str,
        project: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_build_output_dir = temp.join("out");
        config.recent_projects = vec![RecentProject::new(name, project, 1)];
        config.runtime.selected_project_path = Some(project.to_path_buf());
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
    }

    fn session_with_projects(
        temp: &std::path::Path,
        projects: &[(&str, PathBuf)],
        selected_project: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_build_output_dir = temp.join("out");
        config.recent_projects = projects
            .iter()
            .map(|(name, path)| RecentProject::new(*name, path, 1))
            .collect();
        config.runtime.selected_project_path = Some(selected_project.to_path_buf());
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
    }

    fn background_request(action_id: &str) -> HubActionRequest {
        HubActionRequest {
            action_id: action_id.to_string(),
            target_id: None,
            payload: None,
        }
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
}
