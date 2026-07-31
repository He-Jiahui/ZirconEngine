use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use tokio::runtime::Runtime;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetEvent, NetManager,
    NetRuntimeMode, NetTransportKind, NetWebSocketCloseReason, NetWebSocketConnectDescriptor,
    NetWebSocketFrame, NetWebSocketListenerDescriptor,
};

use crate::{
    DefaultNetManager, WebSocketRuntimeBackend, WebSocketRuntimeConnection,
    WebSocketRuntimeListener,
};

#[test]
fn net_runtime_queues_websocket_frames_with_budget() {
    let net = DefaultNetManager::default();
    let (client, server) = net.open_websocket_loopback().unwrap();

    net.send_websocket_frame(client, NetWebSocketFrame::Text("hello".to_string()))
        .unwrap();
    net.send_websocket_frame(client, NetWebSocketFrame::Binary(vec![1, 2, 3]))
        .unwrap();

    assert_eq!(
        net.poll_websocket_frames(server, 1).unwrap(),
        vec![NetWebSocketFrame::Text("hello".to_string())]
    );
    assert_eq!(
        net.poll_websocket_frames(server, 8).unwrap(),
        vec![NetWebSocketFrame::Binary(vec![1, 2, 3])]
    );

    net.send_websocket_frame(
        server,
        NetWebSocketFrame::Close(NetWebSocketCloseReason::normal("done")),
    )
    .unwrap();
    assert!(matches!(
        net.poll_websocket_frames(client, 8).unwrap().as_slice(),
        [NetWebSocketFrame::Close(reason)] if reason.reason == "done"
    ));
    assert_eq!(
        net.connection_state(client).unwrap(),
        NetConnectionState::Closed
    );

    net.close_connection(server).unwrap();
    let events = net.drain_events(16);
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionClosed {
            connection,
            transport,
        } if *connection == server && *transport == NetTransportKind::WebSocket
    )));
}

#[test]
fn base_net_runtime_requires_websocket_feature_for_real_handshake() {
    let net = DefaultNetManager::default();

    assert_eq!(
        net.listen_websocket(NetWebSocketListenerDescriptor::new(NetEndpoint::new(
            "127.0.0.1",
            0,
        )))
        .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.websocket".to_string(),
        }
    );
    assert_eq!(
        net.connect_websocket(NetWebSocketConnectDescriptor::new(
            "ws://127.0.0.1:9/socket"
        ))
        .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.websocket".to_string(),
        }
    );
}

#[test]
fn websocket_callbacks_can_reenter_manager_without_registry_deadlock() {
    let (net, manager_slot, _, _) =
        manager_with_reentrant_websocket_backend(ReentrantWebSocketMode::Normal);

    let caller = net.clone();
    let (completed, result) = mpsc::sync_channel(1);
    let request_thread = thread::spawn(move || {
        let result = (|| {
            let listener = caller.listen_websocket(NetWebSocketListenerDescriptor::new(
                NetEndpoint::new("127.0.0.1", 0),
            ))?;
            assert_eq!(
                caller.listener_endpoint(listener)?,
                NetEndpoint::new("127.0.0.1", 0)
            );
            caller.close_listener(listener)?;
            let connection = caller.connect_websocket(NetWebSocketConnectDescriptor::new(
                "ws://127.0.0.1/reentrant",
            ))?;
            assert_eq!(
                caller.connection_state(connection)?,
                NetConnectionState::Open
            );
            caller.send_websocket_frame(connection, NetWebSocketFrame::Ping(vec![1]))?;
            assert!(caller.poll_websocket_frames(connection, 1)?.is_empty());
            caller.close_connection(connection)
        })();
        let _ = completed.send(result);
    });
    result
        .recv_timeout(Duration::from_secs(2))
        .expect("reentrant WebSocket callbacks must not deadlock the connection registry")
        .unwrap();
    request_thread.join().unwrap();
    *manager_slot.lock().unwrap() = None;
    net.shutdown_worker_result_for_tests().unwrap();
}

#[test]
fn websocket_connect_post_callback_poison_closes_without_orphan_event() {
    let (net, manager_slot, connection_closed, _) =
        manager_with_reentrant_websocket_backend(ReentrantWebSocketMode::PoisonConnectCommit);

    assert_eq!(
        net.connect_websocket(NetWebSocketConnectDescriptor::new(
            "ws://127.0.0.1/poison-connect",
        ))
        .unwrap_err(),
        NetError::SharedStatePoisoned {
            resource: "net.websocket_connections".to_string(),
        }
    );
    assert!(connection_closed.load(Ordering::SeqCst));
    assert_eq!(net.diagnostics().open_websocket_connections, 0);
    assert!(net.drain_events(usize::MAX).is_empty());

    *manager_slot.lock().unwrap() = None;
    net.shutdown_worker_result_for_tests().unwrap();
}

#[test]
fn websocket_listen_post_callback_poison_drops_without_orphan_event() {
    let (net, manager_slot, _, listener_dropped) =
        manager_with_reentrant_websocket_backend(ReentrantWebSocketMode::PoisonListenCommit);

    assert_eq!(
        net.listen_websocket(NetWebSocketListenerDescriptor::new(NetEndpoint::new(
            "127.0.0.1",
            0,
        )))
        .unwrap_err(),
        NetError::SharedStatePoisoned {
            resource: "net.websocket_listeners".to_string(),
        }
    );
    assert!(listener_dropped.load(Ordering::SeqCst));
    assert_eq!(net.diagnostics().open_websocket_listeners, 0);
    assert!(net.drain_events(usize::MAX).is_empty());

    *manager_slot.lock().unwrap() = None;
    net.shutdown_worker_result_for_tests().unwrap();
}

#[test]
fn websocket_accept_callback_failure_closes_every_staged_connection() {
    let (net, manager_slot, connection_closed, _) =
        manager_with_reentrant_websocket_backend(ReentrantWebSocketMode::FailSecondAccept);
    let listener = net
        .listen_websocket(NetWebSocketListenerDescriptor::new(NetEndpoint::new(
            "127.0.0.1",
            0,
        )))
        .unwrap();
    net.drain_events(usize::MAX);

    assert!(matches!(
        net.accept_websocket(listener, 2),
        Err(NetError::Io(detail)) if detail.contains("second accept")
    ));
    assert!(connection_closed.load(Ordering::SeqCst));
    assert_eq!(net.diagnostics().open_websocket_connections, 0);
    assert!(net.drain_events(usize::MAX).is_empty());

    net.close_listener(listener).unwrap();
    *manager_slot.lock().unwrap() = None;
    net.shutdown_worker_result_for_tests().unwrap();
}

fn manager_with_reentrant_websocket_backend(
    mode: ReentrantWebSocketMode,
) -> (
    DefaultNetManager,
    Arc<Mutex<Option<DefaultNetManager>>>,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
) {
    let manager_slot = Arc::new(Mutex::new(None));
    let connection_closed = Arc::new(AtomicBool::new(false));
    let listener_dropped = Arc::new(AtomicBool::new(false));
    let backend = Arc::new(ReentrantWebSocketBackend {
        manager: Arc::clone(&manager_slot),
        mode,
        connection_closed: Arc::clone(&connection_closed),
        listener_dropped: Arc::clone(&listener_dropped),
    });
    let net = DefaultNetManager::for_mode(NetRuntimeMode::Client).with_websocket_backend(backend);
    *manager_slot.lock().unwrap() = Some(net.clone());
    (net, manager_slot, connection_closed, listener_dropped)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReentrantWebSocketMode {
    Normal,
    PoisonConnectCommit,
    PoisonListenCommit,
    FailSecondAccept,
}

#[derive(Debug)]
struct ReentrantWebSocketBackend {
    manager: Arc<Mutex<Option<DefaultNetManager>>>,
    mode: ReentrantWebSocketMode,
    connection_closed: Arc<AtomicBool>,
    listener_dropped: Arc<AtomicBool>,
}

impl ReentrantWebSocketBackend {
    fn manager(&self) -> DefaultNetManager {
        self.manager
            .lock()
            .unwrap()
            .as_ref()
            .expect("test manager must be installed")
            .clone()
    }
}

impl WebSocketRuntimeBackend for ReentrantWebSocketBackend {
    fn listen_websocket(
        &self,
        _runtime: &Runtime,
        descriptor: NetWebSocketListenerDescriptor,
    ) -> Result<Box<dyn WebSocketRuntimeListener>, NetError> {
        let manager = self.manager();
        let _ = manager.diagnostics();
        if self.mode == ReentrantWebSocketMode::PoisonListenCommit {
            manager.poison_websocket_listeners_for_test();
        }
        Ok(Box::new(ReentrantWebSocketListener {
            manager,
            endpoint: descriptor.bind,
            mode: self.mode,
            accept_calls: AtomicUsize::new(0),
            connection_closed: Arc::clone(&self.connection_closed),
            dropped: Arc::clone(&self.listener_dropped),
        }))
    }

    fn connect_websocket(
        &self,
        _runtime: &Runtime,
        _connection: NetConnectionId,
        _descriptor: NetWebSocketConnectDescriptor,
        _events: Arc<Mutex<VecDeque<NetEvent>>>,
    ) -> Result<Box<dyn WebSocketRuntimeConnection>, NetError> {
        let manager = self.manager();
        let _ = manager.diagnostics();
        if self.mode == ReentrantWebSocketMode::PoisonConnectCommit {
            manager.poison_websocket_connections_for_test();
        }
        Ok(Box::new(ReentrantWebSocketConnection {
            manager,
            closed: Arc::clone(&self.connection_closed),
        }))
    }
}

#[derive(Debug)]
struct ReentrantWebSocketListener {
    manager: DefaultNetManager,
    endpoint: NetEndpoint,
    mode: ReentrantWebSocketMode,
    accept_calls: AtomicUsize,
    connection_closed: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
}

impl WebSocketRuntimeListener for ReentrantWebSocketListener {
    fn local_endpoint(&self) -> NetEndpoint {
        let _ = self.manager.diagnostics();
        self.endpoint.clone()
    }

    fn accept_websocket(
        &self,
        _runtime: &Runtime,
        _connection: NetConnectionId,
        _events: Arc<Mutex<VecDeque<NetEvent>>>,
        _poll_timeout: Duration,
    ) -> Result<Option<(NetEndpoint, Box<dyn WebSocketRuntimeConnection>)>, NetError> {
        let _ = self.manager.diagnostics();
        if self.mode != ReentrantWebSocketMode::FailSecondAccept {
            return Ok(None);
        }
        if self.accept_calls.fetch_add(1, Ordering::SeqCst) == 0 {
            return Ok(Some((
                NetEndpoint::new("127.0.0.1", 9000),
                Box::new(ReentrantWebSocketConnection {
                    manager: self.manager.clone(),
                    closed: Arc::clone(&self.connection_closed),
                }),
            )));
        }
        Err(NetError::Io("injected second accept failure".to_string()))
    }
}

impl Drop for ReentrantWebSocketListener {
    fn drop(&mut self) {
        let _ = self.manager.diagnostics();
        self.dropped.store(true, Ordering::SeqCst);
    }
}

#[derive(Debug)]
struct ReentrantWebSocketConnection {
    manager: DefaultNetManager,
    closed: Arc<AtomicBool>,
}

impl ReentrantWebSocketConnection {
    fn reenter(&self) {
        let _ = self.manager.diagnostics();
    }
}

impl WebSocketRuntimeConnection for ReentrantWebSocketConnection {
    fn state(&self) -> NetConnectionState {
        self.reenter();
        if self.closed.load(Ordering::SeqCst) {
            NetConnectionState::Closed
        } else {
            NetConnectionState::Open
        }
    }

    fn set_state(&self, state: NetConnectionState) {
        self.reenter();
        self.closed
            .store(state == NetConnectionState::Closed, Ordering::SeqCst);
    }

    fn send(&self, _runtime: &Runtime, _frame: NetWebSocketFrame) -> Result<(), NetError> {
        self.reenter();
        Ok(())
    }

    fn drain_frames(&self, _max_frames: usize) -> Vec<NetWebSocketFrame> {
        self.reenter();
        Vec::new()
    }
}
