use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::engines::active_source_engine;
use crate::error::HubError;
use crate::process::EditorChildReaper;
use crate::projects::RecentProject;
use crate::state::{
    DeliveryMessageId, EngineMessageId, HubActionKind, HubActionRecord, HubActionStatus,
    HubMessage, HubMessageId, ProcessMessageId, ProjectMessageId, ShellMessageId,
    TaskCancellationToken, TaskExecutionOutcome, TaskOperationKind, TaskStatus,
    TASK_PROGRESS_PREPARED_PERCENT,
};

use super::{recent_project_display_name, HubRuntimeSession};
use crate::tauri_app::action_id::HubActionId;
use crate::tauri_app::HubActionRequest;
use crate::tauri_app::HubViewModel;

mod queue_admission;

use queue_admission::enqueue_background_action;

#[derive(Default)]
pub(in crate::tauri_app) struct EditorLaunchOwner {
    active: Mutex<Option<TaskCancellationToken>>,
}

impl EditorLaunchOwner {
    pub(in crate::tauri_app) fn admit(&self, cancellation: &TaskCancellationToken) {
        *self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(cancellation.clone());
    }

    fn finish(&self, task_id: u64) {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if active
            .as_ref()
            .is_some_and(|token| token.task_id() == task_id)
        {
            *active = None;
        }
    }

    pub(in crate::tauri_app) fn request_shutdown(&self) -> Option<u64> {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let cancellation = active.as_ref()?;
        cancellation.request_cancellation();
        Some(cancellation.task_id())
    }
}

pub(in crate::tauri_app) trait BackgroundTask: Send + 'static {
    type Output: Send + 'static;

    fn run(
        &self,
        context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<Self::Output>, HubError>;
}

#[derive(Clone)]
pub(in crate::tauri_app) struct BackgroundTaskContext {
    cancellation: TaskCancellationToken,
    editor_child_reaper: EditorChildReaper,
}

impl BackgroundTaskContext {
    pub(in crate::tauri_app) fn cancellation(&self) -> &TaskCancellationToken {
        &self.cancellation
    }

    pub(in crate::tauri_app) fn editor_child_reaper(&self) -> &EditorChildReaper {
        &self.editor_child_reaper
    }

    #[cfg(test)]
    pub(in crate::tauri_app) fn for_test(task_id: u64) -> Self {
        Self {
            cancellation: TaskCancellationToken::new(task_id),
            editor_child_reaper: EditorChildReaper::start()
                .expect("start test Editor child reaper"),
        }
    }
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
    editor_child_reaper: &EditorChildReaper,
    prepare: fn(&mut HubRuntimeSession) -> Result<Option<T>, HubError>,
    complete: fn(&mut HubRuntimeSession, T, Result<T::Output, HubError>) -> Result<(), HubError>,
) {
    execute_background_task_with_request(
        request,
        session_handle,
        emit_state,
        editor_child_reaper,
        |_, session| prepare(session),
        complete,
    );
}

fn execute_background_task_with_request<T, P>(
    request: &HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
    editor_child_reaper: &EditorChildReaper,
    prepare: P,
    complete: fn(&mut HubRuntimeSession, T, Result<T::Output, HubError>) -> Result<(), HubError>,
) where
    T: BackgroundTask,
    P: FnOnce(&HubActionRequest, &mut HubRuntimeSession) -> Result<Option<T>, HubError>,
{
    let (pending, task_id, cancellation) = {
        let mut session = lock_session(session_handle);
        let Some(task_id) = session.active_background_task_id() else {
            let _ = session.record_background_action_error(
                request,
                HubError::message("background task is missing its task-bound control state"),
            );
            let view_model = session.publish_view_model();
            drop(session);
            emit_state(&view_model);
            return;
        };
        let background_action = BackgroundHubAction::from_request(request);
        if background_action.is_some_and(BackgroundHubAction::uses_existing_project_target) {
            if let Err(error) = session.apply_request_project_target(request) {
                let _ = session.record_background_action_error(request, error);
                let view_model = session.publish_view_model();
                drop(session);
                emit_state(&view_model);
                return;
            }
        }
        let pending = match prepare(request, &mut session) {
            Ok(pending) => pending,
            Err(error) => {
                let _ = session.record_background_action_error(request, error);
                let view_model = session.publish_view_model();
                drop(session);
                emit_state(&view_model);
                return;
            }
        };
        let Some(pending) = pending else {
            session.finish_background_task(task_id);
            session.task_status.task_id = task_id;
            let view_model = session.publish_view_model();
            drop(session);
            emit_state(&view_model);
            return;
        };
        let cancellation = match session.background_cancellation_for_task(task_id) {
            Some(cancellation) => cancellation,
            None => {
                let _ = session.record_background_action_error(
                    request,
                    HubError::message(
                        "background task is missing its task-bound cancellation token",
                    ),
                );
                let view_model = session.publish_view_model();
                drop(session);
                emit_state(&view_model);
                return;
            }
        };
        let view_model = session.publish_view_model();
        drop(session);
        emit_state(&view_model);
        (pending, task_id, cancellation)
    };

    let context = BackgroundTaskContext {
        cancellation,
        editor_child_reaper: editor_child_reaper.clone(),
    };
    let result = pending.run(&context);
    let mut session = lock_session(session_handle);
    session.finish_background_task(task_id);
    let completion = match result {
        Ok(TaskExecutionOutcome::Completed(output)) => complete(&mut session, pending, Ok(output)),
        Ok(TaskExecutionOutcome::Cancelled) => {
            session.record_background_action_cancelled(request, task_id)
        }
        Err(error) => complete(&mut session, pending, Err(error)),
    };
    let view_model = match completion {
        Ok(()) => {
            session.task_status.task_id = task_id;
            session.publish_view_model()
        }
        Err(error) => {
            session.task_status.task_id = task_id;
            let _ = session.record_background_action_error(request, error);
            session.task_status.task_id = task_id;
            session.publish_view_model()
        }
    };
    drop(session);
    emit_state(&view_model);
}

pub(in crate::tauri_app) fn dispatch_background_request(
    request: &HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
    editor_child_reaper: &EditorChildReaper,
) {
    match BackgroundHubAction::from_request(request) {
        Some(BackgroundHubAction::CreateProject) => execute_background_task_with_request(
            request,
            session_handle,
            emit_state,
            editor_child_reaper,
            |request, session| session.prepare_background_project_creation(request),
            HubRuntimeSession::complete_background_project_creation,
        ),
        Some(BackgroundHubAction::BuildProject) => execute_background_task(
            request,
            session_handle,
            emit_state,
            editor_child_reaper,
            HubRuntimeSession::prepare_background_editor_runtime_build,
            HubRuntimeSession::complete_background_editor_runtime_build,
        ),
        Some(BackgroundHubAction::PackageProject) => execute_background_task(
            request,
            session_handle,
            emit_state,
            editor_child_reaper,
            HubRuntimeSession::prepare_background_project_package,
            HubRuntimeSession::complete_background_project_package,
        ),
        Some(BackgroundHubAction::InstallDevice) => execute_background_task(
            request,
            session_handle,
            emit_state,
            editor_child_reaper,
            HubRuntimeSession::prepare_background_device_install,
            HubRuntimeSession::complete_background_device_install,
        ),
        Some(BackgroundHubAction::OpenEditor) => execute_background_task(
            request,
            session_handle,
            emit_state,
            editor_child_reaper,
            HubRuntimeSession::prepare_background_editor_launch,
            HubRuntimeSession::complete_background_editor_launch,
        ),
        None => {
            let mut session = lock_session(session_handle);
            let view_model = match session.apply_action(request.clone()) {
                Ok(view_model) => view_model,
                Err(error) => {
                    let _ = session.record_background_action_error(request, error);
                    session.publish_view_model()
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
    editor_child_reaper: &EditorChildReaper,
    editor_launch_admission_closed: &AtomicBool,
) {
    let mut request = first_request;
    loop {
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            if editor_launch_admission_closed.load(Ordering::Acquire)
                && matches!(
                    BackgroundHubAction::from_request(&request),
                    Some(BackgroundHubAction::OpenEditor | BackgroundHubAction::CreateProject)
                )
            {
                let mut session = lock_session(session_handle);
                if let Some(task_id) = session.active_background_task_id() {
                    if let Err(error) =
                        session.record_background_action_cancelled(&request, task_id)
                    {
                        let _ = session.record_background_action_error(&request, error);
                    }
                }
                let view_model = session.publish_view_model();
                drop(session);
                emit_state(&view_model);
            } else {
                dispatch_background_request(
                    &request,
                    session_handle,
                    emit_state,
                    editor_child_reaper,
                );
            }
        }));
        if let Err(payload) = outcome {
            let detail = panic_detail(payload.as_ref());
            eprintln!(
                "zircon_hub: background worker panicked while running {}: {detail}",
                request.action_id
            );
            let mut session = lock_session(session_handle);
            session.record_background_worker_panic(&request, &detail);
            let view_model = session.publish_view_model();
            drop(session);
            emit_state(&view_model);
        }

        if editor_launch_admission_closed.load(Ordering::Acquire)
            && matches!(
                BackgroundHubAction::from_request(&request),
                Some(BackgroundHubAction::OpenEditor | BackgroundHubAction::CreateProject)
            )
        {
            return;
        }

        let (next_request, started_view_model) = {
            let mut session = lock_session(session_handle);
            let next_request = session.take_next_background_action();
            let started_view_model = next_request.as_ref().map(|_| session.publish_view_model());
            (next_request, started_view_model)
        };
        if let Some(view_model) = started_view_model {
            emit_state(&view_model);
        }
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
    CreateProject,
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
            HubActionId::CreateProject => Some(Self::CreateProject),
            HubActionId::BuildProject => Some(Self::BuildProject),
            HubActionId::PackageProject => Some(Self::PackageProject),
            HubActionId::InstallDevice => Some(Self::InstallDevice),
            HubActionId::OpenEditor => Some(Self::OpenEditor),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::CreateProject => "Creating Project",
            Self::BuildProject => "Building",
            Self::PackageProject => "Packaging",
            Self::InstallDevice => "Installing",
            Self::OpenEditor => "Opening Editor",
        }
    }

    fn detail(self) -> HubMessage {
        match self {
            Self::CreateProject => HubMessage::new(HubMessageId::Process(
                ProcessMessageId::LaunchingEditorProcess,
            )),
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
            Self::CreateProject => TaskOperationKind::Project,
            Self::BuildProject => TaskOperationKind::Build,
            Self::PackageProject | Self::InstallDevice => TaskOperationKind::Project,
            Self::OpenEditor => TaskOperationKind::Process,
        }
    }

    fn fallback_target(self) -> &'static str {
        match self {
            Self::CreateProject => "Project",
            Self::BuildProject => "Source Engine",
            Self::PackageProject => "Project",
            Self::InstallDevice => "Device Install",
            Self::OpenEditor => "Editor",
        }
    }

    fn uses_existing_project_target(self) -> bool {
        !matches!(self, Self::CreateProject)
    }

    fn action_kind(self) -> HubActionKind {
        match self {
            Self::CreateProject => HubActionKind::CreateProject,
            Self::BuildProject => HubActionKind::BuildEditorRuntime,
            Self::PackageProject => HubActionKind::PackageProject,
            Self::InstallDevice => HubActionKind::InstallProject,
            Self::OpenEditor => HubActionKind::OpenEditor,
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
        if self.set_background_action_status(request)? {
            self.persist()
        } else {
            Ok(())
        }
    }

    fn set_background_action_status(
        &mut self,
        request: &HubActionRequest,
    ) -> Result<bool, HubError> {
        let Some(action) = BackgroundHubAction::from_request(request) else {
            return Ok(false);
        };
        let target = self.background_action_target(action, request);
        self.background_task_counter = self
            .background_task_counter
            .checked_add(1)
            .ok_or_else(|| HubError::message("background task identity sequence is exhausted"))?;
        let task_id = self.background_task_counter;
        let cancellation = TaskCancellationToken::new(task_id);
        let status = TaskStatus::running_operation(
            action.label(),
            action.detail(),
            action.operation(),
            target,
        )
        .with_cancellable()
        .with_task_id(task_id);
        self.active_background_task = Some(super::ActiveBackgroundTask {
            cancellation: cancellation.clone(),
            status: status.clone(),
        });
        self.active_editor_launch_task_id = matches!(
            action,
            BackgroundHubAction::OpenEditor | BackgroundHubAction::CreateProject
        )
        .then_some(task_id);
        if self.active_editor_launch_task_id.is_some() {
            self.editor_launch_owner.admit(&cancellation);
            self.editor_process_ledger.admit_editor_attempt(
                task_id,
                status
                    .target
                    .clone()
                    .unwrap_or_else(|| "Editor".to_string()),
            );
        }
        self.task_status = status;
        Ok(true)
    }

    pub(in crate::tauri_app) fn start_background_action_or_record_error(
        &mut self,
        request: &HubActionRequest,
    ) -> Result<bool, HubError> {
        let Some(action) = BackgroundHubAction::from_request(request) else {
            return Ok(false);
        };

        if self.background_worker_active {
            enqueue_background_action(&mut self.background_action_queue, request)?;
            return Ok(false);
        }

        if action.uses_existing_project_target() {
            if let Err(error) = self.apply_request_project_target(request) {
                self.record_background_action_error(request, error)?;
                return Ok(false);
            }
        }

        if matches!(action, BackgroundHubAction::CreateProject) {
            if let Err(error) = request.parse_as(HubActionId::CreateProject) {
                self.record_background_action_error(request, error)?;
                return Ok(false);
            }
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
        self.active_background_task = None;
        if let Some(task_id) = self.active_editor_launch_task_id {
            self.editor_launch_owner.finish(task_id);
        }
        self.active_editor_launch_task_id = None;
        if let Some(request) = next_request.as_ref() {
            if let Err(error) = self.set_background_action_status(request) {
                let _ = self.record_background_action_error(request, error);
                self.background_worker_active = false;
                self.background_action_queue.clear();
                return None;
            }
        }
        next_request
    }

    pub(in crate::tauri_app) fn request_background_task_cancellation(&mut self, task_id: u64) {
        let Some(active_task) = self.active_background_task.as_mut() else {
            return;
        };
        if active_task.cancellation.task_id() != task_id || !active_task.status.running {
            return;
        }

        active_task.cancellation.request_cancellation();
        active_task.status.cancellable = false;
        active_task.status.detail = HubMessage::new(HubMessageId::Shell(
            ShellMessageId::TaskCancellationRequested,
        ));
    }

    pub(in crate::tauri_app) fn request_editor_launch_shutdown(&mut self) -> Option<u64> {
        self.background_action_queue.retain(|request| {
            !matches!(
                BackgroundHubAction::from_request(request),
                Some(BackgroundHubAction::OpenEditor | BackgroundHubAction::CreateProject)
            )
        });
        let task_id = self.editor_launch_owner.request_shutdown()?;
        let active = self.active_background_task.as_mut()?;
        debug_assert_eq!(active.cancellation.task_id(), task_id);
        active.cancellation.request_cancellation();
        Some(task_id)
    }

    pub(in crate::tauri_app) fn editor_launch_task_is_active(&self, task_id: u64) -> bool {
        self.active_editor_launch_task_id == Some(task_id)
    }

    pub(in crate::tauri_app) fn active_background_task_id(&self) -> Option<u64> {
        self.active_background_task
            .as_ref()
            .map(|task| task.cancellation.task_id())
    }

    fn background_cancellation_for_task(&self, task_id: u64) -> Option<TaskCancellationToken> {
        self.active_background_task
            .as_ref()
            .filter(|task| task.cancellation.task_id() == task_id)
            .map(|task| task.cancellation.clone())
    }

    pub(in crate::tauri_app) fn finish_background_task(&mut self, task_id: u64) {
        if self.active_background_task_id() == Some(task_id) {
            self.editor_launch_owner.finish(task_id);
            self.active_background_task = None;
            self.active_editor_launch_task_id = None;
            self.task_status.task_id = task_id;
        }
    }

    pub(in crate::tauri_app) fn editor_launch_owner(&self) -> Arc<EditorLaunchOwner> {
        Arc::clone(&self.editor_launch_owner)
    }

    pub(in crate::tauri_app) fn mark_background_action_prepared(&mut self) {
        if let Some(active_task) = self.active_background_task.as_mut() {
            active_task
                .status
                .set_progress_percent(TASK_PROGRESS_PREPARED_PERCENT);
        }
        if self.task_status.running
            && self.active_background_task_id() == Some(self.task_status.task_id)
        {
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
        let task_id = self
            .active_background_task_id()
            .unwrap_or(self.task_status.task_id);
        self.finish_background_task(task_id);
        let (detail, recovery) = error.into_status_messages();
        self.task_status = TaskStatus::error(
            label,
            detail,
            recovery.unwrap_or_else(|| {
                HubMessage::new(HubMessageId::Shell(ShellMessageId::ReviewActionTarget))
            }),
        )
        .with_operation(operation, target)
        .with_task_id(task_id);
        self.persist()
    }

    fn record_background_action_cancelled(
        &mut self,
        request: &HubActionRequest,
        task_id: u64,
    ) -> Result<(), HubError> {
        let action = BackgroundHubAction::from_request(request).ok_or_else(|| {
            HubError::message("cancelled background request is not a background action")
        })?;
        self.finish_background_task(task_id);
        let target = self.background_action_target(action, request);
        let detail = HubMessage::new(HubMessageId::Shell(ShellMessageId::TaskCancelled));
        let (recovery, output_dir) = cancelled_action_recovery(action, request);
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: action.action_kind(),
            status: HubActionStatus::Cancelled,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: HubMessage::empty(),
            recovery: recovery.clone(),
            process_id: None,
            command_line: Vec::new(),
            output_dir,
        })?;
        self.task_status = TaskStatus::cancelled("Task cancelled", detail, recovery)
            .with_operation(action.operation(), target)
            .with_task_id(task_id);
        Ok(())
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
            BackgroundHubAction::CreateProject => request
                .parse_as(HubActionId::CreateProject)
                .ok()
                .and_then(|action| match action {
                    crate::tauri_app::action_request::HubAction::CreateProject { payload } => {
                        Some(payload.name)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| action.fallback_target().to_string()),
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

fn cancelled_action_recovery(
    action: BackgroundHubAction,
    request: &HubActionRequest,
) -> (Option<HubMessage>, Option<std::path::PathBuf>) {
    if action != BackgroundHubAction::CreateProject {
        return (None, None);
    }
    let Ok(crate::tauri_app::action_request::HubAction::CreateProject { payload }) =
        request.parse_as(HubActionId::CreateProject)
    else {
        return (None, None);
    };
    let project_root = payload.location.join(payload.name);
    if project_root.exists() {
        (
            Some(HubMessage::new(HubMessageId::Project(
                ProjectMessageId::KeptFolderUseImport,
            ))),
            Some(project_root),
        )
    } else {
        (None, None)
    }
}

fn path_label(path: &std::ffi::OsStr) -> Option<String> {
    (!path.is_empty()).then(|| path.to_string_lossy().into_owned())
}

#[cfg(test)]
#[path = "action_tasks/tests/cases.rs"]
mod tests;
