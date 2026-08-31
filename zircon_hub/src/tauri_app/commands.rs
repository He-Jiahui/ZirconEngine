use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use tauri::Emitter;

use crate::error::HubError;

use super::action_request::HubActionRequest;
use super::runtime_state::action_tasks::run_background_worker_loop;
use super::runtime_state::HubRuntimeSession;
use super::view_model::HubViewModel;

mod focus_refresh_gate;

use focus_refresh_gate::FocusRefreshGate;

pub(super) struct HubCommandState {
    session: Arc<Mutex<HubRuntimeSession>>,
    focus_refresh_gate: FocusRefreshGate,
}

impl HubCommandState {
    pub(super) fn load() -> Result<Self, HubError> {
        Ok(Self {
            session: Arc::new(Mutex::new(HubRuntimeSession::load()?)),
            focus_refresh_gate: FocusRefreshGate::default(),
        })
    }

    fn session(&self) -> Result<MutexGuard<'_, HubRuntimeSession>, HubError> {
        self.session.lock().map_err(|_| {
            eprintln!("zircon_hub: Hub runtime session lock is poisoned");
            HubError::message("Hub runtime state lock is poisoned")
        })
    }

    fn session_handle(&self) -> Arc<Mutex<HubRuntimeSession>> {
        Arc::clone(&self.session)
    }

    pub(super) fn refresh_recent_projects_on_window_focus(&self, app: tauri::AppHandle) {
        let Some(focus_refresh_permit) = self.focus_refresh_gate.try_enter() else {
            return;
        };

        let session_handle = self.session_handle();
        thread::spawn(move || {
            let _focus_refresh_permit = focus_refresh_permit;
            let view_model = match session_handle.lock() {
                Ok(mut session) => {
                    let should_emit = match session.refresh_shared_recent_projects_on_focus() {
                        Ok(changed) => changed,
                        Err(error) => {
                            eprintln!(
                                "zircon_hub: failed to refresh shared recent projects: {error}"
                            );
                            true
                        }
                    };
                    should_emit.then(|| session.view_model())
                }
                Err(_) => {
                    eprintln!("zircon_hub: Hub runtime state lock is poisoned");
                    None
                }
            };
            if let Some(view_model) = view_model {
                let _ = app.emit("hub-state-changed", &view_model);
            }
        });
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
        let emit_state = |view_model: &HubViewModel| {
            let _ = app.emit("hub-state-changed", view_model);
        };
        run_background_worker_loop(request, &session_handle, &emit_state);
    });
}
