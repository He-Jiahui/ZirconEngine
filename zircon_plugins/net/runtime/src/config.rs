//! 描述网络运行模式与 TCP/UDP 轮询预算，预期由启动链形成有效配置快照。
//! 目前生产入口仅导出类型，manager/worker/scene 系统仍使用硬编码默认值。

use zircon_runtime::core::framework::net::NetRuntimeMode;

#[derive(Clone, Debug, Default)]
// TODO: [CR-PLUGIN-NET-0005] 启动链尚未消费 enabled、runtime_mode 或轮询预算；确认有效配置的 owner 与校验/更新边界。
pub struct NetConfig {
    pub enabled: bool,
    pub runtime_mode: NetRuntimeMode,
    pub tcp_poll_budget_bytes: usize,
    pub udp_poll_budget_packets: usize,
}

impl NetConfig {
    pub fn client() -> Self {
        Self {
            enabled: true,
            runtime_mode: NetRuntimeMode::Client,
            tcp_poll_budget_bytes: 65_536,
            udp_poll_budget_packets: 64,
        }
    }
}
