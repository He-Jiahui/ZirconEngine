use zircon_runtime::core::framework::net::{NetEndpoint, NetError, NetListenerId};

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn listener_endpoint_impl(
        &self,
        listener: NetListenerId,
    ) -> Result<NetEndpoint, NetError> {
        if let Some(endpoint) = self
            .state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned")
            .get(&listener)
            .map(|entry| entry.local_endpoint.clone())
        {
            return Ok(endpoint);
        }

        if let Some(endpoint) = self
            .state
            .http_listeners
            .lock()
            .expect("net HTTP listeners mutex poisoned")
            .get(&listener)
            .map(|entry| entry.local_endpoint.clone())
        {
            return Ok(endpoint);
        }

        self.state
            .websocket_listeners
            .lock()
            .expect("net WebSocket listeners mutex poisoned")
            .get(&listener)
            .map(|entry| entry.local_endpoint())
            .ok_or(NetError::UnknownListener { listener })
    }

    pub(in crate::service_types) fn close_listener_impl(
        &self,
        listener: NetListenerId,
    ) -> Result<(), NetError> {
        if self
            .state
            .tcp_listeners
            .lock()
            .expect("net TCP listeners mutex poisoned")
            .remove(&listener)
            .is_some()
        {
            return Ok(());
        }

        if self
            .state
            .http_listeners
            .lock()
            .expect("net HTTP listeners mutex poisoned")
            .remove(&listener)
            .map(|entry| {
                if let Some(abort_handle) = entry.abort_handle {
                    abort_handle.abort();
                }
            })
            .is_some()
        {
            return Ok(());
        }

        self.state
            .websocket_listeners
            .lock()
            .expect("net WebSocket listeners mutex poisoned")
            .remove(&listener)
            .map(|_| ())
            .ok_or(NetError::UnknownListener { listener })
    }
}
