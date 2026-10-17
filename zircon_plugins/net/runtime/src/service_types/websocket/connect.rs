//! 调用可选 feature 建立真实 WS 连接，再登记 manager 连接表并发布状态事件。
//! 后端回调可重入；登记失败写入 Closed 状态，reader 随后停止接收和发布事件。

use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetTransportKind,
    NetWebSocketConnectDescriptor,
};

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn connect_websocket_impl(
        &self,
        descriptor: NetWebSocketConnectDescriptor,
    ) -> Result<NetConnectionId, NetError> {
        let backend = self.websocket_backend()?;
        drop(lock_or_error(
            &self.state.websocket_connections,
            NetSharedState::WebSocketConnections,
        )?);
        let connection = self.next_connection_id();
        let network: Arc<dyn crate::websocket::WebSocketRuntimeConnection> =
            Arc::from(backend.connect_websocket(
                &self.state.runtime,
                connection,
                descriptor,
                self.state.events.clone(),
            )?);
        let mut connections = match lock_or_error(
            &self.state.websocket_connections,
            NetSharedState::WebSocketConnections,
        ) {
            Ok(connections) => connections,
            Err(error) => {
                network.set_state(NetConnectionState::Closed);
                return Err(error);
            }
        };
        connections.insert(
            connection,
            ManagedWebSocketConnection::Network(Arc::clone(&network)),
        );
        drop(connections);
        // Cover a drain that raced with the reader before registry publication.
        network.notify_event_capacity();
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Connecting,
        });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        Ok(connection)
    }
}
