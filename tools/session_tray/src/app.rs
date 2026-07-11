use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::menu::{Menu, MenuItem};
use tauri::{AppHandle, Manager, Runtime};

use crate::coordinator_client::CoordinatorClient;
use crate::lifecycle::{self, LifecycleAction};
use crate::menu::{menu_model, MenuCommand};
use crate::notifications::{self, NotificationPolicy};
use crate::process_identity::inspect_process;
use crate::recovery::{RecoveryController, RecoveryDecision, RecoveryGuard};
use crate::repository_identity::{RepositoryIdentity, RepositoryMutex};
use crate::runtime_descriptor::RuntimeDescriptor;
use crate::startup::{self, StartupAction};
use crate::tray_state::{MenuEnablement, SupervisionState, TrayVisualState};
use crate::TrayError;

const TRAY_ID: &str = "zircon-session-tray";

pub struct TrayContext {
    repo_root: PathBuf,
    repository: RepositoryIdentity,
    _mutex: RepositoryMutex,
    notifications: Mutex<NotificationPolicy>,
    recovery: Mutex<RecoveryController>,
}

struct Observation {
    visual: TrayVisualState,
    identity_verified: bool,
    supervision_state: SupervisionState,
    explicit_stop: bool,
    maintenance_hold: bool,
}

pub fn run() -> Result<(), TrayError> {
    let repo_root = resolve_repo_root()?;
    let repository = RepositoryIdentity::for_path(&repo_root)?;
    let repository_mutex = RepositoryMutex::acquire(&repository)?;
    tauri::Builder::default()
        .setup(move |app| {
            app.manage(TrayContext {
                repo_root,
                repository,
                _mutex: repository_mutex,
                notifications: Mutex::new(NotificationPolicy::default()),
                recovery: Mutex::new(RecoveryController::default()),
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
                let _ = handle_menu(&handle, &id);
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
    })
}

fn supervise_loop<R: Runtime>(app: AppHandle<R>) {
    loop {
        thread::sleep(Duration::from_secs(2));
        let context = app.state::<TrayContext>();
        match observe(&context) {
            Ok(observation) => {
                if let Ok(mut recovery) = context.recovery.lock() {
                    recovery.observe_online(
                        unix_seconds(),
                        RecoveryGuard {
                            state: observation.supervision_state,
                            explicit_stop: observation.explicit_stop,
                            maintenance_hold: observation.maintenance_hold,
                            valid_competing_instance: false,
                        },
                    );
                }
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
            Err(_) => match context
                .recovery
                .lock()
                .map(|mut recovery| recovery.observe_offline(unix_seconds(), true))
                .unwrap_or(RecoveryDecision::Suppressed)
            {
                RecoveryDecision::RetryAfter(delay) => {
                    notify_state_change(&context, TrayVisualState::Recovering);
                    let _ = render_tray(&app, TrayVisualState::Recovering, true);
                    thread::sleep(Duration::from_secs(delay));
                    let _ = lifecycle::start_hidden(&context.repo_root);
                }
                RecoveryDecision::CircuitOpen | RecoveryDecision::Suppressed => {
                    notify_state_change(&context, TrayVisualState::Offline);
                    let _ = render_tray(&app, TrayVisualState::Offline, true);
                }
            },
        }
    }
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

fn render_tray<R: Runtime>(
    app: &AppHandle<R>,
    state: TrayVisualState,
    identity_verified: bool,
) -> Result<(), TrayError> {
    let tray = app.tray_by_id(TRAY_ID).ok_or(TrayError::InvalidDescriptor(
        "configured tray icon is missing",
    ))?;
    tray.set_tooltip(Some(state.tooltip()))?;
    tray.set_menu(Some(build_menu(app, state.menu(identity_verified))?))?;
    Ok(())
}

fn build_menu<R: Runtime>(
    app: &AppHandle<R>,
    enablement: MenuEnablement,
) -> Result<Menu<R>, TrayError> {
    let entries = menu_model(enablement);
    let items = entries
        .iter()
        .map(|entry| {
            MenuItem::with_id(
                app,
                entry.command.id(),
                entry.label,
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
        "force-stop" => with_verified_descriptor(&context, |descriptor, client| {
            lifecycle::force_stop(client, descriptor, 30)?;
            if let Ok(mut recovery) = context.recovery.lock() {
                recovery.request_stop();
            }
            Ok(())
        }),
        "diagnostics" => write_diagnostics(&context),
        "startup" => {
            let output = startup::manage(&context.repo_root, StartupAction::Query, false)?;
            let path = context
                .repo_root
                .join(".codex")
                .join("state")
                .join("session-coordinator")
                .join("startup-query.txt");
            fs::write(path, output.stdout)?;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn lifecycle_action(context: &TrayContext, action: LifecycleAction) -> Result<(), TrayError> {
    with_verified_descriptor(context, |_descriptor, client| {
        lifecycle::request(client, action, 30, "Windows tray controlled lifecycle")?;
        if let Ok(mut recovery) = context.recovery.lock() {
            match action {
                LifecycleAction::Stop => recovery.request_stop(),
                LifecycleAction::Restart => recovery.request_restart(),
                _ => {}
            }
        }
        Ok(())
    })
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
    let payload = match with_verified_descriptor(context, |descriptor, client| {
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
