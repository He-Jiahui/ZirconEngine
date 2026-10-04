//! 撤销 WebSocket 连接登记；网络对象在表锁外收到 Closed 状态请求，loopback 向对端排入关闭帧。
//! 关闭事件表示连接表已移除，真实 reader 的退出尚未由该状态请求保证，见 CR-PLUGIN-NET-0014。

use std::sync::Arc;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetTransportKind,
    NetWebSocketCloseReason, NetWebSocketFrame,
};

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn close_websocket_connection_impl(
        &self,
        connection: NetConnectionId,
    ) -> Result<(), NetError> {
        let network = {
            let mut websockets = lock_or_error(
                &self.state.websocket_connections,
                NetSharedState::WebSocketConnections,
            )?;
            let entry = websockets
                .remove(&connection)
                .ok_or(NetError::UnknownConnection { connection })?;
            match entry {
                ManagedWebSocketConnection::Loopback(entry) => {
                    if let Some(ManagedWebSocketConnection::Loopback(peer_entry)) =
                        websockets.get_mut(&entry.peer)
                    {
                        peer_entry.state = NetConnectionState::Closed;
                        peer_entry.inbound.push_back(NetWebSocketFrame::Close(
                            NetWebSocketCloseReason::normal("peer closed"),
                        ));
                    }
                    None
                }
                ManagedWebSocketConnection::Network(entry) => Some(Arc::clone(&entry)),
            }
        };
        if let Some(network) = network {
            network.set_state(NetConnectionState::Closed);
        }
        self.state.push_event(NetEvent::ConnectionClosed {
            connection,
            transport: NetTransportKind::WebSocket,
        });
        Ok(())
    }
}
