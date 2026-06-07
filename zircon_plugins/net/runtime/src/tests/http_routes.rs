use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetHttpMethod, NetHttpRequestDescriptor,
    NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetManager, NetRequestId,
};

use crate::DefaultNetManager;

#[test]
fn net_runtime_dispatches_registered_http_route() {
    let net = DefaultNetManager::default();
    let route = net
        .register_http_route(
            NetHttpRouteDescriptor::new("/health", [NetHttpMethod::Get]),
            NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, b"ok".to_vec())
                .with_header("content-type", "text/plain"),
        )
        .unwrap();

    let response = net
        .send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(7),
            NetHttpMethod::Get,
            "http://127.0.0.1/health",
        ))
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(7));
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"ok");
    assert_eq!(response.body_bytes, 2);
    assert_eq!(net.diagnostics().open_http_routes, 1);
    net.unregister_http_route(route).unwrap();
    assert_eq!(net.diagnostics().open_http_routes, 0);
    assert!(net
        .drain_events(8)
        .iter()
        .any(|event| matches!(event, NetEvent::HttpRouteUnregistered { route: removed } if *removed == route)));
}

#[test]
fn net_runtime_dispatches_dynamic_http_route_handler() {
    let net = DefaultNetManager::default();
    net.register_http_route_handler(
        NetHttpRouteDescriptor::new("/echo", [NetHttpMethod::Post]),
        |request| NetHttpResponseDescriptor::new(request.request, 201, request.body),
    )
    .unwrap();

    let response = net
        .send_http_request(
            NetHttpRequestDescriptor::new(
                NetRequestId::new(31),
                NetHttpMethod::Post,
                "http://127.0.0.1/echo",
            )
            .with_body(b"payload".to_vec()),
        )
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(31));
    assert_eq!(response.status_code, 201);
    assert_eq!(response.body, b"payload");
}

#[test]
fn base_net_runtime_requires_http_feature_for_real_socket_backend() {
    let net = DefaultNetManager::default();

    assert_eq!(
        net.listen_http(&NetEndpoint::new("127.0.0.1", 0))
            .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.http".to_string(),
        }
    );
    assert_eq!(
        net.send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(17),
            NetHttpMethod::Get,
            "http://127.0.0.1:9/socket-health",
        ))
        .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.http".to_string(),
        }
    );
}
