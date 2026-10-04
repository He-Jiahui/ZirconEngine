use super::*;
use zircon_runtime::core::framework::net::{NetSecurityPolicy, NetWebSocketConnectDescriptor};

#[test]
fn custom_roots_and_pins_are_admitted_for_tls_configuration() {
    let mut pinned = NetWebSocketConnectDescriptor::new("wss://example.invalid/socket");
    pinned.security = NetSecurityPolicy::production_tls()
        .with_certificate_pin("example.invalid", "sha256/example");
    assert!(validate_websocket_security_policy(&pinned).is_ok());

    let mut rooted = NetWebSocketConnectDescriptor::new("wss://example.invalid/socket");
    rooted.security = NetSecurityPolicy::production_tls().with_certificate_root_der(vec![1, 2, 3]);
    assert!(validate_websocket_security_policy(&rooted).is_ok());
}

#[test]
fn websocket_url_host_handles_userinfo_ports_and_ipv6() {
    assert_eq!(
        websocket_url_host("wss://user:secret@example.invalid:443/socket"),
        Some("example.invalid".to_string())
    );
    assert_eq!(
        websocket_url_host("wss://[::1]:443/socket"),
        Some("::1".to_string())
    );
}

#[test]
fn configured_tls_material_cannot_be_silently_ignored_for_plaintext_loopback() {
    let mut descriptor = NetWebSocketConnectDescriptor::new("ws://127.0.0.1:9000/socket");
    descriptor.security =
        NetSecurityPolicy::development().with_certificate_pin("127.0.0.1", "sha256/example");

    assert_eq!(
        validate_websocket_security_policy(&descriptor),
        Err(NetError::SecurityPolicyViolation {
            reason: "WebSocket certificate roots and pinning require WSS".to_string(),
        })
    );
}
