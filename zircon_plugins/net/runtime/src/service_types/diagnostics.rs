//! 向 NetManager 提供事件出队和诊断快照；worker ingress 与 HTTP/WS 本地事件汇聚到 manager。
//! 诊断调用会先消费 worker ingress 并写入主事件队列，scene First 阶段先诊断后发布事件；读取快照并非无副作用。

use std::collections::VecDeque;

use zircon_runtime::core::framework::net::{NetDiagnostics, NetEvent};

use crate::poison_recovery::lock_recover;

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn backend_name_impl(&self) -> String {
        let mut name = "tokio-net".to_string();
        if lock_recover(&self.state.http_backend).is_some() {
            name.push_str("+http");
        }
        if lock_recover(&self.state.websocket_backend).is_some() {
            name.push_str("+websocket");
        }
        name
    }

    pub(in crate::service_types) fn drain_events_impl(&self, max_events: usize) -> Vec<NetEvent> {
        self.state.poll_worker_ingress(max_events);
        let mut events = lock_recover(&self.state.events);
        drain_bounded_events(&mut events, max_events)
    }

    pub(in crate::service_types) fn diagnostics_impl(&self) -> NetDiagnostics {
        // BUG: [CR-PLUGIN-NET-0002] 诊断先以 usize::MAX 搬运 ingress，可超过 scene 的 256 条发布预算；主 VecDeque 没有容量约束。
        self.state.poll_worker_ingress(usize::MAX);
        let (outbound_bytes, inbound_bytes, last_observed_latency_ms) =
            self.state.diagnostic_counters();
        NetDiagnostics {
            backend_name: self.backend_name_impl(),
            mode: self.state.mode,
            outbound_bytes,
            inbound_bytes,
            last_observed_latency_ms,
            open_udp_sockets: lock_recover(&self.state.udp_sockets).len(),
            open_tcp_listeners: lock_recover(&self.state.tcp_listeners).len(),
            open_http_listeners: lock_recover(&self.state.http_listeners).len(),
            open_websocket_listeners: lock_recover(&self.state.websocket_listeners).len(),
            open_tcp_connections: lock_recover(&self.state.tcp_connections).len(),
            open_http_routes: lock_recover(&self.state.http_routes).len(),
            open_websocket_connections: lock_recover(&self.state.websocket_connections).len(),
            queued_events: lock_recover(&self.state.events).len(),
        }
    }
}

fn drain_bounded_events(events: &mut VecDeque<NetEvent>, max_events: usize) -> Vec<NetEvent> {
    let drain_count = max_events.min(events.len());
    let mut drained = Vec::with_capacity(drain_count);
    drained.extend(events.drain(..drain_count));
    drained
}

#[cfg(test)]
#[path = "tests/diagnostics_bounded_event_drain_tests.rs"]
mod bounded_event_drain_tests;
