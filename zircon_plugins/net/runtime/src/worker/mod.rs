//! 按命令、事件、生命周期与 socket 实现拆分 net worker；根 manager 只经 NetWorker facade 请求 I/O。

#[path = "egress.rs"]
mod egress;
#[path = "ingress.rs"]
mod ingress;
#[path = "net_worker.rs"]
mod net_worker;
#[path = "shutdown.rs"]
mod shutdown;
#[path = "transport_runtime.rs"]
mod transport_runtime;

pub(crate) use self::egress::{AcceptedTcpConnection, TcpPollResult};
pub(crate) use self::net_worker::NetWorker;
pub(crate) use self::shutdown::NetWorkerShutdownReport;
