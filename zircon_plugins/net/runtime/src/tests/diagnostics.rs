use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::Runtime;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetHttpRequestDescriptor, NetHttpResponseDescriptor, NetRouteId,
    NetWebSocketConnectDescriptor, NetWebSocketListenerDescriptor,
};
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetManager, NetRuntimeMode,
};

use crate::{
    DefaultNetManager, HttpRuntimeBackend, ManagedHttpListener, ManagedHttpRoute,
    WebSocketRuntimeBackend, WebSocketRuntimeConnection, WebSocketRuntimeListener,
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
