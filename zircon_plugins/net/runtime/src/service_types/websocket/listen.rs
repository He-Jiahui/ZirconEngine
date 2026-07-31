use std::sync::Arc;
use std::time::Duration;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetListenerId, NetTransportKind,
    NetWebSocketListenerDescriptor,
};

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

const WEBSOCKET_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

impl DefaultNetManager {
    pub(in crate::service_types) fn listen_websocket_impl(
        &self,
        descriptor: NetWebSocketListenerDescriptor,
    ) -> Result<NetListenerId, NetError> {
        let backend = self.websocket_backend()?;
        drop(lock_or_error(
            &self.state.websocket_listeners,
            NetSharedState::WebSocketListeners,
        )?);
        let listener: Arc<dyn crate::websocket::WebSocketRuntimeListener> =
            Arc::from(backend.listen_websocket(&self.state.runtime, descriptor)?);
        let local_endpoint = listener.local_endpoint();
        let listener_id = self.next_listener_id();
        let mut listeners = lock_or_error(
            &self.state.websocket_listeners,
            NetSharedState::WebSocketListeners,
        )?;
        listeners.insert(listener_id, listener);
        self.state.push_event(NetEvent::ListenerStarted {
            listener: listener_id,
            transport: NetTransportKind::WebSocket,
            endpoint: local_endpoint,
        });
        Ok(listener_id)
    }

    pub(in crate::service_types) fn accept_websocket_impl(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<NetConnectionId>, NetError> {
        if max_connections == 0 {
            return Ok(Vec::new());
        }
        let listener_entry = {
            let listeners = lock_or_error(
                &self.state.websocket_listeners,
                NetSharedState::WebSocketListeners,
            )?;
            Arc::clone(
                listeners
                    .get(&listener)
                    .ok_or(NetError::UnknownListener { listener })?,
            )
        };
        drop(lock_or_error(
            &self.state.websocket_connections,
            NetSharedState::WebSocketConnections,
        )?);
        let mut staged = Vec::new();
        while staged.len() < max_connections {
            let connection = self.next_connection_id();
            let accepted = listener_entry.accept_websocket(
                &self.state.runtime,
                connection,
                self.state.events.clone(),
                WEBSOCKET_ACCEPT_POLL_TIMEOUT,
            );
            let (remote_endpoint, network) = match accepted {
                Ok(Some(accepted)) => accepted,
                Ok(None) => break,
                Err(error) => {
                    close_staged_connections(&staged);
                    return Err(error);
                }
            };
            let network: Arc<dyn crate::websocket::WebSocketRuntimeConnection> = Arc::from(network);
            staged.push((connection, remote_endpoint, network));
        }
        let mut connections = match lock_or_error(
            &self.state.websocket_connections,
            NetSharedState::WebSocketConnections,
        ) {
            Ok(connections) => connections,
            Err(error) => {
                close_staged_connections(&staged);
                return Err(error);
            }
        };
        for (connection, _, network) in &staged {
            connections.insert(
                *connection,
                ManagedWebSocketConnection::Network(Arc::clone(network)),
            );
        }
        drop(connections);
        let mut accepted = Vec::with_capacity(staged.len());
        for (connection, remote_endpoint, _) in staged {
            self.state.push_event(NetEvent::ConnectionAccepted {
                listener,
                connection,
                transport: NetTransportKind::WebSocket,
                remote: remote_endpoint,
            });
            self.state.push_event(NetEvent::ConnectionStateChanged {
                connection,
                transport: NetTransportKind::WebSocket,
                state: NetConnectionState::Open,
            });
            accepted.push(connection);
        }
        Ok(accepted)
    }
}

fn close_staged_connections(
    staged: &[(
        NetConnectionId,
        zircon_runtime::core::framework::net::NetEndpoint,
        Arc<dyn crate::websocket::WebSocketRuntimeConnection>,
    )],
) {
    for (_, _, connection) in staged {
        connection.set_state(NetConnectionState::Closed);
    }
}
