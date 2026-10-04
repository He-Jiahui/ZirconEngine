//! 独立任务把网络 message 放入连接入站队列，同时向根 manager 发布帧/状态事件。
//! 队列出队由业务显式 poll，scene 系统只发布事件；读取任务的终止决定 socket 读半部生命周期。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use tokio::runtime::Runtime;
use tokio_tungstenite::WebSocketStream;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEvent, NetTransportKind, NetWebSocketFrame,
};

use super::connection::TungsteniteWebSocketConnection;
use super::frame::message_to_frame;
use super::stream::TungsteniteWebSocketReadHalf;

pub(super) fn spawn_reader(
    runtime: &Runtime,
    connection: NetConnectionId,
    network: &TungsteniteWebSocketConnection,
    read_half: TungsteniteWebSocketReadHalf,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) {
    runtime.spawn(read_websocket_frames(
        connection,
        read_half,
        network.state.clone(),
        network.inbound.clone(),
        events,
    ));
}

async fn read_websocket_frames(
    connection: NetConnectionId,
    read_half: TungsteniteWebSocketReadHalf,
    state: Arc<Mutex<NetConnectionState>>,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) {
    match read_half {
        TungsteniteWebSocketReadHalf::Client(stream) => {
            read_stream(connection, stream, state, inbound, events).await;
        }
        TungsteniteWebSocketReadHalf::Server(stream) => {
            read_stream(connection, stream, state, inbound, events).await;
        }
    }
}

// reader 将帧交给业务 pull 队列，另把通知送入 scene 事件流；二者没有统一容量约束。
async fn read_stream<S>(
    connection: NetConnectionId,
    mut stream: SplitStream<WebSocketStream<S>>,
    state: Arc<Mutex<NetConnectionState>>,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    while let Some(message) = stream.next().await {
        match message {
            Ok(message) => {
                let frame = message_to_frame(message);
                let closing = matches!(frame, NetWebSocketFrame::Close(_));
                let mut queue = inbound
                    .lock()
                    .expect("net WebSocket inbound mutex poisoned");
                // TODO: [CR-PLUGIN-NET-0010] 入站帧和事件队列均无容量/字节上限；需明确慢消费者的背压和断开策略。
                queue.push_back(frame);
                let queued_frames = queue.len();
                drop(queue);
                events.lock().expect("net events mutex poisoned").push_back(
                    NetEvent::WebSocketFrameQueued {
                        connection,
                        queued_frames,
                    },
                );
                if closing {
                    *state.lock().expect("net WebSocket state mutex poisoned") =
                        NetConnectionState::Closed;
                    events.lock().expect("net events mutex poisoned").push_back(
                        NetEvent::ConnectionClosed {
                            connection,
                            transport: NetTransportKind::WebSocket,
                        },
                    );
                    return;
                }
            }
            Err(error) => {
                *state.lock().expect("net WebSocket state mutex poisoned") =
                    NetConnectionState::Failed;
                events.lock().expect("net events mutex poisoned").push_back(
                    NetEvent::ConnectionStateChanged {
                        connection,
                        transport: NetTransportKind::WebSocket,
                        state: NetConnectionState::Failed,
                    },
                );
                events.lock().expect("net events mutex poisoned").push_back(
                    NetEvent::WebSocketFrameQueued {
                        connection,
                        queued_frames: 0,
                    },
                );
                let _ = error;
                return;
            }
        }
    }
    *state.lock().expect("net WebSocket state mutex poisoned") = NetConnectionState::Closed;
    events
        .lock()
        .expect("net events mutex poisoned")
        .push_back(NetEvent::ConnectionClosed {
            connection,
            transport: NetTransportKind::WebSocket,
        });
}
