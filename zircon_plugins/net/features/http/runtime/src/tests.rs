use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_plugin_net_runtime::DefaultNetManager;
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetHttpRouteDescriptor, NetManager, NetRequestId, NetSecurityPolicy,
};

use crate::backend::HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES;

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

#[test]
fn http_feature_manager_retries_transient_server_statuses() {
    let net = http_runtime_manager();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_handler = attempts.clone();
    net.register_http_route_handler(
        NetHttpRouteDescriptor::new("/retry", [NetHttpMethod::Get]),
        move |request| {
            let attempt = attempts_for_handler.fetch_add(1, Ordering::SeqCst);
            if attempt == 0 {
                NetHttpResponseDescriptor::new(request.request, 503, b"try-again".to_vec())
            } else {
                NetHttpResponseDescriptor::new(request.request, 200, b"ok-after-retry".to_vec())
            }
        },
    )
    .unwrap();
    let listener = net.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let response = net
        .send_http_request(
            NetHttpRequestDescriptor::new(
                NetRequestId::new(34),
                NetHttpMethod::Get,
                format!("http://{}:{}/retry", endpoint.host, endpoint.port),
            )
            .with_max_retry_attempts(1),
        )
        .unwrap();

    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"ok-after-retry");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[test]
fn http_feature_manager_forwards_headers_and_body_to_socket_route_handlers() {
    let net = http_runtime_manager();
    let saw_header = Arc::new(AtomicBool::new(false));
    let saw_body = Arc::new(AtomicBool::new(false));
    let saw_header_for_handler = saw_header.clone();
    let saw_body_for_handler = saw_body.clone();
    net.register_http_route_handler(
        NetHttpRouteDescriptor::new("/inspect", [NetHttpMethod::Post]),
        move |request| {
            saw_header_for_handler.store(
                request.headers.iter().any(|(name, value)| {
                    name.eq_ignore_ascii_case("x-zircon-test") && value == "present"
                }),
                Ordering::SeqCst,
            );
            saw_body_for_handler.store(request.body == b"request-body", Ordering::SeqCst);
            NetHttpResponseDescriptor::new(request.request, 204, Vec::new())
        },
    )
    .unwrap();
    let listener = net.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let response = net
        .send_http_request(
            NetHttpRequestDescriptor::new(
                NetRequestId::new(35),
                NetHttpMethod::Post,
                format!("http://{}:{}/inspect", endpoint.host, endpoint.port),
            )
            .with_header("x-zircon-test", "present")
            .with_body(b"request-body".to_vec()),
        )
        .unwrap();

    assert_eq!(response.status_code, 204);
    assert!(saw_header.load(Ordering::SeqCst));
    assert!(saw_body.load(Ordering::SeqCst));
}

#[test]
fn http_feature_manager_rejects_oversized_route_body_before_handler_dispatch() {
    let net = http_runtime_manager();
    let handler_called = Arc::new(AtomicBool::new(false));
    let handler_called_for_handler = handler_called.clone();
    net.register_http_route_handler(
        NetHttpRouteDescriptor::new("/limited", [NetHttpMethod::Post]),
        move |request| {
            handler_called_for_handler.store(true, Ordering::SeqCst);
            NetHttpResponseDescriptor::new(request.request, 204, Vec::new())
        },
    )
    .unwrap();
    let listener = net.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let response = net
        .send_http_request(
            NetHttpRequestDescriptor::new(
                NetRequestId::new(36),
                NetHttpMethod::Post,
                format!("http://{}:{}/limited", endpoint.host, endpoint.port),
            )
            .with_body(vec![b'x'; HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES + 1]),
        )
        .unwrap();

    assert_eq!(response.status_code, 413);
    assert!(!handler_called.load(Ordering::SeqCst));
}

#[test]
fn http_feature_manager_matches_route_before_applying_body_limit() {
    let net = http_runtime_manager();
    let listener = net.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let response = net
        .send_http_request(
            NetHttpRequestDescriptor::new(
                NetRequestId::new(37),
                NetHttpMethod::Post,
                format!("http://{}:{}/missing", endpoint.host, endpoint.port),
            )
            .with_body(vec![b'x'; HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES + 1]),
        )
        .unwrap();

    assert_eq!(response.status_code, 404);
}
