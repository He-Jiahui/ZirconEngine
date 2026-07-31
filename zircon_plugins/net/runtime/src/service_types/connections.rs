use std::sync::Arc;

use zircon_runtime::core::framework::net::{NetConnectionId, NetConnectionState, NetError};

use crate::poison_recovery::{lock_or_error, NetSharedState};

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn connection_state_impl(
        &self,
        connection: NetConnectionId,
    ) -> Result<NetConnectionState, NetError> {
        if let Some(state) =
            lock_or_error(&self.state.tcp_connections, NetSharedState::TcpConnections)?
                .get(&connection)
                .map(|entry| entry.state)
        {
            return Ok(state);
        }

        let network = {
            let websockets = lock_or_error(
                &self.state.websocket_connections,
                NetSharedState::WebSocketConnections,
            )?;
            match websockets.get(&connection) {
                Some(crate::websocket::ManagedWebSocketConnection::Loopback(entry)) => {
                    return Ok(entry.state);
                }
                Some(crate::websocket::ManagedWebSocketConnection::Network(entry)) => {
                    Arc::clone(entry)
                }
                None => return Err(NetError::UnknownConnection { connection }),
            }
        };
        Ok(network.state())
    }

    pub(in crate::service_types) fn close_connection_impl(
        &self,
        connection: NetConnectionId,
    ) -> Result<(), NetError> {
        let mut tcp_connections =
            lock_or_error(&self.state.tcp_connections, NetSharedState::TcpConnections)?;
        if tcp_connections.contains_key(&connection) {
            self.state.worker.close_tcp(connection)?;
            tcp_connections.remove(&connection);
            return Ok(());
        }
        drop(tcp_connections);

        self.close_websocket_connection_impl(connection)
    }
}
