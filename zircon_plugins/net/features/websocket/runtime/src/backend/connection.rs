use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use tokio::runtime::Runtime;
use zircon_plugin_net_runtime::WebSocketRuntimeConnection;
use zircon_runtime::core::framework::net::{NetConnectionState, NetError, NetWebSocketFrame};

use super::frame::frame_to_message;
use super::stream::{
    ClientWebSocketStream, ServerWebSocketStream, TungsteniteMessage, TungsteniteWebSocketReadHalf,
};

#[derive(Debug)]
pub(super) struct TungsteniteWebSocketConnection {
    pub(super) state: Arc<Mutex<NetConnectionState>>,
    outbound: TungsteniteWebSocketSink,
    pub(super) inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
}

#[derive(Debug)]
enum TungsteniteWebSocketSink {
    Client(Arc<Mutex<SplitSink<ClientWebSocketStream, TungsteniteMessage>>>),
    Server(Arc<Mutex<SplitSink<ServerWebSocketStream, TungsteniteMessage>>>),
}

impl WebSocketRuntimeConnection for TungsteniteWebSocketConnection {
    fn state(&self) -> NetConnectionState {
        *self
            .state
            .lock()
            .expect("net WebSocket state mutex poisoned")
    }

    fn set_state(&self, state: NetConnectionState) {
        *self
            .state
            .lock()
            .expect("net WebSocket state mutex poisoned") = state;
    }

    fn send(&self, runtime: &Runtime, frame: NetWebSocketFrame) -> Result<(), NetError> {
        runtime.block_on(self.send_async(frame))
    }

    fn drain_frames(&self, max_frames: usize) -> Vec<NetWebSocketFrame> {
        let mut inbound = self
            .inbound
            .lock()
            .expect("net WebSocket inbound mutex poisoned");
        let mut frames = Vec::new();
        while frames.len() < max_frames {
            match inbound.pop_front() {
                Some(NetWebSocketFrame::Close(reason)) => {
                    *self
                        .state
                        .lock()
                        .expect("net WebSocket state mutex poisoned") = NetConnectionState::Closed;
                    frames.push(NetWebSocketFrame::Close(reason));
                }
                Some(frame) => frames.push(frame),
                None => break,
            }
        }
        frames
    }
}

impl TungsteniteWebSocketConnection {
    pub(super) fn client(
        sink: SplitSink<ClientWebSocketStream, TungsteniteMessage>,
        stream: futures_util::stream::SplitStream<ClientWebSocketStream>,
    ) -> (Self, TungsteniteWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        (
            Self {
                state,
                outbound: TungsteniteWebSocketSink::Client(Arc::new(Mutex::new(sink))),
                inbound,
            },
            TungsteniteWebSocketReadHalf::Client(stream),
        )
    }

    pub(super) fn server(
        sink: SplitSink<ServerWebSocketStream, TungsteniteMessage>,
        stream: futures_util::stream::SplitStream<ServerWebSocketStream>,
    ) -> (Self, TungsteniteWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        (
            Self {
                state,
                outbound: TungsteniteWebSocketSink::Server(Arc::new(Mutex::new(sink))),
                inbound,
            },
            TungsteniteWebSocketReadHalf::Server(stream),
        )
    }

    async fn send_async(&self, frame: NetWebSocketFrame) -> Result<(), NetError> {
        let message = frame_to_message(frame.clone());
        match &self.outbound {
            TungsteniteWebSocketSink::Client(sink) => {
                let mut sink = sink
                    .lock()
                    .expect("net WebSocket client sink mutex poisoned");
                sink.send(message)
                    .await
                    .map_err(|error| NetError::Io(error.to_string()))?;
            }
            TungsteniteWebSocketSink::Server(sink) => {
                let mut sink = sink
                    .lock()
                    .expect("net WebSocket server sink mutex poisoned");
                sink.send(message)
                    .await
                    .map_err(|error| NetError::Io(error.to_string()))?;
            }
        }
        if matches!(frame, NetWebSocketFrame::Close(_)) {
            self.set_state(NetConnectionState::Closing);
        }
        Ok(())
    }
}
