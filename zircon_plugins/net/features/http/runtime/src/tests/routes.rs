use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetEndpoint, NetHttpMethod, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetHttpRouteDescriptor, NetManager, NetRequestId,
};

use crate::backend::HTTP_ROUTE_REQUEST_BODY_LIMIT_BYTES;
use crate::http_runtime_manager;

#[test]
fn http_round_trip_against_local_hyper_server() {
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
fn range_request_returns_partial() {
    let net = http_runtime_manager();
    let saw_range = Arc::new(AtomicBool::new(false));
    let saw_range_for_handler = saw_range.clone();
    net.register_http_route_handler(
        NetHttpRouteDescriptor::new("/chunks/range", [NetHttpMethod::Get]),
        move |request| {
            saw_range_for_handler.store(
                request.headers.iter().any(|(name, value)| {
                    name.eq_ignore_ascii_case("range") && value == "bytes=4-8"
                }),
                Ordering::SeqCst,
            );
            NetHttpResponseDescriptor::new(request.request, 206, b"45678".to_vec())
                .with_header("content-range", "bytes 4-8/10")
        },
    )
    .unwrap();
    let listener = net.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let response = net
        .send_http_request(
            NetHttpRequestDescriptor::new(
                NetRequestId::new(18),
                NetHttpMethod::Get,
                format!("http://{}:{}/chunks/range", endpoint.host, endpoint.port),
            )
            .with_byte_range(4, 8),
        )
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(18));
    assert_eq!(response.status_code, 206);
    assert_eq!(response.body, b"45678");
    assert!(response.headers.iter().any(|(name, value)| {
        name.eq_ignore_ascii_case("content-range") && value == "bytes 4-8/10"
    }));
    assert!(saw_range.load(Ordering::SeqCst));
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
