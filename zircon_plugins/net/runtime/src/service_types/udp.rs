use std::io::ErrorKind;

use tokio::net::UdpSocket;
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetPacket, NetSocketId,
};

use crate::runtime_state::ManagedUdpSocket;

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn bind_udp_impl(
        &self,
        bind: &NetEndpoint,
    ) -> Result<NetSocketId, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let socket = self
            .state
            .runtime
            .block_on(UdpSocket::bind(bind_addr))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = socket
            .local_addr()
            .map(Self::endpoint_from_addr)
            .map_err(|error| NetError::Io(error.to_string()))?;
        let socket_id = self.next_socket_id();
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .insert(
                socket_id,
                ManagedUdpSocket {
                    socket,
                    local_endpoint: local_endpoint.clone(),
                },
            );
        self.state.push_event(NetEvent::UdpSocketBound {
            socket: socket_id,
            endpoint: local_endpoint,
        });
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
        let destination = destination.to_socket_addr()?;
        let sockets = self
            .state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned");
        let entry = sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;
        entry
            .socket
            .try_send_to(payload, destination)
            .map_err(|error| NetError::Io(error.to_string()))
    }

    pub(in crate::service_types) fn poll_udp_impl(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        if max_packets == 0 {
            return Ok(Vec::new());
        }

        let sockets = self
            .state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned");
        let entry = sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;

        let mut packets = Vec::new();
        let mut buffer = vec![0_u8; u16::MAX as usize];
        while packets.len() < max_packets {
            match entry.socket.try_recv_from(&mut buffer) {
                Ok((received, source)) => packets.push(NetPacket {
                    source: Self::endpoint_from_addr(source),
                    payload: buffer[..received].to_vec(),
                }),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(NetError::Io(error.to_string())),
            }
        }

        Ok(packets)
    }

    pub(in crate::service_types) fn close_socket_impl(
        &self,
        socket: NetSocketId,
    ) -> Result<(), NetError> {
        self.state
            .udp_sockets
            .lock()
            .expect("net UDP sockets mutex poisoned")
            .remove(&socket)
            .map(|_| ())
            .ok_or(NetError::UnknownSocket { socket })
    }
}
