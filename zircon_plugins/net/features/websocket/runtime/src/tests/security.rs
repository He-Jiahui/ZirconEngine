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

    assert_eq!(net.diagnostics().open_websocket_connections, 0);
    assert_eq!(net.diagnostics().queued_events, 0);
    assert!(net.drain_events(usize::MAX).is_empty());
}

#[test]
fn websocket_feature_manager_rejects_malformed_custom_certificate_root_before_network_io() {
    let net = websocket_runtime_manager();
    let mut descriptor = NetWebSocketConnectDescriptor::new("wss://example.invalid/socket");
    descriptor.security =
        NetSecurityPolicy::production_tls().with_certificate_root_der(vec![1, 2, 3]);

    let error = net.connect_websocket(descriptor).unwrap_err();
    assert!(
        matches!(error, NetError::SecurityPolicyViolation { reason } if reason.contains("TLS root certificate rejected")),
        "malformed custom roots must fail closed during TLS configuration: {error:?}"
    );
    assert_eq!(net.diagnostics().open_websocket_connections, 0);
    assert_eq!(net.diagnostics().queued_events, 0);
    assert!(net.drain_events(usize::MAX).is_empty());
}
