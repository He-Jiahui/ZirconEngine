//! 提供同一 manager 内成对 WS 连接，供无外部 socket 的帧和生命周期语义调用。
//! 此路径不证明真实握手、TLS 或后端并发行为；两个 ID 使用统一分配器。

use std::collections::VecDeque;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetTransportKind,
};

use crate::poison_recovery::{lock_or_error, NetSharedState};
use crate::websocket::{LoopbackWebSocketConnection, ManagedWebSocketConnection};

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn open_websocket_loopback_impl(
        &self,
    ) -> Result<(NetConnectionId, NetConnectionId), NetError> {
        let client = self.next_connection_id();
        let server = self.next_connection_id();
        let mut websockets = lock_or_error(
            &self.state.websocket_connections,
            NetSharedState::WebSocketConnections,
        )?;
        websockets.insert(
            client,
            ManagedWebSocketConnection::Loopback(LoopbackWebSocketConnection {
                peer: server,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            }),
        );
        websockets.insert(
            server,
            ManagedWebSocketConnection::Loopback(LoopbackWebSocketConnection {
                peer: client,
                state: NetConnectionState::Open,
                inbound: VecDeque::new(),
            }),
        );
        self.state
            .push_event(NetEvent::WebSocketPairOpened { client, server });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection: client,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        self.state.push_event(NetEvent::ConnectionStateChanged {
            connection: server,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        });
        Ok((client, server))
    }
}
