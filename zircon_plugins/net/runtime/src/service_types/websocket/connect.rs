use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetTransportKind,
    NetWebSocketConnectDescriptor,
};

use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn connect_websocket_impl(
        &self,
        descriptor: NetWebSocketConnectDescriptor,
    ) -> Result<NetConnectionId, NetError> {
        let backend = self.websocket_backend()?;
        let connection = self.next_connection_id();
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Connecting,
        });
        let network = backend.connect_websocket(
            &self.state.runtime,
            connection,
            descriptor,
            self.state.events.clone(),
        )?;
        self.state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned")
            .insert(connection, ManagedWebSocketConnection::Network(network));
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        Ok(connection)
    }
}
