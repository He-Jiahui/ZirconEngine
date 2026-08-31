use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, Instant};

use zircon_runtime_interface::hub_protocol::{
    HubEditorMailboxV1, HubEditorReadyReceiptV1, HubSessionToken,
};

use super::mailbox_path::editor_handshake_mailbox_path;
use super::read::read_editor_handshake;
use super::wait::wait_for_editor_handshake_until;
use crate::error::HubError;

struct TestProjectRoot {
    path: PathBuf,
}

impl TestProjectRoot {
    fn create(label: &str) -> Self {
        let target_directory = std::env::var_os("CARGO_TARGET_DIR")
            .expect("Hub mailbox filesystem tests require coordinator-managed CARGO_TARGET_DIR");
        let unique = format!(
            "{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after the Unix epoch")
                .as_nanos()
        );
        let path = PathBuf::from(target_directory)
            .join("zircon-hub-editor-handshake-tests")
            .join(unique);
        fs::create_dir_all(&path).expect("create test project root");
        Self { path }
    }
}

impl Drop for TestProjectRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn mailbox_path_is_scoped_to_the_project_and_typed_session_token() {
    let token = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("parse deterministic test token");

    assert_eq!(
        editor_handshake_mailbox_path("E:/Projects/My Game", token),
        PathBuf::from("E:/Projects/My Game/.zircon/hub/0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52.json")
    );
}

#[test]
fn missing_mailbox_is_pending() {
    let project = TestProjectRoot::create("missing-mailbox");
    let token = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("parse deterministic test token");
    let mailbox_path = editor_handshake_mailbox_path(&project.path, token);

    assert_eq!(
        read_editor_handshake(&mailbox_path, token).expect("read missing mailbox"),
        None
    );
}

#[test]
fn malformed_mailbox_is_an_immediate_json_error() {
    let project = TestProjectRoot::create("malformed-mailbox");
    let token = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("parse deterministic test token");
    let mailbox_path = editor_handshake_mailbox_path(&project.path, token);
    fs::create_dir_all(mailbox_path.parent().expect("mailbox parent"))
        .expect("create mailbox directory");
    fs::write(&mailbox_path, b"not valid JSON").expect("write malformed mailbox");

    assert!(matches!(
        read_editor_handshake(&mailbox_path, token),
        Err(HubError::Json(_))
    ));
}

#[test]
fn mailbox_with_another_launch_session_is_rejected_before_hub_reports_ready() {
    let project = TestProjectRoot::create("mismatched-session-mailbox");
    let expected_session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("parse expected session");
    let other_session = HubSessionToken::from_str("2bf0bdb0-45f0-4cc9-a32c-8e1829854a14")
        .expect("parse mismatched session");
    let mailbox_path = editor_handshake_mailbox_path(&project.path, expected_session);
    fs::create_dir_all(mailbox_path.parent().expect("mailbox parent"))
        .expect("create mailbox directory");
    let mailbox = HubEditorMailboxV1::ready(
        other_session,
        HubEditorReadyReceiptV1::after_first_present(913, "913-42", 1).expect("ready receipt"),
    );
    fs::write(
        &mailbox_path,
        serde_json::to_vec(&mailbox).expect("encode mailbox"),
    )
    .expect("write mismatched mailbox");

    let error = read_editor_handshake(&mailbox_path, expected_session)
        .expect_err("a mailbox for another launch must be rejected");

    assert!(error.to_string().contains("launch session does not match"));
}

#[test]
fn wait_returns_the_first_complete_editor_mailbox_without_sleeping() {
    let session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("parse deterministic test token");
    let expected = HubEditorMailboxV1::ready(
        session,
        HubEditorReadyReceiptV1::after_first_present(913, "913-42", 1).expect("ready receipt"),
    );

    assert_eq!(
        wait_for_editor_handshake_until(
            Instant::now() + Duration::from_secs(1),
            Duration::ZERO,
            || Ok(Some(expected.clone()))
        )
        .expect("receive mailbox"),
        expected
    );
}

#[test]
fn wait_reports_timeout_when_no_mailbox_arrives_before_the_deadline() {
    let error = wait_for_editor_handshake_until(Instant::now(), Duration::ZERO, || Ok(None))
        .expect_err("deadline is already exhausted");

    assert_eq!(error.to_string(), "editor Hub handshake timed out");
}
