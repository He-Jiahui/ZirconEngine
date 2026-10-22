use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use zircon_runtime_interface::hub_protocol::HubSessionToken;
use zircon_runtime_interface::project::session_lock::{
    ProjectSessionAdmissionLifecycleV1, ProjectSessionAdmissionRecordV1,
    ProjectSessionGenerationV1, ProjectSessionPrincipalV1,
};
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use crate::core::hub_link::{consume_focus_signals, publish_focus_signal};

use super::{consume_pending_focus_signals, may_publish_focus_signal, HubFocusSignalWatch};

#[test]
fn mailbox_reads_and_removals_do_not_trigger_another_scan() {
    use notify::event::{AccessKind, AccessMode, RemoveKind};
    use notify::{Event, EventKind};

    let directory = std::path::Path::new("project/.zircon/hub/focus/913-42/requests");
    for kind in [
        EventKind::Access(AccessKind::Any),
        EventKind::Access(AccessKind::Open(AccessMode::Read)),
        EventKind::Access(AccessKind::Read),
        EventKind::Access(AccessKind::Close(AccessMode::Read)),
        EventKind::Remove(RemoveKind::File),
    ] {
        let event = Event::new(kind).add_path(directory.to_path_buf());
        assert!(!may_publish_focus_signal(&event, directory), "{kind:?}");
    }
}

#[test]
fn mailbox_publication_and_overflow_recovery_trigger_a_scan() {
    use notify::event::{CreateKind, Flag, ModifyKind, RenameMode};
    use notify::{Event, EventKind};

    let directory = std::path::Path::new("project/.zircon/hub/focus/913-42/requests");
    for kind in [
        EventKind::Any,
        EventKind::Create(CreateKind::File),
        EventKind::Modify(ModifyKind::Name(RenameMode::To)),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
    ] {
        assert!(may_publish_focus_signal(
            &Event::new(kind).add_path(directory.join("request.json")),
            directory,
        ));
        assert!(!may_publish_focus_signal(
            &Event::new(kind).add_path(PathBuf::from("another/inbox/request.json")),
            directory,
        ));
    }
    assert!(may_publish_focus_signal(
        &Event::new(EventKind::Other).set_flag(Flag::Rescan),
        directory,
    ));
}

#[test]
fn live_watcher_delivers_a_request_published_after_registration_once() {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("focus watcher filesystem tests require a managed CARGO_TARGET_DIR");
    let project_root = PathBuf::from(target_directory).join(format!(
        "zircon-editor-live-focus-watch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    let instance_id = "913-42";
    let (sender, receiver) = std::sync::mpsc::channel();
    let watcher = HubFocusSignalWatch::start(&project_root, instance_id, 1, move |request| {
        sender
            .send(request)
            .expect("focus receiver should remain alive");
    })
    .expect("register real OS watcher");
    let request_id = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("valid request id");
    publish_focus_signal(&project_root, &ready_focus_record(instance_id), request_id)
        .expect("publish while the OS watcher is registered");
    let request = receiver
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("watcher should consume the newly published request");
    assert_eq!(request.request_id, request_id);
    assert!(receiver
        .recv_timeout(std::time::Duration::from_millis(100))
        .is_err());
    drop(watcher);
    std::fs::remove_dir_all(project_root).expect("remove temporary project root");
}

#[test]
fn focus_mailbox_path_names_the_target_instance_only() {
    let request_id = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("valid request id");
    let target = super::super::focus_signal_path("E:/Projects/My Game", "4132-17", 1, request_id)
        .expect("valid instance id");

    assert_eq!(
        target,
        PathBuf::from(
            "E:/Projects/My Game/.zircon/hub/focus/4132-17/requests/00000000000000000001-0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52.json"
        )
    );
}

#[test]
fn watcher_startup_consumes_a_focus_signal_published_before_registration() {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("focus watcher filesystem tests require coordinator-managed CARGO_TARGET_DIR");
    let project_root = std::path::PathBuf::from(target_directory).join(format!(
        "zircon-editor-focus-watch-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    let instance_id = "913-42";
    let lock_record = ready_focus_record(instance_id);
    let session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("test Hub session token should be valid");
    std::fs::create_dir_all(project_root.join(".zircon/hub/focus"))
        .expect("create focus mailbox directory");
    publish_focus_signal(&project_root, &lock_record, session)
        .expect("publish focus signal before watcher registration");

    let attention_count = AtomicUsize::new(0);
    consume_pending_focus_signals(&project_root, instance_id, 1, |_| {
        attention_count.fetch_add(1, Ordering::Relaxed);
    })
    .expect("startup consume should accept the targeted signal");

    assert_eq!(attention_count.load(Ordering::Relaxed), 1);
    assert_eq!(
        consume_focus_signals(&project_root, instance_id, 1)
            .expect("focus mailbox should remain readable after startup consume"),
        Vec::new()
    );
    std::fs::remove_dir_all(project_root).expect("remove temporary project root");
}

fn ready_focus_record(instance_id: &str) -> ProjectSessionAdmissionRecordV1 {
    let operation = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fixture operation");
    let approved = ProjectSessionAdmissionRecordV1::claim(
        913,
        instance_id,
        ProjectSessionPrincipalV1::Hub,
        ZrRuntimeBuildSetId::parse(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("fixture BuildSet"),
        operation,
        42,
    )
    .expect("fixture admission record")
    .transition_to(ProjectSessionAdmissionLifecycleV1::PreflightApproved)
    .expect("fixture preflight approval");
    let activating = approved
        .transition_to(ProjectSessionAdmissionLifecycleV1::Activating)
        .expect("fixture activation");
    activating
        .commit_ready(ProjectSessionGenerationV1::new(1).expect("fixture generation"))
        .expect("fixture ready record")
}
