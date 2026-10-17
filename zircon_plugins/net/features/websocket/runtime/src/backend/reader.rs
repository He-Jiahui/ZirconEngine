//! 独立任务把网络 message 放入连接入站队列，同时向根 manager 发布帧/状态事件。
//! 队列出队由业务显式 poll，scene 系统只发布事件；读取任务的终止决定 socket 读半部生命周期。

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use tokio::runtime::Runtime;
use tokio_tungstenite::WebSocketStream;
use zircon_plugin_net_runtime::NET_RUNTIME_EVENT_QUEUE_CAPACITY;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEvent, NetTransportKind, NetWebSocketFrame,
};

use super::connection::{lock_recover, TungsteniteWebSocketConnection, WebSocketReaderCancellation};
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
        network.reader_cancel.clone(),
        events,
    ));
}

async fn read_websocket_frames(
    connection: NetConnectionId,
    read_half: TungsteniteWebSocketReadHalf,
    state: Arc<Mutex<NetConnectionState>>,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    reader_cancel: Arc<WebSocketReaderCancellation>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) {
    match read_half {
        TungsteniteWebSocketReadHalf::Client(stream) => {
            read_stream(connection, stream, state, inbound, reader_cancel, events).await;
        }
        TungsteniteWebSocketReadHalf::Server(stream) => {
            read_stream(connection, stream, state, inbound, reader_cancel, events).await;
        }
    }
}

// Register capacity interest before checking either bounded queue, so a concurrent
// drain cannot be missed. Cancellation has its own notification, with no competing
// waiter consuming a capacity wakeup. No queue/state lock crosses the wait.
async fn read_stream<S>(
    connection: NetConnectionId,
    mut stream: SplitStream<WebSocketStream<S>>,
    state: Arc<Mutex<NetConnectionState>>,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    reader_cancel: Arc<WebSocketReaderCancellation>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    loop {
        let message = tokio::select! {
            _ = reader_cancel.cancelled() => return,
            message = stream.next() => message,
        };
        let Some(message) = message else { break };
        match message {
            Ok(message) => {
                let mut frame = message_to_frame(message);
                loop {
                    let capacity = reader_cancel.capacity.notified();
                    tokio::pin!(capacity);
                    capacity.as_mut().enable();
                    if reader_cancel.is_cancelled() {
                        return;
                    }
                    match queue_reader_frame(connection, &state, &inbound, &events, frame) {
                        ReaderFrameQueueResult::Queued => break,
                        ReaderFrameQueueResult::Closed => return,
                        ReaderFrameQueueResult::Full(returned_frame) => {
                            frame = returned_frame;
                            tokio::select! {
                                _ = reader_cancel.cancelled() => return,
                                _ = capacity => {},
                            }
                        }
                    }
                }
            }
            Err(_error) => {
                publish_reader_failure(connection, &state, &events, &reader_cancel).await;
                return;
            }
        }
    }
    publish_reader_closed(connection, &state, &events, &reader_cancel).await;
}

enum ReaderFrameQueueResult {
    Queued,
    Closed,
    Full(NetWebSocketFrame),
}

fn queue_reader_frame(
    connection: NetConnectionId,
    state: &Arc<Mutex<NetConnectionState>>,
    inbound: &Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    events: &Arc<Mutex<VecDeque<NetEvent>>>,
    frame: NetWebSocketFrame,
) -> ReaderFrameQueueResult {
    let mut connection_state = lock_recover(state);
    if matches!(
        *connection_state,
        NetConnectionState::Closed | NetConnectionState::Failed
    ) {
        return ReaderFrameQueueResult::Closed;
    }
    let closing = matches!(&frame, NetWebSocketFrame::Close(_));
    let mut events = lock_recover(events);
    let required_slots = if closing { 2 } else { 1 };
    if events.len().saturating_add(required_slots) > NET_RUNTIME_EVENT_QUEUE_CAPACITY {
        drop(events);
        drop(connection_state);
        return ReaderFrameQueueResult::Full(frame);
    }
    let mut queue = lock_recover(inbound);
    if queue.len() >= NET_RUNTIME_EVENT_QUEUE_CAPACITY {
        drop(queue);
        drop(events);
        drop(connection_state);
        return ReaderFrameQueueResult::Full(frame);
    }
    queue.push_back(frame);
    let queued_frames = queue.len();
    drop(queue);
    events.push_back(NetEvent::WebSocketFrameQueued {
        connection,
        queued_frames,
    });
    if closing {
        *connection_state = NetConnectionState::Closed;
        events.push_back(NetEvent::ConnectionClosed {
            connection,
            transport: NetTransportKind::WebSocket,
        });
        ReaderFrameQueueResult::Closed
    } else {
        ReaderFrameQueueResult::Queued
    }
}

async fn publish_reader_failure(
    connection: NetConnectionId,
    state: &Arc<Mutex<NetConnectionState>>,
    events: &Arc<Mutex<VecDeque<NetEvent>>>,
    reader_cancel: &WebSocketReaderCancellation,
) {
    loop {
        let capacity = reader_cancel.capacity.notified();
        tokio::pin!(capacity);
        capacity.as_mut().enable();
        let published = {
            let mut current = lock_recover(state);
            if reader_cancel.is_cancelled()
                || matches!(
                    *current,
                    NetConnectionState::Closed | NetConnectionState::Failed
                )
            {
                return;
            }
            let mut events = lock_recover(events);
            if events.len().saturating_add(2) <= NET_RUNTIME_EVENT_QUEUE_CAPACITY {
                *current = NetConnectionState::Failed;
                events.push_back(NetEvent::ConnectionStateChanged {
                    connection,
                    transport: NetTransportKind::WebSocket,
                    state: NetConnectionState::Failed,
                });
                events.push_back(NetEvent::WebSocketFrameQueued {
                    connection,
                    queued_frames: 0,
                });
                true
            } else {
                false
            }
        };
        if published {
            return;
        }
        tokio::select! {
            _ = reader_cancel.cancelled() => return,
            _ = capacity => {},
        }
    }
}

async fn publish_reader_closed(
    connection: NetConnectionId,
    state: &Arc<Mutex<NetConnectionState>>,
    events: &Arc<Mutex<VecDeque<NetEvent>>>,
    reader_cancel: &WebSocketReaderCancellation,
) {
    loop {
        let capacity = reader_cancel.capacity.notified();
        tokio::pin!(capacity);
        capacity.as_mut().enable();
        let published = {
            let mut current = lock_recover(state);
            if reader_cancel.is_cancelled()
                || matches!(
                    *current,
                    NetConnectionState::Closed | NetConnectionState::Failed
                )
            {
                return;
            }
            let mut events = lock_recover(events);
            if events.len() < NET_RUNTIME_EVENT_QUEUE_CAPACITY {
                *current = NetConnectionState::Closed;
                events.push_back(NetEvent::ConnectionClosed {
                    connection,
                    transport: NetTransportKind::WebSocket,
                });
                true
            } else {
                false
            }
        };
        if published {
            return;
        }
        tokio::select! {
            _ = reader_cancel.cancelled() => return,
            _ = capacity => {},
        }
    }
}

#[cfg(test)]
#[path = "../tests/reader_admission.rs"]
mod admission_tests;
