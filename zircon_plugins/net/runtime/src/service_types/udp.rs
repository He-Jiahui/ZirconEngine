use zircon_runtime::core::framework::net::{NetEndpoint, NetError, NetPacket, NetSocketId};

use crate::runtime_state::ManagedUdpSocket;

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn bind_udp_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetSocketId, NetError> {
        let socket_id = self.next_socket_id();
        let local_endpoint = self.state.worker.bind_udp(socket_id, bind.clone())?;
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .insert(
                socket_id,
                ManagedUdpSocket {
                    local_endpoint: local_endpoint.clone(),
                },
            );
        Ok(socket_id)
    }

    pub(in crate::service_types) fn local_endpoint_impl(
        &self,
        socket: NetSocketId,
    ) -> Result<NetEndpoint, NetError> {
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .get(&socket)
            .map(|entry| entry.local_endpoint.clone())
            .ok_or(NetError::UnknownSocket { socket })
    }

    pub(in crate::service_types) fn send_udp_impl(
        &self,
        socket: NetSocketId,
        destination: &NetEndpoint,
        payload: &[u8],
    ) -> Result<usize, NetError> {
        self.state
            .worker
            .send_udp(socket, destination.clone(), payload.to_vec())
    }

    pub(in crate::service_types) fn poll_udp_impl(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        if max_packets == 0 {
            return Ok(Vec::new());
        }

        self.state.worker.poll_udp(socket, max_packets)
    }

    pub(in crate::service_types) fn close_socket_impl(
        &self,
        socket: NetSocketId,
    ) -> Result<(), NetError> {
        if !self
            .state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .contains_key(&socket)
        {
            return Err(NetError::UnknownSocket { socket });
        }

        self.state.worker.close_udp(socket)?;
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .remove(&socket);
        Ok(())
    }
}
