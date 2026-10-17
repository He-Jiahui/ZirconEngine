use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::Runtime;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetDiagnostics, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetRequestId, NetRouteId, NetWebSocketConnectDescriptor, NetWebSocketFrame,
    NetWebSocketListenerDescriptor,
};
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetManager, NetRuntimeMode,
};
use zircon_runtime::core::{CoreRuntime, TasksModule, TimeModule};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime::scene::create_default_level;

use crate::record_net_diagnostics;
use crate::{
    DefaultNetManager, HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute,
    WebSocketRuntimeBackend, WebSocketRuntimeConnection, WebSocketRuntimeListener,
    NET_DIAGNOSTIC_INBOUND_BYTES, NET_DIAGNOSTIC_LAST_LATENCY_MS,
    NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS, NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS,
    NET_DIAGNOSTIC_OUTBOUND_BYTES, NET_DIAGNOSTIC_PATHS, NET_DIAGNOSTIC_QUEUED_EVENTS,
    NET_POLL_INGRESS_SYSTEM,
};

#[test]
fn net_runtime_manager_reports_mode_diagnostics_and_events() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::DedicatedServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();

    let diagnostics = net.diagnostics();
    assert_eq!(diagnostics.mode, NetRuntimeMode::DedicatedServer);
    assert_eq!(diagnostics.open_tcp_listeners, 1);
    assert_eq!(diagnostics.open_http_listeners, 0);
    assert_eq!(diagnostics.open_websocket_listeners, 0);
    assert_eq!(diagnostics.open_tcp_connections, 0);
    assert_eq!(diagnostics.outbound_bytes, 0);
    assert_eq!(diagnostics.inbound_bytes, 0);
    assert_eq!(diagnostics.last_observed_latency_ms, None);

    let events = net.drain_events(8);
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ListenerStarted {
            listener: started,
            transport,
            ..
        } if *started == listener && transport.is_tcp()
    )));
}

#[test]
fn diagnostic_paths_registered() {
    let runtime = CoreRuntime::new();
    let diagnostics = NetDiagnostics {
        backend_name: "tokio-net".to_string(),
        mode: NetRuntimeMode::Client,
        outbound_bytes: 42,
        inbound_bytes: 24,
        last_observed_latency_ms: Some(16),
        open_udp_sockets: 0,
        open_tcp_listeners: 0,
        open_http_listeners: 0,
        open_websocket_listeners: 0,
        open_tcp_connections: 2,
        open_http_routes: 0,
        open_websocket_connections: 3,
        queued_events: 4,
    };

    record_net_diagnostics(&runtime.handle(), 7, &diagnostics);

    let snapshot = runtime.diagnostic_store_snapshot();
    let outbound_series = snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == NET_DIAGNOSTIC_OUTBOUND_BYTES)
        .expect("outbound diagnostics should be recorded");
    assert_eq!(outbound_series.history.last().unwrap().frame_index, 7);
    assert!(NET_DIAGNOSTIC_PATHS.contains(&NET_DIAGNOSTIC_OUTBOUND_BYTES));
    assert_series(
        &snapshot,
        NET_DIAGNOSTIC_OUTBOUND_BYTES,
        Some("byte"),
        42.0,
        &["bandwidth", "net", "outbound"],
    );
    assert_series(
        &snapshot,
        NET_DIAGNOSTIC_INBOUND_BYTES,
        Some("byte"),
        24.0,
        &["bandwidth", "inbound", "net"],
    );
    assert_series(
        &snapshot,
        NET_DIAGNOSTIC_LAST_LATENCY_MS,
        Some("ms"),
        16.0,
        &["latency", "net"],
    );
    assert_series(
        &snapshot,
        NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS,
        Some("count"),
        2.0,
        &["connection", "net", "tcp"],
    );
    assert_series(
        &snapshot,
        NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS,
        Some("count"),
        3.0,
        &["connection", "net", "websocket"],
    );
    assert_series(
        &snapshot,
        NET_DIAGNOSTIC_QUEUED_EVENTS,
        Some("count"),
        4.0,
        &["event", "net"],
    );
}

#[test]
fn net_poll_ingress_records_outer_frame_indices_in_diagnostic_history() {
    let runtime = CoreRuntime::new();
    for descriptor in [
        TasksModule.descriptor(),
        TimeModule.descriptor(),
        zircon_runtime::asset::module_descriptor(),
        zircon_runtime::scene::module_descriptor(),
        crate::module_descriptor(),
    ] {
        runtime.register_module(descriptor).unwrap();
    }
    runtime.activate_registered_modules().unwrap();
    let core = runtime.handle();
    let level = create_default_level(&core).unwrap();

    let report = RuntimePluginRegistrationReport::from_plugin(&crate::runtime_plugin());
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .plugin_runtime_systems()
        .any(|(_, system)| system.id == NET_POLL_INGRESS_SYSTEM));
    let mut extensions = report.extensions;
    level
        .with_world_mut(|world| extensions.apply_to_world(world))
        .expect("net runtime systems should install into the level schedule");

    for frame_index in 1..=72 {
        let snapshot = runtime.advance_time_by(Duration::ZERO, 0);
        assert_eq!(snapshot.outer_frame_index(), frame_index);
    }
    for frame_index in [73, 74] {
        let snapshot = runtime.advance_time_by(Duration::ZERO, 0);
        assert_eq!(snapshot.outer_frame_index(), frame_index);
        level.tick(&core, snapshot).unwrap();
    }

    let snapshot = runtime.diagnostic_store_snapshot();
    for path in [
        NET_DIAGNOSTIC_OUTBOUND_BYTES,
        NET_DIAGNOSTIC_INBOUND_BYTES,
        NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS,
        NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS,
        NET_DIAGNOSTIC_QUEUED_EVENTS,
    ] {
        let series = snapshot
            .series
            .iter()
            .find(|series| series.path.as_str() == path)
            .unwrap_or_else(|| panic!("expected diagnostics series {path}"));
        assert_eq!(
            series
                .history
                .iter()
                .map(|measurement| measurement.frame_index)
                .collect::<Vec<_>>(),
            vec![73, 74],
            "diagnostics history should retain outer frame coordinates for {path}"
        );
    }
}

#[test]
fn net_runtime_diagnostics_records_bandwidth_counters() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::Client);
    let (sender, receiver) = net.open_websocket_loopback().unwrap();

    net.send_websocket_frame(sender, NetWebSocketFrame::Text("ping".to_string()))
        .unwrap();
    let sent = net.diagnostics();
    assert_eq!(sent.outbound_bytes, 4);
    assert_eq!(sent.inbound_bytes, 0);

    let frames = net.poll_websocket_frames(receiver, 4).unwrap();
    assert_eq!(frames, vec![NetWebSocketFrame::Text("ping".to_string())]);
    let received = net.diagnostics();
    assert_eq!(received.outbound_bytes, 4);
    assert_eq!(received.inbound_bytes, 4);
}

#[test]
fn net_runtime_diagnostics_count_listeners_by_transport() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::DedicatedServer)
        .with_http_backend(Arc::new(FakeHttpBackend))
        .with_websocket_backend(Arc::new(FakeWebSocketBackend));

    let tcp = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let http = net
        .listen_http(&NetEndpoint::new("127.0.0.1", 8080))
        .unwrap();
    let websocket = net
        .listen_websocket(NetWebSocketListenerDescriptor::new(NetEndpoint::new(
            "127.0.0.1",
            9001,
        )))
        .unwrap();

    let diagnostics = net.diagnostics();
    assert_eq!(diagnostics.open_tcp_listeners, 1);
    assert_eq!(diagnostics.open_http_listeners, 1);
    assert_eq!(diagnostics.open_websocket_listeners, 1);

    net.close_listener(http).unwrap();
    net.close_listener(websocket).unwrap();
    net.close_listener(tcp).unwrap();

    let diagnostics = net.diagnostics();
    assert_eq!(diagnostics.open_tcp_listeners, 0);
    assert_eq!(diagnostics.open_http_listeners, 0);
    assert_eq!(diagnostics.open_websocket_listeners, 0);
}

#[test]
fn event_ingress_rejects_full_route_admission_without_state_change_and_recovers_fifo() {
    let net = DefaultNetManager::default();
    const EVENT_COUNT: usize = 5_000;
    let mut accepted_count = 0;
    let mut rejected_count = 0;
    for index in 0..EVENT_COUNT {
        let result = net.register_http_route(
            zircon_runtime::core::framework::net::NetHttpRouteDescriptor::new(
                format!("/event-{index}"),
                [zircon_runtime::core::framework::net::NetHttpMethod::Get],
            ),
            NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, Vec::new()),
        );
        match result {
            Ok(route) => {
                assert_eq!(route, NetRouteId::new(accepted_count as u64 + 1));
                accepted_count += 1;
            }
            Err(NetError::EventQueueFull { capacity }) => {
                assert_eq!(capacity, 4_096);
                rejected_count += 1;
            }
            Err(error) => panic!("unexpected route admission error: {error:?}"),
        }
    }
    assert_eq!(accepted_count, 4_096);
    assert_eq!(rejected_count, EVENT_COUNT - 4_096);

    let handler_result = net.register_http_route_handler(
        zircon_runtime::core::framework::net::NetHttpRouteDescriptor::new(
            "/handler-rejected",
            [zircon_runtime::core::framework::net::NetHttpMethod::Get],
        ),
        |request| NetHttpResponseDescriptor::new(request.request, 200, Vec::new()),
    );
    assert_eq!(
        handler_result.unwrap_err(),
        NetError::EventQueueFull { capacity: 4_096 }
    );
    assert_eq!(
        net.unregister_http_route(NetRouteId::new(1)).unwrap_err(),
        NetError::EventQueueFull { capacity: 4_096 }
    );

    for _ in 0..20 {
        assert_eq!(net.diagnostics().queued_events, 4_096);
        assert_eq!(net.diagnostics().open_http_routes, 4_096);
    }
    assert_eq!(net.manager_event_queue_len_for_test(), 4_096);

    let mut delivered = Vec::new();
    loop {
        let batch = net.drain_events(4_096);
        if batch.is_empty() {
            break;
        }
        delivered.extend(batch);
    }
    assert_eq!(delivered.len(), 4_096);
    for (index, event) in delivered.into_iter().enumerate() {
        match event {
            NetEvent::HttpRouteRegistered { path, route, .. } => {
                assert_eq!(route, NetRouteId::new(index as u64 + 1));
                assert_eq!(path, format!("/event-{index}"));
            }
            other => panic!("unexpected event in accepted route FIFO: {other:?}"),
        }
    }

    let recovered = net
        .register_http_route(
            zircon_runtime::core::framework::net::NetHttpRouteDescriptor::new(
                "/after-drain",
                [zircon_runtime::core::framework::net::NetHttpMethod::Get],
            ),
            NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, Vec::new()),
        )
        .unwrap();
    assert_eq!(recovered, NetRouteId::new(4_097));
    net.unregister_http_route(NetRouteId::new(1)).unwrap();
    let recovered_events = net.drain_events(2);
    assert!(matches!(
        recovered_events.as_slice(),
        [
            NetEvent::HttpRouteRegistered { route, path, .. },
            NetEvent::HttpRouteUnregistered { route: removed },
        ] if *route == recovered && path == "/after-drain" && *removed == NetRouteId::new(1)
    ));
    assert_eq!(net.diagnostics().queued_events, 0);
}

#[test]
fn net_runtime_manager_closes_listeners_across_transports() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::DedicatedServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    assert_eq!(net.diagnostics().open_tcp_listeners, 1);

    net.close_listener(listener).unwrap();

    assert_eq!(net.diagnostics().open_tcp_listeners, 0);
    assert_eq!(
        net.listener_endpoint(listener).unwrap_err(),
        NetError::UnknownListener { listener }
    );
    assert!(net.drain_events(8).iter().any(|event| matches!(
        event,
        NetEvent::ListenerClosed {
            listener: closed,
            transport,
        } if *closed == listener && transport.is_tcp()
    )));
}

#[derive(Debug)]
struct FakeHttpBackend;

impl HttpRuntimeBackend for FakeHttpBackend {
    fn listen_http(
        &self,
        _runtime: &Runtime,
        bind: SocketAddr,
        _routes: Arc<Mutex<HashMap<NetRouteId, ManagedHttpRoute>>>,
    ) -> Result<ManagedHttpListener, NetError> {
        Ok(ManagedHttpListener {
            local_endpoint: NetEndpoint::from(bind),
            abort_handle: None,
        })
    }

    fn send_http_request(
        &self,
        _runtime: &Runtime,
        _request: NetHttpRequestDescriptor,
    ) -> Result<NetHttpResponseDescriptor, NetError> {
        Err(NetError::ProtocolUnavailable {
            capability: "test.http.request".to_string(),
        })
    }
}

#[derive(Debug)]
struct FakeWebSocketBackend;

impl WebSocketRuntimeBackend for FakeWebSocketBackend {
    fn listen_websocket(
        &self,
        _runtime: &Runtime,
        descriptor: NetWebSocketListenerDescriptor,
    ) -> Result<Box<dyn WebSocketRuntimeListener>, NetError> {
        Ok(Box::new(FakeWebSocketListener {
            endpoint: descriptor.bind,
        }))
    }

    fn connect_websocket(
        &self,
        _runtime: &Runtime,
        _connection: NetConnectionId,
        _descriptor: NetWebSocketConnectDescriptor,
        _events: Arc<Mutex<VecDeque<NetEvent>>>,
    ) -> Result<Box<dyn WebSocketRuntimeConnection>, NetError> {
        Err(NetError::ProtocolUnavailable {
            capability: "test.websocket.connect".to_string(),
        })
    }
}

#[derive(Debug)]
struct FakeWebSocketListener {
    endpoint: NetEndpoint,
}

impl WebSocketRuntimeListener for FakeWebSocketListener {
    fn local_endpoint(&self) -> NetEndpoint {
        self.endpoint.clone()
    }

    fn accept_websocket(
        &self,
        _runtime: &Runtime,
        _connection: NetConnectionId,
        _events: Arc<Mutex<VecDeque<NetEvent>>>,
        _poll_timeout: Duration,
    ) -> Result<Option<(NetEndpoint, Box<dyn WebSocketRuntimeConnection>)>, NetError> {
        Ok(None)
    }
}

fn assert_series(
    snapshot: &zircon_runtime::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
    unit: Option<&str>,
    current: f64,
    tags: &[&str],
) {
    let series = snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .expect("diagnostic path should be present");
    assert_eq!(series.unit.as_deref(), unit);
    assert_eq!(series.current, Some(current));
    assert_eq!(
        series
            .subsystem_tags
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        tags
    );
}
