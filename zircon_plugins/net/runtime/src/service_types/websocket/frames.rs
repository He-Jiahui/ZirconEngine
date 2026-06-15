use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetWebSocketFrame,
};

use crate::websocket::ManagedWebSocketConnection;

use super::super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn send_websocket_frame_impl(
        &self,
        connection: NetConnectionId,
        frame: NetWebSocketFrame,
    ) -> Result<(), NetError> {
        let frame_bytes = websocket_frame_bytes(&frame);
        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        if let Some(ManagedWebSocketConnection::Network(entry)) = websockets.get(&connection) {
            entry.send(&self.state.runtime, frame)?;
            self.state.record_outbound_bytes(frame_bytes);
            return Ok(());
        }
        let peer = websockets
            .get(&connection)
            .and_then(|entry| match entry {
                ManagedWebSocketConnection::Loopback(entry) => Some(entry.peer),
                ManagedWebSocketConnection::Network(_) => None,
            })
            .ok_or(NetError::UnknownConnection { connection })?;
        if let NetWebSocketFrame::Close(_) = frame {
            if let Some(ManagedWebSocketConnection::Loopback(entry)) =
                websockets.get_mut(&connection)
            {
                entry.state = NetConnectionState::Closed;
            }
        }
        let peer_entry = websockets
            .get_mut(&peer)
            .and_then(|entry| match entry {
                ManagedWebSocketConnection::Loopback(entry) => Some(entry),
                ManagedWebSocketConnection::Network(_) => None,
            })
            .ok_or(NetError::UnknownConnection { connection: peer })?;
        peer_entry.inbound.push_back(frame);
        let queued_frames = peer_entry.inbound.len();
        self.state.push_event(NetEvent::WebSocketFrameQueued {
            connection: peer,
            queued_frames,
        });
        self.state.record_outbound_bytes(frame_bytes);
        Ok(())
    }

    pub(in crate::service_types) fn poll_websocket_frames_impl(
        &self,
        connection: NetConnectionId,
        max_frames: usize,
    ) -> Result<Vec<NetWebSocketFrame>, NetError> {
        let mut websockets = self
            .state
            .websocket_connections
            .lock()
            .expect("net WebSocket connections mutex poisoned");
        let entry = websockets
            .get_mut(&connection)
            .ok_or(NetError::UnknownConnection { connection })?;
        match entry {
            ManagedWebSocketConnection::Loopback(entry) => {
                let mut frames = Vec::new();
                while frames.len() < max_frames {
                    match entry.inbound.pop_front() {
                        Some(NetWebSocketFrame::Close(reason)) => {
                            entry.state = NetConnectionState::Closed;
                            frames.push(NetWebSocketFrame::Close(reason));
                        }
                        Some(frame) => frames.push(frame),
                        None => break,
                    }
                }
                self.state
                    .record_inbound_bytes(frames.iter().map(websocket_frame_bytes).sum());
                Ok(frames)
            }
            ManagedWebSocketConnection::Network(entry) => {
                let frames = entry.drain_frames(max_frames);
                self.state
                    .record_inbound_bytes(frames.iter().map(websocket_frame_bytes).sum());
                Ok(frames)
            }
        }
    }
}

fn websocket_frame_bytes(frame: &NetWebSocketFrame) -> usize {
    match frame {
        NetWebSocketFrame::Text(payload) => payload.len(),
        NetWebSocketFrame::Binary(payload) => payload.len(),
        NetWebSocketFrame::Ping(payload) => payload.len(),
        NetWebSocketFrame::Pong(payload) => payload.len(),
        NetWebSocketFrame::Close(reason) => reason.reason.len(),
    }
}
