use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use tauri::Emitter;

use crate::build::run_build_command;
use crate::error::HubError;

use super::action_request::HubActionRequest;
use super::runtime_state::HubRuntimeSession;
use super::view_model::HubViewModel;

pub(super) struct HubCommandState {
    session: Arc<Mutex<HubRuntimeSession>>,
}

impl HubCommandState {
    pub(super) fn load() -> Result<Self, HubError> {
        Ok(Self {
            session: Arc::new(Mutex::new(HubRuntimeSession::load()?)),
        })
    }

    fn session(&self) -> Result<MutexGuard<'_, HubRuntimeSession>, HubError> {
        self.session
            .lock()
            .map_err(|_| HubError::message("Hub runtime state lock is poisoned"))
    }

    fn session_handle(&self) -> Arc<Mutex<HubRuntimeSession>> {
        Arc::clone(&self.session)
    }
}

pub(super) fn hub_state(
    state: tauri::State<'_, HubCommandState>,
) -> Result<HubViewModel, HubError> {
    let session = state.session()?;
    Ok(session.view_model())
}

pub(super) fn hub_action(
    request: HubActionRequest,
    state: tauri::State<'_, HubCommandState>,
    app: tauri::AppHandle,
) -> Result<HubViewModel, HubError> {
    if HubRuntimeSession::should_run_action_in_background(&request) {
        let session_handle = state.session_handle();
        let mut session = state.session()?;
        let should_spawn = session.start_background_action_or_record_error(&request)?;
        let view_model = session.view_model();
        drop(session);

        if should_spawn {
            spawn_background_action(request, session_handle, app.clone());
        }
        let _ = app.emit("hub-state-changed", &view_model);
        return Ok(view_model);
    }

    let mut session = state.session()?;
    let view_model = session.apply_action(request)?;
    let _ = app.emit("hub-state-changed", &view_model);
    Ok(view_model)
}

fn spawn_background_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    thread::spawn(move || {
        if request.action_id.trim() == "build-project" {
            run_background_build_action(request, session_handle, app);
            return;
        }
        if request.action_id.trim() == "package-project" {
            run_background_package_action(request, session_handle, app);
            return;
        }
        if request.action_id.trim() == "install-device" {
            run_background_install_action(request, session_handle, app);
            return;
        }
        if request.action_id.trim() == "open-editor" {
            run_background_editor_action(request, session_handle, app);
            return;
        }

        let Ok(mut session) = session_handle.lock() else {
            return;
        };
        let view_model = match session.apply_action(request.clone()) {
            Ok(view_model) => view_model,
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                session.view_model()
            }
        };
        drop(session);

        emit_and_continue(session_handle, app, view_model);
    });
}

fn run_background_build_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    let pending_build = {
        let Ok(mut session) = session_handle.lock() else {
            return;
        };
        if let Err(error) = session.apply_request_project_target(&request) {
            let _ = session.record_background_action_error(&request, error.to_string());
            let view_model = session.view_model();
            drop(session);
            emit_and_continue(session_handle, app, view_model);
            return;
        }
        let pending_build = match session.prepare_background_editor_runtime_build() {
            Ok(pending_build) => pending_build,
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                let view_model = session.view_model();
                drop(session);
                emit_and_continue(session_handle, app, view_model);
                return;
            }
        };
        let view_model = session.view_model();
        drop(session);
        let _ = app.emit("hub-state-changed", &view_model);
        pending_build
    };

    let Some(pending_build) = pending_build else {
        emit_current_state_and_continue(session_handle, app);
        return;
    };

    let build_result = run_build_command(pending_build.command());
    let Ok(mut session) = session_handle.lock() else {
        return;
    };
    let view_model =
        match session.complete_background_editor_runtime_build(pending_build, build_result) {
            Ok(()) => session.view_model(),
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                session.view_model()
            }
        };
    drop(session);

    emit_and_continue(session_handle, app, view_model);
}

fn run_background_package_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    let pending_package = {
        let Ok(mut session) = session_handle.lock() else {
            return;
        };
        if let Err(error) = session.apply_request_project_target(&request) {
            let _ = session.record_background_action_error(&request, error.to_string());
            let view_model = session.view_model();
            drop(session);
            emit_and_continue(session_handle, app, view_model);
            return;
        }
        let pending_package = match session.prepare_background_project_package() {
            Ok(pending_package) => pending_package,
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                let view_model = session.view_model();
                drop(session);
                emit_and_continue(session_handle, app, view_model);
                return;
            }
        };
        let view_model = session.view_model();
        drop(session);
        let _ = app.emit("hub-state-changed", &view_model);
        pending_package
    };

    let Some(pending_package) = pending_package else {
        emit_current_state_and_continue(session_handle, app);
        return;
    };

    let package_result = pending_package.run();
    let Ok(mut session) = session_handle.lock() else {
        return;
    };
    let view_model =
        match session.complete_background_project_package(pending_package, package_result) {
            Ok(()) => session.view_model(),
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                session.view_model()
            }
        };
    drop(session);

    emit_and_continue(session_handle, app, view_model);
}

fn run_background_install_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    let pending_install = {
        let Ok(mut session) = session_handle.lock() else {
            return;
        };
        if let Err(error) = session.apply_request_project_target(&request) {
            let _ = session.record_background_action_error(&request, error.to_string());
            let view_model = session.view_model();
            drop(session);
            emit_and_continue(session_handle, app, view_model);
            return;
        }
        let pending_install = match session.prepare_background_device_install() {
            Ok(pending_install) => pending_install,
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                let view_model = session.view_model();
                drop(session);
                emit_and_continue(session_handle, app, view_model);
                return;
            }
        };
        let view_model = session.view_model();
        drop(session);
        let _ = app.emit("hub-state-changed", &view_model);
        pending_install
    };

    let Some(pending_install) = pending_install else {
        emit_current_state_and_continue(session_handle, app);
        return;
    };

    let install_result = pending_install.run();
    let Ok(mut session) = session_handle.lock() else {
        return;
    };
    let view_model =
        match session.complete_background_device_install(pending_install, install_result) {
            Ok(()) => session.view_model(),
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                session.view_model()
            }
        };
    drop(session);

    emit_and_continue(session_handle, app, view_model);
}

fn run_background_editor_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    let pending_launch = {
        let Ok(mut session) = session_handle.lock() else {
            return;
        };
        if let Err(error) = session.apply_request_project_target(&request) {
            let _ = session.record_background_action_error(&request, error.to_string());
            let view_model = session.view_model();
            drop(session);
            emit_and_continue(session_handle, app, view_model);
            return;
        }
        let pending_launch = match session.prepare_background_editor_launch() {
            Ok(pending_launch) => pending_launch,
            Err(error) => {
                let _ = session.record_background_action_error(&request, error.to_string());
                let view_model = session.view_model();
                drop(session);
                emit_and_continue(session_handle, app, view_model);
                return;
            }
        };
        let view_model = session.view_model();
        drop(session);
        let _ = app.emit("hub-state-changed", &view_model);
        pending_launch
    };

    let Some(pending_launch) = pending_launch else {
        emit_current_state_and_continue(session_handle, app);
        return;
    };

    let launch_result = pending_launch.run();
    let Ok(mut session) = session_handle.lock() else {
        return;
    };
    let view_model = match session.complete_background_editor_launch(pending_launch, launch_result)
    {
        Ok(()) => session.view_model(),
        Err(error) => {
            let _ = session.record_background_action_error(&request, error.to_string());
            session.view_model()
        }
    };
    drop(session);

    emit_and_continue(session_handle, app, view_model);
}

fn emit_and_continue(
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
    view_model: HubViewModel,
) {
    let _ = app.emit("hub-state-changed", &view_model);
    continue_background_queue(session_handle, app);
}

fn emit_current_state_and_continue(
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    let Ok(session) = session_handle.lock() else {
        return;
    };
    let view_model = session.view_model();
    drop(session);
    emit_and_continue(session_handle, app, view_model);
}

fn continue_background_queue(session_handle: Arc<Mutex<HubRuntimeSession>>, app: tauri::AppHandle) {
    let next_request = {
        let Ok(mut session) = session_handle.lock() else {
            return;
        };
        session.take_next_background_action()
    };

    if let Some(next_request) = next_request {
        spawn_background_action(next_request, session_handle, app);
    }
}
