use zircon_runtime::core::framework::net::{NetDiagnostics, NetEvent};

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn backend_name_impl(&self) -> String {
        let mut name = "tokio-net".to_string();
        if self
            .state
            .http_backend
            .lock()
            .expect("net HTTP backend mutex poisoned")
            .is_some()
        {
            name.push_str("+http");
        }
        if self
            .state
            .websocket_backend
            .lock()
            .expect("net WebSocket backend mutex poisoned")
            .is_some()
        {
            name.push_str("+websocket");
        }
        name
    }

    pub(in crate::service_types) fn drain_events_impl(&self, max_events: usize) -> Vec<NetEvent> {
        let mut events = self.state.events.lock().expect("net events mutex poisoned");
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
        NetDiagnostics {
            backend_name: self.backend_name_impl(),
            mode: self.state.mode,
            open_udp_sockets: self
                .state
                .udp_sockets
                .lock()
                .expect("net UDP sockets mutex poisoned")
                .len(),
            open_tcp_listeners: self
                .state
                .tcp_listeners
                .lock()
                .expect("net TCP listeners mutex poisoned")
                .len(),
            open_http_listeners: self
                .state
                .http_listeners
                .lock()
                .expect("net HTTP listeners mutex poisoned")
                .len(),
            open_websocket_listeners: self
                .state
                .websocket_listeners
                .lock()
                .expect("net WebSocket listeners mutex poisoned")
                .len(),
            open_tcp_connections: self
                .state
                .tcp_connections
                .lock()
                .expect("net TCP connections mutex poisoned")
                .len(),
            open_http_routes: self
                .state
                .http_routes
                .lock()
                .expect("net HTTP routes mutex poisoned")
                .len(),
            open_websocket_connections: self
                .state
                .websocket_connections
                .lock()
                .expect("net WebSocket connections mutex poisoned")
                .len(),
            queued_events: self
                .state
                .events
                .lock()
                .expect("net events mutex poisoned")
                .len(),
        }
    }
}
