use zircon_runtime::core::framework::net::NetRuntimeMode;

#[derive(Clone, Debug, Default)]
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
