use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetListenerId,
};

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::runtime_state::{ManagedTcpConnection, ManagedTcpListener};

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn listen_tcp_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetListenerId, NetError> {
        let mut listeners = lock_or_error(&self.state.tcp_listeners, NetSharedState::TcpListeners)?;
        let listener_id = self.next_listener_id();
        let local_endpoint = self.state.worker.listen_tcp(listener_id, bind.clone())?;
        listeners.insert(
            listener_id,
            ManagedTcpListener {
                local_endpoint: local_endpoint.clone(),
            },
        );
        Ok(listener_id)
    }

    pub(in crate::service_types) fn accept_tcp_impl(
        &self,
        listener: NetListenerId,
        max_connections: usize,
    ) -> Result<Vec<NetConnectionId>, NetError> {
        if max_connections == 0 {
            return Ok(Vec::new());
        }

        let listeners = lock_or_error(&self.state.tcp_listeners, NetSharedState::TcpListeners)?;
        if !listeners.contains_key(&listener) {
            return Err(NetError::UnknownListener { listener });
        }
        drop(listeners);

        let mut connections =
            lock_or_error(&self.state.tcp_connections, NetSharedState::TcpConnections)?;
        let accepted = self.state.worker.accept_tcp(listener, max_connections)?;
        let mut connection_ids = Vec::with_capacity(accepted.len());
        for accepted_connection in accepted {
            let connection = accepted_connection.connection;
            connections.insert(
                connection,
                ManagedTcpConnection {
                    state: NetConnectionState::Open,
                },
            );
            connection_ids.push(connection);
        }

        Ok(connection_ids)
    }

    pub(in crate::service_types) fn connect_tcp_impl(
        &self,
        remote: &NetEndpoint,
    ) -> Result<NetConnectionId, NetError> {
        let mut connections =
            lock_or_error(&self.state.tcp_connections, NetSharedState::TcpConnections)?;
        let connection = self.next_connection_id();
        self.state.worker.connect_tcp(connection, remote.clone())?;
        connections.insert(
            connection,
            ManagedTcpConnection {
                state: NetConnectionState::Open,
            },
        );
        Ok(connection)
    }

    pub(in crate::service_types) fn send_tcp_impl(
        &self,
        connection: NetConnectionId,
        payload: &[u8],
    ) -> Result<usize, NetError> {
        let connections =
            lock_or_error(&self.state.tcp_connections, NetSharedState::TcpConnections)?;
        if !connections.contains_key(&connection) {
            return Err(NetError::UnknownConnection { connection });
        }
        let bytes = self.state.worker.send_tcp(connection, payload.to_vec())?;
        self.state.record_outbound_bytes(bytes);
        Ok(bytes)
    }

    pub(in crate::service_types) fn poll_tcp_impl(
        &self,
        connection: NetConnectionId,
        max_bytes: usize,
    ) -> Result<Vec<u8>, NetError> {
        if max_bytes == 0 {
            return Ok(Vec::new());
        }

        let mut connections =
            lock_or_error(&self.state.tcp_connections, NetSharedState::TcpConnections)?;
        if !connections.contains_key(&connection) {
            return Err(NetError::UnknownConnection { connection });
        }
        let result = self.state.worker.poll_tcp(connection, max_bytes)?;
        if let Some(entry) = connections.get_mut(&connection) {
            entry.state = result.state;
        }
        self.state.record_inbound_bytes(result.payload.len());
        Ok(result.payload)
    }
}
