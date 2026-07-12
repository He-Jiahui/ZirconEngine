use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::{AppHandle, Manager, Runtime};

use crate::coordinator_client::CoordinatorClient;
use crate::lifecycle::{self, LifecycleAction};
use crate::menu::{menu_model, MenuCommand};
use crate::notifications::{self, NotificationPolicy, TrayNotification};
use crate::process_identity::inspect_process;
use crate::recovery::{RecoveryController, RecoveryDecision, RecoveryGuard, RecoveryStatus};
use crate::repository_identity::{RepositoryIdentity, RepositoryMutex};
use crate::runtime_descriptor::RuntimeDescriptor;
use crate::startup::{self, StartupAction, StartupManagementResult, StartupPreview};
use crate::tray_state::{MenuEnablement, SupervisionState, TrayVisualState};
use crate::TrayError;

const TRAY_ID: &str = "zircon-session-tray";

pub struct TrayContext {
    repo_root: PathBuf,
    repository: RepositoryIdentity,
    _mutex: RepositoryMutex,
    notifications: Mutex<NotificationPolicy>,
    recovery: Mutex<RecoveryController>,
    recovery_sync_pending: Mutex<bool>,
    pending_action: Mutex<Option<PendingAction>>,
    pending_startup: Mutex<Option<StartupPreview>>,
    active_action: Mutex<Option<ActiveAction>>,
    last_startup: Mutex<Option<StartupManagementResult>>,
    last_error: Mutex<Option<String>>,
}

#[derive(Clone)]
struct PendingAction {
    action: LifecycleAction,
    action_id: String,
    confirmation_phrase: String,
    created_at: Instant,
}

#[derive(Clone)]
struct ActiveAction {
    action: LifecycleAction,
    action_id: String,
}

struct Observation {
    visual: TrayVisualState,
    identity_verified: bool,
    supervision_state: SupervisionState,
    explicit_stop: bool,
    maintenance_hold: bool,
    persisted_failure_count: u32,
}

pub fn run() -> Result<(), TrayError> {
    let repo_root = resolve_repo_root()?;
    let repository = RepositoryIdentity::for_path(&repo_root)?;
    let repository_mutex = RepositoryMutex::acquire(&repository)?;
    tauri::Builder::default()
        .setup(move |app| {
            let recovery = RecoveryController::load(&recovery_path(&repo_root))?;
            app.manage(TrayContext {
                repo_root,
                repository,
                _mutex: repository_mutex,
                notifications: Mutex::new(NotificationPolicy::default()),
                recovery: Mutex::new(recovery),
                recovery_sync_pending: Mutex::new(true),
                pending_action: Mutex::new(None),
                pending_startup: Mutex::new(None),
                active_action: Mutex::new(None),
                last_startup: Mutex::new(None),
                last_error: Mutex::new(None),
            });
            let handle = app.handle().clone();
            refresh_tray(&handle)?;
            thread::spawn(move || supervise_loop(handle));
            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref().to_owned();
            if id == MenuCommand::ExitTray.id() {
                app.exit(0);
                return;
            }
            let handle = app.clone();
            thread::spawn(move || {
                if let Err(error) = handle_menu(&handle, &id) {
                    record_error(&handle, error);
                } else {
                    let context = handle.state::<TrayContext>();
                    if let Ok(mut last_error) = context.last_error.lock() {
                        *last_error = None;
                    };
                }
                let _ = refresh_tray(&handle);
            });
        })
        .run(tauri::generate_context!())
        .map_err(TrayError::Tauri)
}

fn resolve_repo_root() -> Result<PathBuf, TrayError> {
    let arguments: Vec<_> = std::env::args_os().collect();
    if let Some(index) = arguments.iter().position(|value| value == "--repo-root") {
        if let Some(path) = arguments.get(index + 1) {
            return Ok(PathBuf::from(path).canonicalize()?);
        }
    }
    let mut candidate = std::env::current_dir()?;
    loop {
        if candidate.join("tools").join("zircon-session.ps1").is_file() {
            return Ok(candidate.canonicalize()?);
        }
        if !candidate.pop() {
            return Err(TrayError::InvalidDescriptor(
                "repository root must be supplied with --repo-root",
            ));
        }
    }
}

fn runtime_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(".codex")
        .join("state")
        .join("session-coordinator")
        .join("runtime.json")
}

fn startup_failure_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(".codex")
        .join("state")
        .join("session-coordinator")
        .join("startup-failure.json")
}

fn recovery_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join(".codex")
        .join("state")
        .join("session-coordinator")
        .join("tray-recovery.json")
}

fn observe(context: &TrayContext) -> Result<Observation, TrayError> {
    let path = runtime_path(&context.repo_root);
    if !path.is_file() {
        if startup_failure_path(&context.repo_root).is_file() {
            return Err(TrayError::RecoverySuppressed(
                "coordinator startup failed integrity checks",
            ));
        }
        return Err(TrayError::Http(
            "coordinator runtime descriptor is unavailable".into(),
        ));
    }
    let descriptor = RuntimeDescriptor::read(path)?;
    descriptor.validate(&context.repository)?;
    inspect_process(descriptor.pid)?.verify(&descriptor)?;
    let health = CoordinatorClient::new(&descriptor).verify_health()?;
    Ok(Observation {
        visual: TrayVisualState::from_health(&health),
        identity_verified: true,
        supervision_state: health.supervision.state,
        explicit_stop: health.supervision.explicit_stop,
        maintenance_hold: health.supervision.maintenance_hold,
        persisted_failure_count: health.supervision.failure_count,
    })
}

fn supervise_loop<R: Runtime>(app: AppHandle<R>) {
    loop {
        thread::sleep(Duration::from_secs(2));
        let context = app.state::<TrayContext>();
        match observe(&context) {
            Ok(observation) => {
                let recovery_update = context.recovery.lock().ok().map(|mut recovery| {
                    let changed = recovery.observe_online(
                        unix_seconds(),
                        RecoveryGuard {
                            state: observation.supervision_state,
                            explicit_stop: observation.explicit_stop,
                            maintenance_hold: observation.maintenance_hold,
                            valid_competing_instance: false,
                        },
                    );
                    (changed, recovery.status())
                });
                if let Some((changed, status)) = recovery_update {
                    if changed {
                        if let Err(error) = save_recovery(&context) {
                            record_error(&app, error);
                        }
                    }
                    let sync_pending = context
                        .recovery_sync_pending
                        .lock()
                        .map(|value| *value)
                        .unwrap_or(true);
                    let sync_allowed = matches!(
                        observation.supervision_state,
                        SupervisionState::Healthy
                            | SupervisionState::Degraded
                            | SupervisionState::Draining
                    );
                    if sync_allowed
                        && (changed
                            || sync_pending
                            || observation.persisted_failure_count as usize != status.failure_count)
                    {
                        if let Err(error) =
                            with_verified_descriptor(&context, |_descriptor, client| {
                                client.record_recovery(status)
                            })
                        {
                            record_error(&app, error);
                        } else if let Ok(mut pending) = context.recovery_sync_pending.lock() {
                            *pending = false;
                        }
                    }
                }
                let _ = refresh_active_action(&context);
                notify_state_change(&context, observation.visual);
                let _ = render_tray(&app, observation.visual, observation.identity_verified);
            }
            Err(TrayError::IdentityMismatch(_)) | Err(TrayError::InvalidDescriptor(_)) => {
                if let Ok(mut recovery) = context.recovery.lock() {
                    let _ = recovery.observe_offline(unix_seconds(), false);
                }
                notify_state_change(&context, TrayVisualState::IdentityMismatch);
                let _ = render_tray(&app, TrayVisualState::IdentityMismatch, false);
            }
            Err(TrayError::RecoverySuppressed(_)) => {
                if let Ok(mut recovery) = context.recovery.lock() {
                    let _ = recovery.observe_offline(unix_seconds(), false);
                }
                notify_state_change(&context, TrayVisualState::FatalIntegrityError);
                let _ = render_tray(&app, TrayVisualState::FatalIntegrityError, false);
            }
            Err(_) => {
                let decision = context
                    .recovery
                    .lock()
                    .map(|mut recovery| recovery.observe_offline(unix_seconds(), true))
                    .unwrap_or(RecoveryDecision::Suppressed);
                if !matches!(decision, RecoveryDecision::Suppressed) {
                    if let Err(error) = save_recovery(&context) {
                        record_error(&app, error);
                        notify_state_change(&context, TrayVisualState::Offline);
                        let _ = render_tray(&app, TrayVisualState::Offline, true);
                        continue;
                    }
                }
                match decision {
                    RecoveryDecision::RetryAfter(delay) => {
                        notify_state_change(&context, TrayVisualState::Recovering);
                        let _ = render_tray(&app, TrayVisualState::Recovering, true);
                        thread::sleep(Duration::from_secs(delay));
                        let start_result = lifecycle::start_hidden(&context.repo_root);
                        if let Ok(mut recovery) = context.recovery.lock() {
                            recovery.retry_finished();
                            if let Err(error) = recovery.save(&recovery_path(&context.repo_root)) {
                                record_error(&app, error);
                            }
                        }
                        if let Err(error) = start_result {
                            record_error(&app, error);
                        }
                    }
                    RecoveryDecision::CircuitOpen | RecoveryDecision::Suppressed => {
                        notify_state_change(&context, TrayVisualState::Offline);
                        let _ = render_tray(&app, TrayVisualState::Offline, true);
                    }
                }
            }
        }
    }
}

fn save_recovery(context: &TrayContext) -> Result<RecoveryStatus, TrayError> {
    let recovery = context
        .recovery
        .lock()
        .map_err(|_| TrayError::Http("tray recovery state is unavailable".into()))?;
    recovery.save(&recovery_path(&context.repo_root))?;
    Ok(recovery.status())
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn refresh_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), TrayError> {
    let context = app.state::<TrayContext>();
    match observe(&context) {
        Ok(observation) => {
            notify_state_change(&context, observation.visual);
            render_tray(app, observation.visual, observation.identity_verified)
        }
        Err(TrayError::IdentityMismatch(_)) | Err(TrayError::InvalidDescriptor(_)) => {
            notify_state_change(&context, TrayVisualState::IdentityMismatch);
            render_tray(app, TrayVisualState::IdentityMismatch, false)
        }
        Err(TrayError::RecoverySuppressed(_)) => {
            notify_state_change(&context, TrayVisualState::FatalIntegrityError);
            render_tray(app, TrayVisualState::FatalIntegrityError, false)
        }
        Err(_) => {
            notify_state_change(&context, TrayVisualState::Offline);
            render_tray(app, TrayVisualState::Offline, true)
        }
    }
}

fn notify_state_change(context: &TrayContext, state: TrayVisualState) {
    let Ok(mut policy) = context.notifications.lock() else {
        return;
    };
    if let Some(notification) = policy.state_change(state.key(), state.tooltip()) {
        let _ = notifications::show_native(&notification);
    }
}

fn record_error<R: Runtime>(app: &AppHandle<R>, error: TrayError) {
    let message = error.to_string();
    let context = app.state::<TrayContext>();
    if let Ok(mut last_error) = context.last_error.lock() {
        *last_error = Some(message.clone());
    }
    let _ = notifications::show_native(&TrayNotification {
        title: "Zircon Coordinator：操作失败".into(),
        body: message,
    });
}

fn render_tray<R: Runtime>(
    app: &AppHandle<R>,
    state: TrayVisualState,
    identity_verified: bool,
) -> Result<(), TrayError> {
    let tray = app.tray_by_id(TRAY_ID).ok_or(TrayError::InvalidDescriptor(
        "configured tray icon is missing",
    ))?;
    tray.set_icon(Some(Image::new_owned(state.icon_rgba(16), 16, 16)))?;
    tray.set_tooltip(Some(state.tooltip()))?;
    let context = app.state::<TrayContext>();
    let pending = context.pending_action.lock().ok().and_then(|mut pending| {
        if pending
            .as_ref()
            .is_some_and(|value| value.created_at.elapsed() >= Duration::from_secs(120))
        {
            *pending = None;
        }
        pending.as_ref().map(|value| value.action)
    });
    let has_error = context
        .last_error
        .lock()
        .map(|value| value.is_some())
        .unwrap_or(false);
    let active = context
        .active_action
        .lock()
        .ok()
        .and_then(|value| value.as_ref().map(|active| active.action));
    let pending_startup = context
        .pending_startup
        .lock()
        .ok()
        .and_then(|value| value.as_ref().map(|preview| preview.action));
    tray.set_menu(Some(build_menu(
        app,
        state.menu(identity_verified),
        pending,
        active,
        pending_startup,
        identity_verified && matches!(state, TrayVisualState::Draining),
        has_error,
    )?))?;
    Ok(())
}

fn build_menu<R: Runtime>(
    app: &AppHandle<R>,
    enablement: MenuEnablement,
    pending: Option<LifecycleAction>,
    active: Option<LifecycleAction>,
    pending_startup: Option<StartupAction>,
    can_cancel_active: bool,
    has_error: bool,
) -> Result<Menu<R>, TrayError> {
    let entries = menu_model(
        enablement,
        pending,
        active,
        pending_startup,
        can_cancel_active,
        has_error,
    );
    let items = entries
        .iter()
        .map(|entry| {
            MenuItem::with_id(
                app,
                entry.command.id(),
                &entry.label,
                entry.enabled,
                None::<&str>,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let references = items
        .iter()
        .map(|item| item as &dyn tauri::menu::IsMenuItem<R>)
        .collect::<Vec<_>>();
    Ok(Menu::with_items(app, &references)?)
}

fn handle_menu<R: Runtime>(app: &AppHandle<R>, id: &str) -> Result<(), TrayError> {
    let context = app.state::<TrayContext>();
    match id {
        "start" => lifecycle::start_hidden(&context.repo_root),
        "open-console" => with_verified_descriptor(&context, |_descriptor, client| {
            let ticket = client.issue_observer_ticket()?;
            open_url(&client.console_url(&ticket))
        }),
        "drain" => lifecycle_action(&context, LifecycleAction::Drain),
        "resume" => lifecycle_action(&context, LifecycleAction::Resume),
        "stop" => lifecycle_action(&context, LifecycleAction::Stop),
        "restart" => lifecycle_action(&context, LifecycleAction::Restart),
        "force-stop" => lifecycle_action(&context, LifecycleAction::ForceStop),
        "cancel-pending" => {
            if let Ok(mut pending) = context.pending_action.lock() {
                *pending = None;
            }
            if let Ok(mut pending) = context.pending_startup.lock() {
                *pending = None;
            }
            Ok(())
        }
        "cancel-active" => cancel_active_action(&context),
        "diagnostics" => write_diagnostics(&context),
        "startup-query" => startup_action(&context, StartupAction::Query),
        "startup-install" => startup_action(&context, StartupAction::Install),
        "startup-update" => startup_action(&context, StartupAction::Update),
        "startup-remove" => startup_action(&context, StartupAction::Remove),
        _ => Ok(()),
    }
}

fn lifecycle_action(context: &TrayContext, action: LifecycleAction) -> Result<(), TrayError> {
    let confirmed = context
        .pending_action
        .lock()
        .map_err(|_| TrayError::Http("tray action state is unavailable".into()))?
        .take()
        .filter(|pending| pending.action == action);
    if let Some(pending) = confirmed {
        if pending.created_at.elapsed() >= Duration::from_secs(120) {
            return Err(TrayError::Http("lifecycle preview expired".into()));
        }
        return with_verified_descriptor(context, |descriptor, client| {
            let accepted = client.confirm_action(
                &pending.action_id,
                &pending.confirmation_phrase,
                "Windows tray confirmed controlled lifecycle",
            )?;
            if accepted.status == "executing" {
                *context
                    .active_action
                    .lock()
                    .map_err(|_| TrayError::Http("tray action state is unavailable".into()))? =
                    Some(ActiveAction {
                        action,
                        action_id: accepted.action_id.clone(),
                    });
            }
            if action == LifecycleAction::ForceStop {
                let result = lifecycle::force_stop_confirmed(
                    client,
                    descriptor,
                    accepted,
                    Duration::from_secs(35),
                );
                if let Ok(mut active) = context.active_action.lock() {
                    *active = None;
                }
                result?;
            }
            if let Ok(mut recovery) = context.recovery.lock() {
                match action {
                    LifecycleAction::Stop | LifecycleAction::ForceStop => recovery.request_stop(),
                    LifecycleAction::Restart => recovery.request_restart(),
                    _ => {}
                }
            }
            save_recovery(context)?;
            Ok(())
        });
    }

    with_verified_descriptor(context, |_descriptor, client| {
        let preview = client.preview_lifecycle(action.kind(), 30)?;
        let confirmation_phrase =
            preview
                .confirmation_phrase
                .clone()
                .ok_or(TrayError::InvalidDescriptor(
                    "lifecycle preview omitted confirmation phrase",
                ))?;
        let warning = if preview.warnings.is_empty() {
            "服务器已生成影响预览；再次点击同一菜单项才会执行。".to_owned()
        } else {
            format!(
                "{}；再次点击同一菜单项才会执行。",
                preview.warnings.join("；")
            )
        };
        *context
            .pending_action
            .lock()
            .map_err(|_| TrayError::Http("tray action state is unavailable".into()))? =
            Some(PendingAction {
                action,
                action_id: preview.action_id,
                confirmation_phrase,
                created_at: Instant::now(),
            });
        notifications::show_native(&TrayNotification {
            title: format!("确认：{}", action.kind()),
            body: warning,
        })?;
        Ok(())
    })
}

fn cancel_active_action(context: &TrayContext) -> Result<(), TrayError> {
    let active = context
        .active_action
        .lock()
        .map_err(|_| TrayError::Http("tray action state is unavailable".into()))?
        .clone()
        .ok_or_else(|| TrayError::Http("no lifecycle action is awaiting cancellation".into()))?;
    with_verified_descriptor(context, |_descriptor, client| {
        let cancelled = client.cancel_action(
            &active.action_id,
            "Windows tray cancelled lifecycle before shutdown",
        )?;
        if cancelled.status != "cancelled" {
            return Err(TrayError::Http(
                "coordinator did not confirm lifecycle cancellation".into(),
            ));
        }
        *context
            .active_action
            .lock()
            .map_err(|_| TrayError::Http("tray action state is unavailable".into()))? = None;
        let mut recovery = context
            .recovery
            .lock()
            .map_err(|_| TrayError::Http("tray recovery state is unavailable".into()))?;
        recovery.cancel_explicit_request();
        recovery.save(&recovery_path(&context.repo_root))?;
        Ok(())
    })
}

fn refresh_active_action(context: &TrayContext) -> Result<(), TrayError> {
    let active = context
        .active_action
        .lock()
        .map_err(|_| TrayError::Http("tray action state is unavailable".into()))?
        .clone();
    let Some(active) = active else {
        return Ok(());
    };
    let record = with_verified_descriptor(context, |_descriptor, client| {
        client.action(&active.action_id)
    })?;
    if lifecycle::is_terminal_status(&record.status) {
        *context
            .active_action
            .lock()
            .map_err(|_| TrayError::Http("tray action state is unavailable".into()))? = None;
    }
    Ok(())
}

fn startup_action(context: &TrayContext, action: StartupAction) -> Result<(), TrayError> {
    let result = if action == StartupAction::Query {
        startup::manage(&context.repo_root, action, false)?
    } else {
        let confirmed = context
            .pending_startup
            .lock()
            .map_err(|_| TrayError::Http("tray startup preview is unavailable".into()))?
            .take()
            .filter(|preview| preview.action == action);
        if let Some(preview) = confirmed {
            startup::execute_preview(&context.repo_root, &preview)?
        } else {
            let preview = startup::preview(&context.repo_root, action)?;
            *context
                .pending_startup
                .lock()
                .map_err(|_| TrayError::Http("tray startup preview is unavailable".into()))? =
                Some(preview);
            notifications::show_native(&TrayNotification {
                title: format!("确认启动项操作：{}", action.argument()),
                body: "已读取协调服务和托盘启动项当前状态；再次点击同一菜单项才会执行。".into(),
            })?;
            return Ok(());
        }
    };
    let path = context
        .repo_root
        .join(".codex")
        .join("state")
        .join("session-coordinator")
        .join("startup-action.json");
    fs::write(&path, serde_json::to_vec_pretty(&result)?)?;
    if let Ok(mut last_startup) = context.last_startup.lock() {
        *last_startup = Some(result.clone());
    }
    notifications::show_native(&TrayNotification {
        title: format!("启动项管理：{}", result.action),
        body: result.summary(),
    })?;
    if result.success() {
        Ok(())
    } else {
        Err(TrayError::Http(format!(
            "startup management was incomplete; inspect {}",
            path.display()
        )))
    }
}

fn with_verified_descriptor<T>(
    context: &TrayContext,
    operation: impl FnOnce(&RuntimeDescriptor, &CoordinatorClient<'_>) -> Result<T, TrayError>,
) -> Result<T, TrayError> {
    let descriptor = RuntimeDescriptor::read(runtime_path(&context.repo_root))?;
    descriptor.validate(&context.repository)?;
    inspect_process(descriptor.pid)?.verify(&descriptor)?;
    let client = CoordinatorClient::new(&descriptor);
    client.verify_health()?;
    operation(&descriptor, &client)
}

fn write_diagnostics(context: &TrayContext) -> Result<(), TrayError> {
    let path = context
        .repo_root
        .join(".codex")
        .join("state")
        .join("session-coordinator")
        .join("tray-diagnostics.json");
    let mut payload = match with_verified_descriptor(context, |descriptor, client| {
        let health = client.verify_health()?;
        Ok(json!({
            "descriptorVersion": descriptor.descriptor_version,
            "repositoryKey": descriptor.repository_key,
            "instanceId": health.instance_id,
            "pid": health.pid,
            "processCreationTime": health.process_creation_time,
            "schemaVersion": health.schema_version,
            "supervisionState": health.supervision.state,
            "blockerCount": health.supervision.blockers.len()
        }))
    }) {
        Ok(value) => value,
        Err(error) => json!({"status": "unavailable", "error": error.to_string()}),
    };
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "lastError".into(),
            context
                .last_error
                .lock()
                .ok()
                .and_then(|value| value.clone())
                .map_or(serde_json::Value::Null, serde_json::Value::String),
        );
        object.insert(
            "pendingAction".into(),
            context
                .pending_action
                .lock()
                .ok()
                .and_then(|value| {
                    value
                        .as_ref()
                        .map(|pending| pending.action.kind().to_owned())
                })
                .map_or(serde_json::Value::Null, serde_json::Value::String),
        );
        object.insert(
            "pendingStartupAction".into(),
            context
                .pending_startup
                .lock()
                .ok()
                .and_then(|value| {
                    value
                        .as_ref()
                        .map(|preview| preview.action.argument().to_owned())
                })
                .map_or(serde_json::Value::Null, serde_json::Value::String),
        );
        object.insert(
            "activeAction".into(),
            context
                .active_action
                .lock()
                .ok()
                .and_then(|value| value.as_ref().map(|active| active.action.kind().to_owned()))
                .map_or(serde_json::Value::Null, serde_json::Value::String),
        );
        object.insert(
            "lastStartup".into(),
            context
                .last_startup
                .lock()
                .ok()
                .and_then(|value| value.clone())
                .and_then(|value| serde_json::to_value(value).ok())
                .unwrap_or(serde_json::Value::Null),
        );
    }
    fs::write(path, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

#[cfg(windows)]
fn open_url(url: &str) -> Result<(), TrayError> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    let operation = HSTRING::from("open");
    let target = HSTRING::from(url);
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(operation.as_ptr()),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 <= 32 {
        return Err(TrayError::Http(
            "Windows could not open the console URL".into(),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn open_url(_url: &str) -> Result<(), TrayError> {
    Err(TrayError::Http(
        "opening the console is Windows-only".into(),
    ))
}
