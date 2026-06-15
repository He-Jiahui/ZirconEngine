use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::Runtime;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetDiagnostics, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
    NetRouteId, NetWebSocketConnectDescriptor, NetWebSocketFrame, NetWebSocketListenerDescriptor,
};
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetManager, NetRuntimeMode,
};
use zircon_runtime::core::CoreRuntime;

use crate::record_net_diagnostics;
use crate::{
    DefaultNetManager, HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute,
    WebSocketRuntimeBackend, WebSocketRuntimeConnection, WebSocketRuntimeListener,
    NET_DIAGNOSTIC_INBOUND_BYTES, NET_DIAGNOSTIC_LAST_LATENCY_MS,
    NET_DIAGNOSTIC_OPEN_TCP_CONNECTIONS, NET_DIAGNOSTIC_OPEN_WEBSOCKET_CONNECTIONS,
    NET_DIAGNOSTIC_OUTBOUND_BYTES, NET_DIAGNOSTIC_PATHS, NET_DIAGNOSTIC_QUEUED_EVENTS,
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
