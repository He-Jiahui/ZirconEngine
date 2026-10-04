use super::*;

#[test]
fn websocket_client_config_uses_public_roots_when_no_custom_roots_are_given() {
    let policy = NetSecurityPolicy::production_tls();
    assert!(rustls_client_config_for_websocket(&policy, "example.invalid").is_ok());
}

#[test]
fn websocket_client_config_rejects_malformed_custom_roots() {
    let policy = NetSecurityPolicy::production_tls().with_certificate_root_der(vec![1, 2, 3]);
    let error = rustls_client_config_for_websocket(&policy, "example.invalid")
        .expect_err("malformed custom roots must fail closed");
    assert!(
        matches!(error, NetError::SecurityPolicyViolation { reason } if reason.contains("TLS root certificate rejected"))
    );
}
