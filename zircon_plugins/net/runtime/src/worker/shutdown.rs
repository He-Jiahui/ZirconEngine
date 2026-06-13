#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NetWorkerShutdownReport {
    pub(crate) drained_egress_commands: usize,
    pub(crate) open_udp_sockets_closed: usize,
    pub(crate) open_tcp_listeners_closed: usize,
    pub(crate) open_tcp_connections_closed: usize,
}

impl NetWorkerShutdownReport {
    pub(crate) fn open_handles_closed(&self) -> usize {
        self.open_udp_sockets_closed
            + self.open_tcp_listeners_closed
            + self.open_tcp_connections_closed
    }
}
