use zircon_runtime::core::framework::net::{
    NetError, NetManager, NetSecurityPolicy, NetWebSocketConnectDescriptor,
};

use crate::websocket_runtime_manager;

#[test]
fn websocket_feature_manager_rejects_connections_that_violate_security_policy_before_network_io() {
    let net = websocket_runtime_manager();
    let mut tls_required = NetWebSocketConnectDescriptor::new("ws://example.invalid/socket");
    tls_required.security = NetSecurityPolicy::production_tls();

    assert_eq!(
        net.connect_websocket(tls_required).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "WebSocket connection requires WSS by security policy".to_string(),
        }
    );

    let mut pinning_missing = NetWebSocketConnectDescriptor::new("wss://example.invalid/socket");
    pinning_missing.security.certificate_pinning = true;

    assert_eq!(
        net.connect_websocket(pinning_missing).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "WebSocket certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
}

#[test]
fn websocket_feature_manager_accepts_configured_certificate_pin_before_network_io() {
    let net = websocket_runtime_manager();
    let mut descriptor = NetWebSocketConnectDescriptor::new("wss://example.invalid/socket");
    descriptor.security = NetSecurityPolicy::production_tls()
        .with_certificate_pin("example.invalid", "sha256/example");

    let error = net.connect_websocket(descriptor).unwrap_err();
    assert_ne!(
        error,
        NetError::SecurityPolicyViolation {
            reason: "WebSocket certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
}
