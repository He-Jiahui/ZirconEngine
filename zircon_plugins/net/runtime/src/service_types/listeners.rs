use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetListenerId, NetTransportKind,
};

use crate::poison_recovery::{lock_or_error, NetSharedState};

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn listener_endpoint_impl(
        &self,
        listener: NetListenerId,
    ) -> Result<NetEndpoint, NetError> {
        if let Some(endpoint) =
            lock_or_error(&self.state.tcp_listeners, NetSharedState::TcpListeners)?
                .get(&listener)
                .map(|entry| entry.local_endpoint.clone())
        {
            return Ok(endpoint);
        }

        if let Some(endpoint) =
            lock_or_error(&self.state.http_listeners, NetSharedState::HttpListeners)?
                .get(&listener)
                .map(|entry| entry.local_endpoint.clone())
        {
            return Ok(endpoint);
        }

        let websocket_listener = {
            let listeners = lock_or_error(
                &self.state.websocket_listeners,
                NetSharedState::WebSocketListeners,
            )?;
            Arc::clone(
                listeners
                    .get(&listener)
                    .ok_or(NetError::UnknownListener { listener })?,
            )
        };
        Ok(websocket_listener.local_endpoint())
    }

    pub(in crate::service_types) fn close_listener_impl(
        &self,
        listener: NetListenerId,
    ) -> Result<(), NetError> {
        let mut tcp_listeners =
            lock_or_error(&self.state.tcp_listeners, NetSharedState::TcpListeners)?;
        if tcp_listeners.contains_key(&listener) {
            self.state.worker.close_tcp_listener(listener)?;
            tcp_listeners.remove(&listener);
            return Ok(());
        }
        drop(tcp_listeners);

        let http_listener = {
            let mut listeners =
                lock_or_error(&self.state.http_listeners, NetSharedState::HttpListeners)?;
            listeners.remove(&listener)
        };
        if let Some(http_listener) = http_listener {
            if let Some(abort_handle) = http_listener.abort_handle {
                abort_handle.abort();
            }
            self.state.push_event(NetEvent::ListenerClosed {
                listener,
                transport: NetTransportKind::Http,
            });
            return Ok(());
        }

        let websocket_listener = {
            let mut listeners = lock_or_error(
                &self.state.websocket_listeners,
                NetSharedState::WebSocketListeners,
            )?;
            listeners.remove(&listener)
        };
        if let Some(websocket_listener) = websocket_listener {
            drop(websocket_listener);
            self.state.push_event(NetEvent::ListenerClosed {
                listener,
                transport: NetTransportKind::WebSocket,
            });
            return Ok(());
        }

        Err(NetError::UnknownListener { listener })
    }
}
