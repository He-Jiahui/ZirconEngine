use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetTransportKind,
    NetWebSocketCloseReason, NetWebSocketFrame,
};

use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn close_websocket_connection_impl(
        &self,
        connection: NetConnectionId,
    ) -> Result<(), NetError> {
        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        let entry = websockets
            .remove(&connection)
            .ok_or(NetError::UnknownConnection { connection })?;
        match entry {
            ManagedWebSocketConnection::Loopback(entry) => {
                if let Some(ManagedWebSocketConnection::Loopback(peer_entry)) =
                    websockets.get_mut(&entry.peer)
                {
                    peer_entry.state = NetConnectionState::Closed;
                    peer_entry.inbound.push_back(NetWebSocketFrame::Close(
                        NetWebSocketCloseReason::normal("peer closed"),
                    ));
                }
            }
            ManagedWebSocketConnection::Network(entry) => {
                entry.set_state(NetConnectionState::Closed);
            }
        }
        self.state.push_event(NetEvent::ConnectionClosed {
            connection,
            transport: NetTransportKind::WebSocket,
        });
        Ok(())
    }
}
