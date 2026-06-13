use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetListenerId,
};

use crate::runtime_state::{ManagedTcpConnection, ManagedTcpListener};

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn listen_tcp_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetListenerId, NetError> {
        let listener_id = self.next_listener_id();
        let local_endpoint = self.state.worker.listen_tcp(listener_id, bind.clone())?;
        self.state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned")
            .insert(
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

        if !self
            .state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned")
            .contains_key(&listener)
        {
            return Err(NetError::UnknownListener { listener });
        }

        let accepted = self.state.worker.accept_tcp(listener, max_connections)?;
        let mut connection_ids = Vec::with_capacity(accepted.len());
        let mut connections = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned");
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
        let connection = self.next_connection_id();
        self.state.worker.connect_tcp(connection, remote.clone())?;
        self.state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned")
            .insert(
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
        self.state.worker.send_tcp(connection, payload.to_vec())
    }

    pub(in crate::service_types) fn poll_tcp_impl(
        &self,
        connection: NetConnectionId,
        max_bytes: usize,
    ) -> Result<Vec<u8>, NetError> {
        if max_bytes == 0 {
            return Ok(Vec::new());
        }

        let result = self.state.worker.poll_tcp(connection, max_bytes)?;
        if let Some(entry) = self
            .state
            .tcp_connections
            .lock()
            .expect("net TCP connections mutex poisoned")
            .get_mut(&connection)
        {
            entry.state = result.state;
        }
        Ok(result.payload)
    }
}
