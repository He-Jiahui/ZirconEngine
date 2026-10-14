//! 网络 WS 连接维护共享状态、异步写队列和入站帧队列；同步 send 成功仅表示写队列接收。
//! writer task 负责实际发送与失败状态，reader task 独立处理入站/关闭事件。

use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::sync::Notify;
use tokio_tungstenite::WebSocketStream;
use zircon_plugin_net_runtime::WebSocketRuntimeConnection;
use zircon_runtime::core::framework::net::{NetConnectionState, NetError, NetWebSocketFrame};

use super::frame::frame_to_message;
use super::stream::{
    ClientWebSocketStream, ServerWebSocketStream, TungsteniteMessage, TungsteniteWebSocketReadHalf,
};

const WEBSOCKET_EGRESS_QUEUE_CAPACITY: usize = 64;

#[derive(Debug)]
pub(super) struct TungsteniteWebSocketConnection {
    pub(super) state: Arc<Mutex<NetConnectionState>>,
    outbound: WebSocketFrameSender,
    pub(super) inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    pub(super) reader_cancel: Arc<WebSocketReaderCancellation>,
}

#[derive(Debug, Default)]
pub(super) struct WebSocketReaderCancellation {
    cancelled: AtomicBool,
    pub(super) notify: Notify,
}

impl WebSocketReaderCancellation {
    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.notify.notify_waiters();
        self.notify.notify_one();
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(super) async fn cancelled(&self) {
        loop {
            let notified = self.notify.notified();
            if self.is_cancelled() {
                return;
            }
            notified.await;
        }
    }
}

impl Drop for TungsteniteWebSocketConnection {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !matches!(
            *state,
            NetConnectionState::Failed | NetConnectionState::Closed
        ) {
            *state = NetConnectionState::Closed;
        }
        self.reader_cancel.cancel();
    }
}

#[derive(Clone)]
struct WebSocketFrameSender {
    queue: mpsc::Sender<NetWebSocketFrame>,
}

impl fmt::Debug for WebSocketFrameSender {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WebSocketFrameSender")
            .finish_non_exhaustive()
    }
}

impl WebSocketRuntimeConnection for TungsteniteWebSocketConnection {
    fn state(&self) -> NetConnectionState {
        *self
            .state
            .lock()
            .expect("net WebSocket state mutex poisoned")
    }

    // Publish terminal state under the reader admission lock before waking a
    // reader that may be waiting for queue capacity.
    fn set_state(&self, state: NetConnectionState) {
        let mut current = self
            .state
            .lock()
            .expect("net WebSocket state mutex poisoned");
        *current = state;
        if matches!(
            state,
            NetConnectionState::Closed | NetConnectionState::Failed
        ) {
            self.reader_cancel.cancel();
        }
    }

    fn send(&self, _runtime: &Runtime, frame: NetWebSocketFrame) -> Result<(), NetError> {
        let closing = matches!(frame, NetWebSocketFrame::Close(_));
        self.outbound
            .queue
            .try_send(frame)
            .map_err(|error| NetError::Io(format!("websocket send queue unavailable: {error}")))?;
        if closing {
            self.set_state(NetConnectionState::Closing);
        }
        Ok(())
    }

    fn drain_frames(&self, max_frames: usize) -> Vec<NetWebSocketFrame> {
        let mut inbound = self
            .inbound
            .lock()
            .expect("net WebSocket inbound mutex poisoned");
        let drain_count = max_frames.min(inbound.len());
        let mut frames = Vec::with_capacity(drain_count);
        let mut received_close = false;
        while frames.len() < max_frames {
            match inbound.pop_front() {
                Some(NetWebSocketFrame::Close(reason)) => {
                    received_close = true;
                    frames.push(NetWebSocketFrame::Close(reason));
                }
                Some(frame) => frames.push(frame),
                None => break,
            }
        }
        drop(inbound);
        if drain_count != 0 {
            self.reader_cancel.notify.notify_one();
        }
        if received_close {
            self.set_state(NetConnectionState::Closed);
        }
        frames
    }
}

impl TungsteniteWebSocketConnection {
    pub(super) fn client(
        runtime: &Runtime,
        sink: SplitSink<ClientWebSocketStream, TungsteniteMessage>,
        stream: futures_util::stream::SplitStream<ClientWebSocketStream>,
    ) -> (Self, TungsteniteWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        let outbound = spawn_writer(runtime, state.clone(), sink);
        (
            Self {
                state,
                outbound,
                inbound,
                reader_cancel: Arc::new(WebSocketReaderCancellation::default()),
            },
            TungsteniteWebSocketReadHalf::Client(stream),
        )
    }

    pub(super) fn server(
        runtime: &Runtime,
        sink: SplitSink<ServerWebSocketStream, TungsteniteMessage>,
        stream: futures_util::stream::SplitStream<ServerWebSocketStream>,
    ) -> (Self, TungsteniteWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        let outbound = spawn_writer(runtime, state.clone(), sink);
        (
            Self {
                state,
                outbound,
                inbound,
                reader_cancel: Arc::new(WebSocketReaderCancellation::default()),
            },
            TungsteniteWebSocketReadHalf::Server(stream),
        )
    }
}

fn spawn_writer<S>(
    runtime: &Runtime,
    state: Arc<Mutex<NetConnectionState>>,
    sink: SplitSink<WebSocketStream<S>, TungsteniteMessage>,
) -> WebSocketFrameSender
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (sender, mut receiver) =
        mpsc::channel::<NetWebSocketFrame>(WEBSOCKET_EGRESS_QUEUE_CAPACITY);
    runtime.spawn(async move {
        let mut sink = sink;
        while let Some(frame) = receiver.recv().await {
            let closing = matches!(frame, NetWebSocketFrame::Close(_));
            if sink.send(frame_to_message(frame)).await.is_err() {
                *state.lock().expect("net WebSocket state mutex poisoned") =
                    NetConnectionState::Failed;
                return;
            }
            if closing {
                *state.lock().expect("net WebSocket state mutex poisoned") =
                    NetConnectionState::Closing;
                return;
            }
        }
    });
    WebSocketFrameSender { queue: sender }
}
