use std::time::Duration;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetListenerId, NetTransportKind,
    NetWebSocketListenerDescriptor,
};

use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

const WEBSOCKET_ACCEPT_POLL_TIMEOUT: Duration = Duration::from_millis(1);

impl DefaultNetManager {
    pub(in crate::service_types) fn listen_websocket_impl(
        &self,
        descriptor: NetWebSocketListenerDescriptor,
    ) -> Result<NetListenerId, NetError> {
        let listener = self
            .websocket_backend()?
            .listen_websocket(&self.state.runtime, descriptor)?;
        let local_endpoint = listener.local_endpoint();
        let listener_id = self.next_listener_id();
        self.state
            .websocket_listeners
            .lock()
            .expect("net WebSocket listeners mutex poisoned")
            .insert(listener_id, listener);
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
        let listeners = self
            .state
            .websocket_listeners
            .lock()
            .expect("net WebSocket listeners mutex poisoned");
        let listener_entry = listeners
            .get(&listener)
            .ok_or(NetError::UnknownListener { listener })?;
        let mut accepted = Vec::new();
        while accepted.len() < max_connections {
            let connection = self.next_connection_id();
            let Some((remote_endpoint, network)) = listener_entry.accept_websocket(
                &self.state.runtime,
                connection,
                self.state.events.clone(),
                WEBSOCKET_ACCEPT_POLL_TIMEOUT,
            )?
            else {
                break;
            };
            self.state
                .websocket_connections
                .lock()
                .expect("net WebSocket connections mutex poisoned")
                .insert(connection, ManagedWebSocketConnection::Network(network));
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
