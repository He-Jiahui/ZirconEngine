use std::str::FromStr;

use zircon_runtime_interface::hub_protocol::HubSessionToken;

use super::reject_automation_hub_handshake;

#[test]
fn automation_rejects_a_hub_handshake_instead_of_dropping_its_terminal_outcome() {
    let handshake = super::super::HubEditorHandshake::new(
        "E:/Projects/Automation",
        HubSessionToken::from_str("0d9a5890-0e44-4e2a-b77e-3e5d4fdf1e52")
            .expect("valid Hub session token"),
    );

    let error = reject_automation_hub_handshake(Some(handshake))
        .expect_err("automation cannot report the interactive Hub ready outcome");

    assert!(error.to_string().contains("cannot acknowledge"));
}

#[test]
fn automation_allows_a_regular_non_hub_config() {
    assert!(reject_automation_hub_handshake(None).is_ok());
}
