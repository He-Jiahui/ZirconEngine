use std::fs;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use zircon_runtime_interface::hub_protocol::HubSessionToken;

use super::{lock_before_deadline, EditorTerminalInbox, HubEditorShutdownHandle};
use crate::process::editor_child_receipt::EditorChildDisposition;
use crate::process::{
    editor_handshake::wait_for_editor_handshake, EditorChildReaper, SupervisedChild,
};
use crate::settings::HubConfig;
use crate::state::{TaskCancellationToken, TaskExecutionOutcome};
use crate::tauri_app::action_request::HubActionRequest;
use crate::tauri_app::runtime_state::action_tasks::EditorLaunchOwner;
use crate::tauri_app::runtime_state::HubRuntimeSession;

#[test]
fn shutdown_session_lock_does_not_wait_past_the_deadline() {
    let session = Mutex::new(());
    let held = session.lock().expect("hold session lock");
    assert!(lock_before_deadline(&session, Instant::now()).is_none());
    drop(held);
    assert!(lock_before_deadline(&session, Instant::now()).is_some());
}

#[test]
fn queued_pre_ready_terminal_projects_after_launch_task_finishes() {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("Hub shutdown tests require coordinator-managed CARGO_TARGET_DIR");
    let temp = std::path::PathBuf::from(target_directory).join(format!(
        "zircon-hub-queued-pre-ready-terminal-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms(),
    ));
    fs::create_dir_all(&temp).unwrap();
    let config_path = temp.join("hub.toml");
    HubConfig::default().save(&config_path).unwrap();
    let session = Arc::new(Mutex::new(
        HubRuntimeSession::load_from_paths(config_path, temp.join("recent_projects.json"))
            .expect("load fixture session"),
    ));
    let inbox = EditorTerminalInbox::start(Arc::downgrade(&session)).unwrap();
    let task_id = {
        let mut held = session.lock().unwrap();
        held.start_background_action_status(&HubActionRequest {
            action_id: "open-editor".to_string(),
            target_id: None,
            payload: None,
        })
        .unwrap();
        let task_id = held.active_background_task_id().unwrap();
        inbox.publish(
            crate::process::editor_child_receipt::EditorChildTerminalReceipt::stopped_before_ready(
                task_id, 913, None,
            ),
        );
        held.finish_background_task(task_id);
        task_id
    };
    let deadline = Instant::now() + Duration::from_secs(2);
    while !inbox.pending_receipts().is_empty() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(inbox.pending_receipts().is_empty());
    let held = session.lock().unwrap();
    assert!(
        held.config.action_history.iter().any(|record| {
            record.process_id == Some(913)
                && record.detail
                    == crate::state::HubMessage::with_params(
                        crate::state::HubMessageId::Process(
                            crate::state::ProcessMessageId::EditorProcessStoppedBeforeReady,
                        ),
                        ["913".to_string()],
                    )
        }),
        "task {task_id} terminal must survive task completion before projection"
    );
    drop(held);
    inbox.shutdown_and_join(deadline).unwrap();
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn busy_session_cannot_prevent_pre_ready_child_cancellation_and_reap() {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("Hub shutdown tests require coordinator-managed CARGO_TARGET_DIR");
    let temp = std::path::PathBuf::from(target_directory).join(format!(
        "zircon-hub-pre-ready-shutdown-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms(),
    ));
    fs::create_dir_all(&temp).unwrap();
    let config_path = temp.join("hub.toml");
    HubConfig::default().save(&config_path).unwrap();
    let session = Arc::new(Mutex::new(
        HubRuntimeSession::load_from_paths(config_path, temp.join("recent_projects.json"))
            .expect("load fixture session"),
    ));
    let owner = Arc::new(EditorLaunchOwner::default());
    let token = TaskCancellationToken::new(41);
    owner.admit(&token);
    let (started_sender, started_receiver) = mpsc::channel();
    let (reaped_sender, reaped_receiver) = mpsc::channel();
    let terminal_inbox = EditorTerminalInbox::start(Arc::downgrade(&session))
        .expect("start production terminal projection");
    let observed_inbox = terminal_inbox.clone();
    let reaper = EditorChildReaper::start_with_observer(move |receipt| {
        observed_inbox.publish(receipt);
    })
    .unwrap();
    let worker_reaper = reaper.clone();
    let worker_session = Arc::clone(&session);
    let worker_temp = temp.clone();
    let worker = thread::spawn(move || {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args([
            "--exact",
            "tauri_app::commands::shutdown_tests::pre_ready_child_fixture",
            "--ignored",
        ]);
        let mut child = SupervisedChild::spawn(&mut command, "pre-Ready Editor fixture")
            .expect("spawn fixture child");
        started_sender.send(child.id()).unwrap();
        let outcome =
            wait_for_editor_handshake(&worker_temp, HubSessionToken::new(), &mut child, &token)
                .expect("cancellation ends handshake wait");
        assert!(matches!(outcome, TaskExecutionOutcome::Cancelled));
        worker_reaper
            .cancel_before_ready(41, child)
            .expect("reap pre-Ready child with terminal receipt");
        reaped_sender.send(()).unwrap();
        let _session = worker_session.lock().unwrap();
    });
    let process_id = started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("child reached pre-Ready wait");
    let worker_slot = Arc::new(Mutex::new(Some(worker)));
    let shutdown = HubEditorShutdownHandle {
        session: Arc::clone(&session),
        editor_child_reaper: reaper,
        editor_launch_admission_closed: Arc::new(AtomicBool::new(false)),
        editor_launch_owner: owner,
        background_worker: Arc::clone(&worker_slot),
        editor_terminal_inbox: terminal_inbox.clone(),
    };
    let held_session = session.lock().unwrap();
    let started = Instant::now();
    let error = shutdown
        .shutdown_with_timeout(Duration::from_secs(3))
        .expect_err("busy session must remain visibly incomplete");
    assert!(error.to_string().contains("attempt 41"));
    assert!(error.to_string().contains(&format!("PID {process_id}")));
    assert!(started.elapsed() < Duration::from_secs(5));
    reaped_receiver
        .try_recv()
        .expect("pre-Ready child must be reaped before bounded shutdown returns");
    let terminal = terminal_inbox
        .pending_receipts()
        .pop()
        .expect("PID-bound receipt remains explicit while session is busy");
    assert_eq!(terminal.attempt_id, 41);
    assert_eq!(terminal.process_id, process_id);
    assert_eq!(
        terminal.disposition,
        EditorChildDisposition::StoppedBeforeReady
    );
    drop(held_session);
    worker_slot
        .lock()
        .unwrap()
        .take()
        .expect("incomplete worker remains owned")
        .join()
        .expect("worker exits after session lock is released");
    fs::remove_dir_all(temp).unwrap();
}

#[test]
#[ignore = "child fixture for busy_session_cannot_prevent_pre_ready_child_cancellation_and_reap"]
fn pre_ready_child_fixture() {
    thread::sleep(Duration::from_secs(30));
}
