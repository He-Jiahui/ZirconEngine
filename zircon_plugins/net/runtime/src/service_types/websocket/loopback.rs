use std::collections::VecDeque;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetTransportKind,
};

use crate::websocket::{LoopbackWebSocketConnection, ManagedWebSocketConnection};

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn open_websocket_loopback_impl(
        &self,
    ) -> Result<(NetConnectionId, NetConnectionId), NetError> {
        let client = self.next_connection_id();
        let server = self.next_connection_id();
        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        websockets.insert(
            client,
            ManagedWebSocketConnection::Loopback(LoopbackWebSocketConnection {
                peer: server,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            }),
        );
        websockets.insert(
            server,
            ManagedWebSocketConnection::Loopback(LoopbackWebSocketConnection {
                peer: client,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            }),
        );
        self.state
            .push_event(NetEvent::WebSocketPairOpened { client, server });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection: client,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection: server,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        Ok((client, server))
    }
}
