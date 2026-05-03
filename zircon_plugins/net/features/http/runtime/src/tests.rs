use zircon_plugin_net_runtime::DefaultNetManager;
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetHttpRouteDescriptor, NetManager, NetRequestId, NetSecurityPolicy,
};

use super::{
    http_runtime_manager, plugin_feature_registration, NET_HTTP_FEATURE_CAPABILITY,
    NET_HTTP_FEATURE_ID, NET_HTTP_FEATURE_MANAGER_NAME, NET_HTTP_FEATURE_MODULE_NAME,
};

#[test]
fn http_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_HTTP_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_HTTP_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_HTTP_FEATURE_MODULE_NAME)
        .expect("HTTP feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_HTTP_FEATURE_MANAGER_NAME
    );
}

#[test]
fn http_feature_manager_serves_registered_route_over_real_socket() {
    let net = http_runtime_manager();
    assert!(net.backend_name().contains("+http"));
    net.register_http_route(
        NetHttpRouteDescriptor::new("/socket-health", [NetHttpMethod::Get]),
        NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, b"socket-ok".to_vec())
            .with_header("content-type", "text/plain"),
    )
    .unwrap();
    let listener = net.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let response = net
        .send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(17),
            NetHttpMethod::Get,
            format!("http://{}:{}/socket-health", endpoint.host, endpoint.port),
        ))
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(17));
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"socket-ok");
}

#[test]
fn default_type_can_receive_http_backend_for_direct_tests() {
    let net: DefaultNetManager = http_runtime_manager();

    assert!(net.backend_name().contains("+http"));
}

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

    let mut pinning_required = NetHttpRequestDescriptor::new(
        NetRequestId::new(32),
        NetHttpMethod::Get,
        "https://example.invalid/socket-health",
    );
    pinning_required.security.certificate_pinning = true;

    assert_eq!(
        net.send_http_request(pinning_required).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "HTTP certificate pinning is not configured".to_string(),
        }
    );
}
