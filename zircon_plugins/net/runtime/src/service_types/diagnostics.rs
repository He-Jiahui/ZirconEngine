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
        let mut drained = Vec::new();
        while drained.len() < max_events {
            match events.pop_front() {
                Some(event) => drained.push(event),
                None => break,
            }
        }
        drained
    }

    pub(in crate::service_types) fn diagnostics_impl(&self) -> NetDiagnostics {
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
