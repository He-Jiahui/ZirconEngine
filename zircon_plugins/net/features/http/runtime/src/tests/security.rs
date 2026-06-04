use zircon_runtime::core::framework::net::{
    NetError, NetHttpMethod, NetHttpRequestDescriptor, NetManager, NetRequestId, NetSecurityPolicy,
};

use crate::http_runtime_manager;

#[test]
fn http_feature_manager_rejects_requests_that_violate_security_policy_before_network_io() {
    let net = http_runtime_manager();
    let mut tls_required = NetHttpRequestDescriptor::new(
        NetRequestId::new(31),
        NetHttpMethod::Get,
        "http://example.invalid/socket-health",
    );
    tls_required.security = NetSecurityPolicy::production_tls();

    assert_eq!(
        net.send_http_request(tls_required).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "HTTP request requires HTTPS by security policy".to_string(),
        }
    );

    let mut pinning_missing = NetHttpRequestDescriptor::new(
        NetRequestId::new(32),
        NetHttpMethod::Get,
        "https://example.invalid/socket-health",
    );
    pinning_missing.security.certificate_pinning = true;

    assert_eq!(
        net.send_http_request(pinning_missing).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "HTTP certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
}

#[test]
fn http_feature_manager_accepts_configured_certificate_pin_before_network_io() {
    let net = http_runtime_manager();
    let mut request = NetHttpRequestDescriptor::new(
        NetRequestId::new(33),
        NetHttpMethod::Get,
        "https://example.invalid/socket-health",
    );
    request.security = NetSecurityPolicy::production_tls()
        .with_certificate_pin("example.invalid", "sha256/example");

    let error = net.send_http_request(request).unwrap_err();
    assert_ne!(
        error,
        NetError::SecurityPolicyViolation {
            reason: "HTTP certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
}
