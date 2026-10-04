use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, TryLockError, Weak};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tauri::Emitter;

use crate::error::HubError;
use crate::process::editor_child_receipt::EditorChildTerminalReceipt;
use crate::process::EditorChildReaper;

use super::action_id::HubActionId;
use super::action_request::HubActionRequest;
use super::runtime_state::action_tasks::{run_background_worker_loop, EditorLaunchOwner};
use super::runtime_state::HubRuntimeSession;
use super::view_model::HubViewModel;

mod focus_refresh_gate;

use focus_refresh_gate::FocusRefreshGate;

pub(super) struct HubCommandState {
    session: Arc<Mutex<HubRuntimeSession>>,
    backend_epoch: String,
    focus_refresh_gate: FocusRefreshGate,
    editor_child_reaper: EditorChildReaper,
    editor_terminal_inbox: EditorTerminalInbox,
    editor_launch_admission_closed: Arc<AtomicBool>,
    editor_launch_owner: Arc<EditorLaunchOwner>,
    background_worker: Arc<Mutex<Option<JoinHandle<()>>>>,
}

pub(super) struct HubEditorShutdownHandle {
    session: Arc<Mutex<HubRuntimeSession>>,
    editor_child_reaper: EditorChildReaper,
    editor_launch_admission_closed: Arc<AtomicBool>,
    editor_launch_owner: Arc<EditorLaunchOwner>,
    background_worker: Arc<Mutex<Option<JoinHandle<()>>>>,
    editor_terminal_inbox: EditorTerminalInbox,
}

const EDITOR_SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

struct EditorTerminalInbox {
    shared: Arc<EditorTerminalInboxShared>,
    worker: Arc<Mutex<Option<JoinHandle<()>>>>,
}

struct EditorTerminalInboxShared {
    receipts: Mutex<VecDeque<EditorChildTerminalReceipt>>,
    wake: Condvar,
    shutdown_requested: AtomicBool,
    handle_count: AtomicUsize,
    emitter: Mutex<Option<tauri::AppHandle>>,
}

impl EditorTerminalInbox {
    fn start(session: Weak<Mutex<HubRuntimeSession>>) -> Result<Self, HubError> {
        let shared = Arc::new(EditorTerminalInboxShared {
            receipts: Mutex::new(VecDeque::new()),
            wake: Condvar::new(),
            shutdown_requested: AtomicBool::new(false),
            handle_count: AtomicUsize::new(1),
            emitter: Mutex::new(None),
        });
        let worker_shared = Arc::clone(&shared);
        let worker = thread::Builder::new()
            .name("zircon-hub-editor-terminal-projector".to_string())
            .spawn(move || project_editor_terminal_receipts(worker_shared, session))?;
        Ok(Self {
            shared,
            worker: Arc::new(Mutex::new(Some(worker))),
        })
    }

    fn publish(&self, receipt: EditorChildTerminalReceipt) {
        self.shared
            .receipts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_back(receipt);
        self.shared.wake.notify_one();
    }

    fn install_emitter(&self, app: tauri::AppHandle) {
        *self
            .shared
            .emitter
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(app);
    }

    fn pending_receipts(&self) -> Vec<EditorChildTerminalReceipt> {
        self.shared
            .receipts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .cloned()
            .collect()
    }

    fn shutdown_and_join(&self, deadline: Instant) -> Result<(), HubError> {
        while !self.pending_receipts().is_empty() && Instant::now() < deadline {
            self.shared.wake.notify_one();
            thread::sleep(Duration::from_millis(10));
        }
        self.shared
            .shutdown_requested
            .store(true, Ordering::Release);
        self.shared.wake.notify_all();
        loop {
            let finished = self
                .worker
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref()
                .is_none_or(JoinHandle::is_finished);
            if finished {
                if let Some(worker) = self
                    .worker
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .take()
                {
                    worker.join().map_err(|_| {
                        HubError::message("Hub Editor terminal projector panicked during shutdown")
                    })?;
                }
                break;
            }
            if Instant::now() >= deadline {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let pending = self.pending_receipts();
        if !pending.is_empty() {
            let evidence = pending
                .iter()
                .map(|receipt| format!("attempt {} PID {}", receipt.attempt_id, receipt.process_id))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(HubError::message(format!(
                "Editor children were reaped but terminal history remained incomplete: {evidence}"
            )));
        }
        if self
            .worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
        {
            return Err(HubError::message(
                "Hub Editor terminal projector did not finish before its shutdown deadline",
            ));
        }
        Ok(())
    }
}

impl Clone for EditorTerminalInbox {
    fn clone(&self) -> Self {
        self.shared.handle_count.fetch_add(1, Ordering::Relaxed);
        Self {
            shared: Arc::clone(&self.shared),
            worker: Arc::clone(&self.worker),
        }
    }
}

impl Drop for EditorTerminalInbox {
    fn drop(&mut self) {
        if self.shared.handle_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.shared
                .shutdown_requested
                .store(true, Ordering::Release);
            self.shared.wake.notify_all();
        }
    }
}

fn project_editor_terminal_receipts(
    shared: Arc<EditorTerminalInboxShared>,
    weak_session: Weak<Mutex<HubRuntimeSession>>,
) {
    loop {
        let receipt = {
            let mut receipts = shared
                .receipts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            while receipts.is_empty() && !shared.shutdown_requested.load(Ordering::Acquire) {
                receipts = shared
                    .wake
                    .wait(receipts)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
            if shared.shutdown_requested.load(Ordering::Acquire) {
                return;
            }
            receipts.front().cloned()
        };
        let Some(receipt) = receipt else { continue };
        let Some(session) = weak_session.upgrade() else {
            return;
        };
        let mut session = match session.try_lock() {
            Ok(session) => session,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
        };
        let result = session.observe_editor_child_terminal(receipt.clone());
        let view_model = match &result {
            Ok(changed) => changed.then(|| session.publish_view_model()),
            Err(error) => {
                eprintln!("zircon_hub: failed to persist Editor child terminal attempt {} PID {}: {error}", receipt.attempt_id, receipt.process_id);
                Some(session.publish_view_model())
            }
        };
        drop(session);
        if result.is_ok() {
            let mut receipts = shared
                .receipts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if receipts.front() == Some(&receipt) {
                receipts.pop_front();
            }
        }
        if let Some(view_model) = view_model {
            let app = shared
                .emitter
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone();
            if let Some(app) = app {
                let _ = app.emit("hub-state-changed", &view_model);
            }
        }
        if result.is_err() {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl HubCommandState {
    pub(super) fn load() -> Result<Self, HubError> {
        let session = Arc::new(Mutex::new(HubRuntimeSession::load()?));
        let backend_epoch = session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .view_model()
            .backend_epoch;
        let editor_launch_owner = session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .editor_launch_owner();
        let editor_terminal_inbox = EditorTerminalInbox::start(Arc::downgrade(&session))?;
        let terminal_inbox = editor_terminal_inbox.clone();
        let editor_child_reaper = EditorChildReaper::start_with_observer(move |receipt| {
            terminal_inbox.publish(receipt);
        })?;
        Ok(Self {
            session,
            backend_epoch,
            focus_refresh_gate: FocusRefreshGate::default(),
            editor_child_reaper,
            editor_terminal_inbox,
            editor_launch_admission_closed: Arc::new(AtomicBool::new(false)),
            editor_launch_owner,
            background_worker: Arc::new(Mutex::new(None)),
        })
    }

    pub(super) fn install_editor_terminal_emitter(&self, app: tauri::AppHandle) {
        self.editor_terminal_inbox.install_emitter(app);
    }

    pub(super) fn editor_shutdown_handle(&self) -> HubEditorShutdownHandle {
        HubEditorShutdownHandle {
            session: self.session_handle(),
            editor_child_reaper: self.editor_child_reaper(),
            editor_launch_admission_closed: Arc::clone(&self.editor_launch_admission_closed),
            editor_launch_owner: Arc::clone(&self.editor_launch_owner),
            background_worker: Arc::clone(&self.background_worker),
            editor_terminal_inbox: self.editor_terminal_inbox.clone(),
        }
    }

    fn own_background_worker(&self, worker: JoinHandle<()>) {
        let previous = self
            .background_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .replace(worker);
        if let Some(previous) = previous {
            if previous.is_finished() {
                let _ = previous.join();
            } else {
                eprintln!(
                    "zircon_hub: prior background worker still finishing after new admission"
                );
            }
        }
    }

    fn editor_launch_admission_closed(&self) -> bool {
        self.editor_launch_admission_closed.load(Ordering::Acquire)
    }

    pub(super) fn session(&self) -> Result<MutexGuard<'_, HubRuntimeSession>, HubError> {
        self.session.lock().map_err(|_| {
            eprintln!("zircon_hub: Hub runtime session lock is poisoned");
            HubError::message("Hub runtime state lock is poisoned")
        })
    }

    pub(super) fn backend_epoch(&self) -> &str {
        &self.backend_epoch
    }

    fn session_handle(&self) -> Arc<Mutex<HubRuntimeSession>> {
        Arc::clone(&self.session)
    }

    fn editor_child_reaper(&self) -> EditorChildReaper {
        self.editor_child_reaper.clone()
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
                    should_emit.then(|| session.publish_view_model())
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

impl HubEditorShutdownHandle {
    pub(super) fn shutdown(self) -> Result<(), HubError> {
        self.shutdown_with_timeout(EDITOR_SHUTDOWN_DEADLINE)
    }

    fn shutdown_with_timeout(self, timeout: Duration) -> Result<(), HubError> {
        let deadline = Instant::now() + timeout;
        self.editor_launch_admission_closed
            .store(true, Ordering::Release);
        let owned_task_id = self.editor_launch_owner.request_shutdown();
        self.editor_child_reaper.request_shutdown_until(deadline);
        let mut session_access_failed = false;
        let initial_task_id = match lock_before_deadline(&self.session, deadline) {
            Some(mut session) => owned_task_id.or(session.request_editor_launch_shutdown()),
            None => {
                session_access_failed = true;
                owned_task_id
            }
        };
        let unfinished_task_id = if session_access_failed {
            None
        } else {
            loop {
                let Some(mut session) = lock_before_deadline(&self.session, deadline) else {
                    session_access_failed = true;
                    break initial_task_id;
                };
                let active = session.request_editor_launch_shutdown();
                drop(session);
                if active.is_none() {
                    break None;
                }
                if Instant::now() >= deadline {
                    break active;
                }
                thread::sleep(Duration::from_millis(10));
            }
        };
        let worker_result = if initial_task_id.is_some() {
            self.join_editor_worker(deadline)
        } else {
            Ok(())
        };
        let reaper_result = self.editor_child_reaper.shutdown_and_join_until(deadline);
        let inbox_result = self.editor_terminal_inbox.shutdown_and_join(deadline);
        let unprojected_terminals = self.editor_terminal_inbox.pending_receipts();
        let incomplete_task_id = if unfinished_task_id.is_some() || worker_result.is_err() {
            initial_task_id
        } else {
            None
        };
        if incomplete_task_id.is_some()
            || reaper_result.is_err()
            || inbox_result.is_err()
            || session_access_failed
        {
            if let Some(mut session) = lock_before_deadline(&self.session, deadline) {
                let incomplete_task_id = incomplete_task_id.or_else(|| {
                    session_access_failed
                        .then(|| session.request_editor_launch_shutdown())
                        .flatten()
                });
                if incomplete_task_id.is_some()
                    || reaper_result.is_err()
                    || !unprojected_terminals.is_empty()
                {
                    if let Err(error) = session.record_editor_supervision_incomplete(
                        incomplete_task_id,
                        reaper_result.is_err(),
                        &unprojected_terminals,
                    ) {
                        eprintln!("zircon_hub: failed to persist incomplete Editor owner: {error}");
                    }
                }
            } else {
                eprintln!("zircon_hub: Editor shutdown could not persist incomplete owner because the Hub session lock stayed busy");
            }
        }
        if let Some(mut session) = lock_before_deadline(&self.session, deadline) {
            if let Err(error) = session.retry_pending_editor_terminal_persistence() {
                eprintln!("zircon_hub: failed to retry Editor terminal history save: {error}");
            }
        }
        if let Err(error) = inbox_result {
            eprintln!("zircon_hub: {error}");
            return Err(error);
        }
        if session_access_failed {
            return Err(HubError::message(
                "Hub session was unavailable before the Editor shutdown deadline",
            ));
        }
        if let Some(task_id) = incomplete_task_id {
            return Err(HubError::message(format!(
                "Editor launch task {task_id} did not stop before Hub shutdown"
            )));
        }
        worker_result?;
        reaper_result
    }

    fn join_editor_worker(&self, deadline: Instant) -> Result<(), HubError> {
        loop {
            let worker = {
                let mut slot = self
                    .background_worker
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if slot.as_ref().is_some_and(JoinHandle::is_finished) {
                    slot.take()
                } else {
                    None
                }
            };
            if let Some(worker) = worker {
                return worker.join().map_err(|_| {
                    HubError::message("Hub Editor launch worker panicked during shutdown")
                });
            }
            if Instant::now() >= deadline {
                return Err(HubError::message(
                    "Hub Editor launch worker did not finish before its shutdown deadline",
                ));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

fn lock_before_deadline<T>(mutex: &Mutex<T>, deadline: Instant) -> Option<MutexGuard<'_, T>> {
    loop {
        match mutex.try_lock() {
            Ok(guard) => return Some(guard),
            Err(TryLockError::Poisoned(poisoned)) => return Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) if Instant::now() >= deadline => return None,
            Err(TryLockError::WouldBlock) => thread::sleep(Duration::from_millis(10)),
        }
    }
}

pub(super) fn hub_state(
    state: tauri::State<'_, HubCommandState>,
) -> Result<HubViewModel, HubError> {
    let mut session = state.session()?;
    if let Err(error) = session.retry_pending_editor_terminal_persistence() {
        eprintln!("zircon_hub: failed to retry Editor terminal history save: {error}");
    }
    Ok(session.publish_view_model())
}

pub(super) fn hub_action(
    request: HubActionRequest,
    state: tauri::State<'_, HubCommandState>,
    app: tauri::AppHandle,
) -> Result<HubViewModel, HubError> {
    if HubRuntimeSession::should_run_action_in_background(&request) {
        let session_handle = state.session_handle();
        let mut session = state.session()?;
        if let Err(error) = session.retry_pending_editor_terminal_persistence() {
            eprintln!("zircon_hub: failed to retry Editor terminal history save: {error}");
        }
        reject_closed_editor_launch(&request, &state)?;
        let should_spawn = session.start_background_action_or_record_error(&request)?;
        let view_model = session.publish_view_model();
        if should_spawn {
            let worker = spawn_background_action(
                request,
                session_handle,
                state.editor_child_reaper(),
                Arc::clone(&state.editor_launch_admission_closed),
                app.clone(),
            );
            state.own_background_worker(worker);
        }
        drop(session);
        let _ = app.emit("hub-state-changed", &view_model);
        return Ok(view_model);
    }

    let mut session = state.session()?;
    if let Err(error) = session.retry_pending_editor_terminal_persistence() {
        eprintln!("zircon_hub: failed to retry Editor terminal history save: {error}");
    }
    reject_closed_editor_launch(&request, &state)?;
    let view_model = session.apply_action(request)?;
    let _ = app.emit("hub-state-changed", &view_model);
    Ok(view_model)
}

fn reject_closed_editor_launch(
    request: &HubActionRequest,
    state: &HubCommandState,
) -> Result<(), HubError> {
    if state.editor_launch_admission_closed()
        && matches!(
            request.action()?,
            HubActionId::OpenEditor | HubActionId::CreateProject
        )
    {
        Err(HubError::message("Hub Editor launch admission is closed"))
    } else {
        Ok(())
    }
}

fn spawn_background_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    editor_child_reaper: EditorChildReaper,
    editor_launch_admission_closed: Arc<AtomicBool>,
    app: tauri::AppHandle,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let emit_state = |view_model: &HubViewModel| {
            let _ = app.emit("hub-state-changed", view_model);
        };
        run_background_worker_loop(
            request,
            &session_handle,
            &emit_state,
            &editor_child_reaper,
            &editor_launch_admission_closed,
        );
    })
}

#[cfg(test)]
#[path = "tests/commands_shutdown_tests.rs"]
mod shutdown_tests;
