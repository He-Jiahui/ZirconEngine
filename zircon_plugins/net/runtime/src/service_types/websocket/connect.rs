use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetTransportKind,
    NetWebSocketConnectDescriptor,
};

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn connect_websocket_impl(
        &self,
        descriptor: NetWebSocketConnectDescriptor,
    ) -> Result<NetConnectionId, NetError> {
        let backend = self.websocket_backend()?;
        drop(lock_or_error(
            &self.state.websocket_connections,
            NetSharedState::WebSocketConnections,
        )?);
        let connection = self.next_connection_id();
        let network: Arc<dyn crate::websocket::WebSocketRuntimeConnection> =
            Arc::from(backend.connect_websocket(
                &self.state.runtime,
                connection,
                descriptor,
                self.state.events.clone(),
            )?);
        let mut connections = match lock_or_error(
            &self.state.websocket_connections,
            NetSharedState::WebSocketConnections,
        ) {
            Ok(connections) => connections,
            Err(error) => {
                network.set_state(NetConnectionState::Closed);
                return Err(error);
            }
        };
        connections.insert(connection, ManagedWebSocketConnection::Network(network));
        drop(connections);
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Connecting,
        });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        Ok(connection)
    }
}
