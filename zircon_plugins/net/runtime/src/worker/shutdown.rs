//! 记录 worker 退出时尚存 socket/连接及被丢弃的出站命令数量，供生命周期测试与宿主关闭诊断。
//! 报告仅统计 TCP/UDP worker，不覆盖异步 HTTP/WS 任务。

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
