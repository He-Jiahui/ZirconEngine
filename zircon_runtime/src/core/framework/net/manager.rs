use super::{NetEndpoint, NetError, NetPacket, NetSocketId};

pub trait NetManager: Send + Sync {
    fn backend_name(&self) -> String;
    fn bind_udp(&self, bind: &NetEndpoint) -> Result<NetSocketId, NetError>;
    fn local_endpoint(&self, socket: NetSocketId) -> Result<NetEndpoint, NetError>;
    fn send_udp(
        &self,
        socket: NetSocketId,
        destination: &NetEndpoint,
        payload: &[u8],
    ) -> Result<usize, NetError>;
    fn poll_udp(&self, socket: NetSocketId, max_packets: usize)
        -> Result<Vec<NetPacket>, NetError>;
    fn close_socket(&self, socket: NetSocketId) -> Result<(), NetError>;
}
