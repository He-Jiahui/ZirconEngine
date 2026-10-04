use std::str::FromStr;

use zircon_runtime_interface::hub_protocol::{
    HubEditorMailboxV1, HubEditorReadyReceiptV1, HubSessionToken,
};

use super::{handshake_mailbox_path, HubEditorHandshake};

#[test]
fn handshake_mailbox_is_scoped_to_the_project_and_typed_session() {
    let session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("valid test session");

    assert_eq!(
        handshake_mailbox_path("E:/Projects/My Game", session),
        std::path::PathBuf::from(
            "E:/Projects/My Game/.zircon/hub/0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52.json"
        )
    );
}

#[test]
fn handshake_publishes_a_ready_mailbox_with_the_editor_process_and_project() {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("Hub handshake filesystem tests require coordinator-managed CARGO_TARGET_DIR");
    let directory = std::path::PathBuf::from(target_directory).join(format!(
        "zircon-editor-hub-handshake-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos()
    ));
    let session = HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
        .expect("valid test session");
    let handshake = HubEditorHandshake::new(&directory, session);

    let receipt =
        HubEditorReadyReceiptV1::after_first_present(913, "913-42", 1).expect("ready receipt");
    handshake
        .publish_ready(receipt.clone())
        .expect("publish ready mailbox");

    let bytes = std::fs::read(handshake.mailbox_path()).expect("read ready mailbox");
    assert_eq!(
        serde_json::from_slice::<HubEditorMailboxV1>(&bytes).expect("decode ready mailbox"),
        HubEditorMailboxV1::ready(session, receipt)
    );
    std::fs::remove_dir_all(&directory).expect("remove temporary project root");
}
