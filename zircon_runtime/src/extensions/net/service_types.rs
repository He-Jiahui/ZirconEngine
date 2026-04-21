use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::framework::net::{NetEndpoint, NetError, NetPacket, NetSocketId};

#[derive(Clone, Debug, Default)]
pub struct NetDriver;

#[derive(Debug)]
struct ManagedUdpSocket {
    socket: UdpSocket,
    local_endpoint: NetEndpoint,
}

#[derive(Debug, Default)]
struct NetSocketState {
    next_socket_id: AtomicU64,
    sockets: Mutex<HashMap<NetSocketId, ManagedUdpSocket>>,
}

#[derive(Clone, Debug, Default)]
pub struct DefaultNetManager {
    state: Arc<NetSocketState>,
}

impl DefaultNetManager {
    fn next_socket_id(&self) -> NetSocketId {
        NetSocketId::new(self.state.next_socket_id.fetch_add(1, Ordering::Relaxed) + 1)
    }
}

impl crate::core::framework::net::NetManager for DefaultNetManager {
    fn backend_name(&self) -> String {
        "std-udp".to_string()
    }

    fn bind_udp(&self, bind: &NetEndpoint) -> Result<NetSocketId, NetError> {
        let bind_addr = bind.to_socket_addr()?;
        let socket = UdpSocket::bind(bind_addr).map_err(|error| NetError::Io(error.to_string()))?;
        socket
            .set_nonblocking(true)
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_addr = socket
            .local_addr()
            .map_err(|error| NetError::Io(error.to_string()))?;
        let socket_id = self.next_socket_id();
        self.state
            .sockets
            .lock()
            .expect("net sockets mutex poisoned")
            .insert(
                socket_id,
                ManagedUdpSocket {
                    socket,
                    local_endpoint: NetEndpoint::from(local_addr),
                },
            );
        Ok(socket_id)
    }

    fn local_endpoint(&self, socket: NetSocketId) -> Result<NetEndpoint, NetError> {
        self.state
            .sockets
            .lock()
            .expect("net sockets mutex poisoned")
            .get(&socket)
            .map(|entry| entry.local_endpoint.clone())
            .ok_or(NetError::UnknownSocket { socket })
    }

    fn send_udp(
        &self,
        socket: NetSocketId,
        destination: &NetEndpoint,
        payload: &[u8],
    ) -> Result<usize, NetError> {
        let destination = destination.to_socket_addr()?;
        let sockets = self
            .state
            .sockets
            .lock()
            .expect("net sockets mutex poisoned");
        let entry = sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;
        entry
            .socket
            .send_to(payload, destination)
            .map_err(|error| NetError::Io(error.to_string()))
    }

    fn poll_udp(
        &self,
        socket: NetSocketId,
        max_packets: usize,
    ) -> Result<Vec<NetPacket>, NetError> {
        if max_packets == 0 {
            return Ok(Vec::new());
        }

        let sockets = self
            .state
            .sockets
            .lock()
            .expect("net sockets mutex poisoned");
        let entry = sockets
            .get(&socket)
            .ok_or(NetError::UnknownSocket { socket })?;

        let mut packets = Vec::new();
        let mut buffer = vec![0_u8; u16::MAX as usize];
        while packets.len() < max_packets {
            match entry.socket.recv_from(&mut buffer) {
                Ok((received, source)) => packets.push(NetPacket {
                    source: NetEndpoint::from(source),
                    payload: buffer[..received].to_vec(),
                }),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(NetError::Io(error.to_string())),
            }
        }

        Ok(packets)
    }

    fn close_socket(&self, socket: NetSocketId) -> Result<(), NetError> {
        self.state
            .sockets
            .lock()
            .expect("net sockets mutex poisoned")
            .remove(&socket)
            .map(|_| ())
            .ok_or(NetError::UnknownSocket { socket })
    }
}
