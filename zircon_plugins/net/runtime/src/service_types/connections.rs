use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent,
};

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn connection_state_impl(
        &self,
        connection: NetConnectionId,
    ) -> Result<NetConnectionState, NetError> {
        if let Some(state) = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned")
            .get(&connection)
            .map(|entry| entry.state)
        {
            return Ok(state);
        }

        self.state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned")
            .get(&connection)
            .map(|entry| entry.state())
            .ok_or(NetError::UnknownConnection { connection })
    }

    pub(in crate::service_types) fn close_connection_impl(
        &self,
        connection: NetConnectionId,
    ) -> Result<(), NetError> {
        if let Some(entry) = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned")
            .remove(&connection)
        {
            let _ = entry.stream;
            self.state
                .push_event(NetEvent::ConnectionClosed { connection });
            return Ok(());
        }

        self.close_websocket_connection_impl(connection)
    }
}
