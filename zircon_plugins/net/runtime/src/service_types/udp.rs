use zircon_runtime::core::framework::net::{NetEndpoint, NetError, NetPacket, NetSocketId};

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::runtime_state::ManagedUdpSocket;

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn bind_udp_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetSocketId, NetError> {
        let mut sockets = lock_or_error(&self.state.udp_sockets, NetSharedState::UdpSockets)?;
        let socket_id = self.next_socket_id();
        let local_endpoint = self.state.worker.bind_udp(socket_id, bind.clone())?;
        sockets.insert(
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
        lock_or_error(&self.state.udp_sockets, NetSharedState::UdpSockets)?
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
        let sockets = lock_or_error(&self.state.udp_sockets, NetSharedState::UdpSockets)?;
        if !sockets.contains_key(&socket) {
            return Err(NetError::UnknownSocket { socket });
        }
        let bytes = self
            .state
            .worker
            .send_udp(socket, destination.clone(), payload.to_vec())?;
        self.state.record_outbound_bytes(bytes);
        Ok(bytes)
    }

    pub(in crate::service_types) fn poll_udp_impl(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        if max_packets == 0 {
            return Ok(Vec::new());
        }

        let sockets = lock_or_error(&self.state.udp_sockets, NetSharedState::UdpSockets)?;
        if !sockets.contains_key(&socket) {
            return Err(NetError::UnknownSocket { socket });
        }
        let packets = self.state.worker.poll_udp(socket, max_packets)?;
        self.state
            .record_inbound_bytes(packets.iter().map(|packet| packet.payload.len()).sum());
        Ok(packets)
    }

    pub(in crate::service_types) fn close_socket_impl(
        &self,
        socket: NetSocketId,
    ) -> Result<(), NetError> {
        let mut sockets = lock_or_error(&self.state.udp_sockets, NetSharedState::UdpSockets)?;
        if !sockets.contains_key(&socket) {
            return Err(NetError::UnknownSocket { socket });
        }

        self.state.worker.close_udp(socket)?;
        sockets.remove(&socket);
        Ok(())
    }
}
