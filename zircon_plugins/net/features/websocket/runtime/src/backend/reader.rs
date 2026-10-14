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

use super::connection::{TungsteniteWebSocketConnection, WebSocketReaderCancellation};
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

// Keep both queues bounded. A full queue holds the current frame and retries after
// yielding, without holding any queue/state lock, until capacity or cancellation.
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
                    match queue_reader_frame(connection, &state, &inbound, &events, frame) {
                        ReaderFrameQueueResult::Queued => break,
                        ReaderFrameQueueResult::Closed => return,
                        ReaderFrameQueueResult::Full(returned_frame) => {
                            frame = returned_frame;
                            tokio::select! {
                                _ = reader_cancel.cancelled() => return,
                                _ = reader_cancel.notify.notified() => {},
                                _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => {},
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
    let mut connection_state = state.lock().expect("net WebSocket state mutex poisoned");
    if matches!(
        *connection_state,
        NetConnectionState::Closed | NetConnectionState::Failed
    ) {
        return ReaderFrameQueueResult::Closed;
    }
    let closing = matches!(&frame, NetWebSocketFrame::Close(_));
    let mut events = events.lock().expect("net events mutex poisoned");
    let required_slots = if closing { 2 } else { 1 };
    if events.len().saturating_add(required_slots) > NET_RUNTIME_EVENT_QUEUE_CAPACITY {
        drop(events);
        drop(connection_state);
        return ReaderFrameQueueResult::Full(frame);
    }
    let mut queue = inbound
        .lock()
        .expect("net WebSocket inbound mutex poisoned");
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
        let published = {
            let mut current = state.lock().expect("net WebSocket state mutex poisoned");
            if reader_cancel.is_cancelled()
                || matches!(
                    *current,
                    NetConnectionState::Closed | NetConnectionState::Failed
                )
            {
                return;
            }
            let mut events = events.lock().expect("net events mutex poisoned");
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
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => {},
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
        let published = {
            let mut current = state.lock().expect("net WebSocket state mutex poisoned");
            if reader_cancel.is_cancelled()
                || matches!(
                    *current,
                    NetConnectionState::Closed | NetConnectionState::Failed
                )
            {
                return;
            }
            let mut events = events.lock().expect("net events mutex poisoned");
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
            _ = tokio::time::sleep(std::time::Duration::from_millis(1)) => {},
        }
    }
}

#[cfg(test)]
mod admission_tests {
    use super::{
        publish_reader_closed, publish_reader_failure, queue_reader_frame, read_websocket_frames,
        ReaderFrameQueueResult,
    };
    use crate::backend::connection::{TungsteniteWebSocketConnection, WebSocketReaderCancellation};
    use crate::backend::stream::TungsteniteMessage;
    use futures_util::{SinkExt, StreamExt};
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio_tungstenite::WebSocketStream;
    use zircon_plugin_net_runtime::WebSocketRuntimeConnection;
    use zircon_plugin_net_runtime::NET_RUNTIME_EVENT_QUEUE_CAPACITY;
    use zircon_runtime::core::framework::net::{
        NetConnectionId, NetConnectionState, NetEvent, NetWebSocketFrame,
    };

    async fn connected_pair(
        runtime: &Runtime,
    ) -> (
        TungsteniteWebSocketConnection,
        super::super::stream::TungsteniteWebSocketReadHalf,
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let accept = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            tokio_tungstenite::accept_async(stream).await.unwrap()
        });
        let (client, _) = tokio_tungstenite::connect_async(format!("ws://{address}/"))
            .await
            .unwrap();
        let server = accept.await.unwrap();
        let (sink, stream) = server.split();
        let (network, read_half) = TungsteniteWebSocketConnection::server(runtime, sink, stream);
        (network, read_half, client)
    }

    async fn wait_for_inbound_len(
        inbound: &Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
        expected: usize,
    ) {
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            loop {
                if inbound.lock().unwrap().len() == expected {
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("reader did not reach the expected bounded queue length");
    }

    fn test_runtime() -> Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap()
    }

    #[test]
    fn real_socket_reader_backpressures_slow_consumer_and_publishes_close_pair_after_drain() {
        let runtime = test_runtime();
        runtime.block_on(async {
            let connection = NetConnectionId::new(16);
            let (network, read_half, mut client) = connected_pair(&runtime).await;
            let events = Arc::new(Mutex::new(VecDeque::new()));
            let keep_draining_events = Arc::new(AtomicBool::new(true));
            let reader = tokio::spawn(read_websocket_frames(
                connection,
                read_half,
                Arc::clone(&network.state),
                Arc::clone(&network.inbound),
                Arc::clone(&network.reader_cancel),
                Arc::clone(&events),
            ));
            let event_drain_task = tokio::spawn({
                let events = Arc::clone(&events);
                let keep_draining_events = Arc::clone(&keep_draining_events);
                async move {
                    while keep_draining_events.load(Ordering::Acquire) {
                        events.lock().unwrap().clear();
                        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    }
                }
            });

            let sender = tokio::spawn(async move {
                for index in 0..=NET_RUNTIME_EVENT_QUEUE_CAPACITY {
                    client
                        .send(TungsteniteMessage::Text(index.to_string().into()))
                        .await
                        .unwrap();
                }
                client
            });

            wait_for_inbound_len(&network.inbound, NET_RUNTIME_EVENT_QUEUE_CAPACITY).await;
            assert!(events.lock().unwrap().len() <= NET_RUNTIME_EVENT_QUEUE_CAPACITY);
            assert_eq!(
                network.drain_frames(1),
                vec![NetWebSocketFrame::Text("0".to_string())]
            );
            wait_for_inbound_len(&network.inbound, NET_RUNTIME_EVENT_QUEUE_CAPACITY).await;
            client = tokio::time::timeout(std::time::Duration::from_secs(10), sender)
                .await
                .expect("socket writer should resume after the consumer frees one slot")
                .unwrap();
            keep_draining_events.store(false, Ordering::Release);
            event_drain_task.await.unwrap();
            {
                let mut events = events.lock().unwrap();
                // Retire the final data-frame notification before setting the close boundary.
                // Exactly one free event slot must hold the close frame without publication.
                events.clear();
                events.extend((0..NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1).map(|_| {
                    NetEvent::WebSocketFrameQueued {
                        connection,
                        queued_frames: 0,
                    }
                }));
            }
            client.send(TungsteniteMessage::Close(None)).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            assert_eq!(network.state(), NetConnectionState::Open);
            assert_eq!(
                network.inbound.lock().unwrap().len(),
                NET_RUNTIME_EVENT_QUEUE_CAPACITY
            );
            assert_eq!(
                events.lock().unwrap().len(),
                NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1
            );

            assert_eq!(
                network.drain_frames(1),
                vec![NetWebSocketFrame::Text("1".into())]
            );
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            assert_eq!(network.state(), NetConnectionState::Open);
            assert_eq!(
                network.inbound.lock().unwrap().len(),
                NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1
            );

            events.lock().unwrap().pop_front();
            tokio::time::timeout(std::time::Duration::from_secs(2), async {
                while network.state() != NetConnectionState::Closed {
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                }
            })
            .await
            .expect("reader should publish close after both event slots become free");
            {
                let events = events.lock().unwrap();
                assert_eq!(events.len(), NET_RUNTIME_EVENT_QUEUE_CAPACITY);
                assert!(matches!(
                    &events[events.len() - 2],
                    NetEvent::WebSocketFrameQueued { queued_frames, .. }
                        if *queued_frames == NET_RUNTIME_EVENT_QUEUE_CAPACITY
                ));
                assert!(matches!(
                    events.back(),
                    Some(NetEvent::ConnectionClosed { connection: event_connection, .. })
                        if *event_connection == connection
                ));
            }
            let remaining = network.drain_frames(NET_RUNTIME_EVENT_QUEUE_CAPACITY);
            assert_eq!(remaining.len(), NET_RUNTIME_EVENT_QUEUE_CAPACITY);
            assert!(
                remaining[..NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1]
                    .iter()
                    .enumerate()
                    .all(|(offset, frame)| frame
                        == &NetWebSocketFrame::Text((offset + 2).to_string()))
            );
            assert_eq!(
                remaining.last(),
                Some(&NetWebSocketFrame::Close(
                    zircon_runtime::core::framework::net::NetWebSocketCloseReason::normal(
                        "peer closed"
                    )
                ))
            );
            tokio::time::timeout(std::time::Duration::from_secs(1), reader)
                .await
                .expect("reader should finish after queueing the close frame")
                .unwrap();
        });
    }

    #[test]
    fn disconnect_cancels_real_reader_waiting_for_inbound_capacity() {
        let runtime = test_runtime();
        runtime.block_on(async {
            let connection = NetConnectionId::new(17);
            let (network, read_half, mut client) = connected_pair(&runtime).await;
            let events = Arc::new(Mutex::new(VecDeque::new()));
            let reader = tokio::spawn(read_websocket_frames(
                connection,
                read_half,
                Arc::clone(&network.state),
                Arc::clone(&network.inbound),
                Arc::clone(&network.reader_cancel),
                Arc::clone(&events),
            ));
            let sender = tokio::spawn(async move {
                for index in 0..=NET_RUNTIME_EVENT_QUEUE_CAPACITY {
                    client
                        .send(TungsteniteMessage::Text(index.to_string().into()))
                        .await
                        .unwrap();
                }
            });

            wait_for_inbound_len(&network.inbound, NET_RUNTIME_EVENT_QUEUE_CAPACITY).await;
            network.set_state(NetConnectionState::Closed);
            tokio::time::timeout(std::time::Duration::from_secs(1), reader)
                .await
                .expect("disconnect must wake the real reader while admission is full")
                .unwrap();
            assert_eq!(network.state(), NetConnectionState::Closed);
            assert_eq!(
                network.inbound.lock().unwrap().len(),
                NET_RUNTIME_EVENT_QUEUE_CAPACITY
            );
            sender.abort();
            let _ = sender.await;
        });
    }

    #[test]
    fn inbound_stays_bounded_when_events_are_drained_but_frames_are_not_consumed() {
        let connection = NetConnectionId::new(12);
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        let events = Arc::new(Mutex::new(VecDeque::new()));

        for index in 0..NET_RUNTIME_EVENT_QUEUE_CAPACITY {
            assert!(matches!(
                queue_reader_frame(
                    connection,
                    &state,
                    &inbound,
                    &events,
                    NetWebSocketFrame::Text(index.to_string()),
                ),
                ReaderFrameQueueResult::Queued
            ));
            events.lock().unwrap().clear();
        }

        let pending = NetWebSocketFrame::Text("pending".to_string());
        assert!(matches!(
            queue_reader_frame(connection, &state, &inbound, &events, pending.clone()),
            ReaderFrameQueueResult::Full(frame) if frame == pending
        ));
        assert_eq!(
            inbound.lock().unwrap().len(),
            NET_RUNTIME_EVENT_QUEUE_CAPACITY
        );
        assert!(events.lock().unwrap().is_empty());

        assert_eq!(
            inbound.lock().unwrap().pop_front(),
            Some(NetWebSocketFrame::Text("0".into()))
        );
        assert!(matches!(
            queue_reader_frame(connection, &state, &inbound, &events, pending.clone()),
            ReaderFrameQueueResult::Queued
        ));
        let inbound = inbound.lock().unwrap();
        assert_eq!(inbound.len(), NET_RUNTIME_EVENT_QUEUE_CAPACITY);
        assert_eq!(inbound.back(), Some(&pending));
    }

    #[test]
    fn close_frame_waits_for_both_event_slots_before_changing_state_or_queueing() {
        let connection = NetConnectionId::new(13);
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        let events = Arc::new(Mutex::new(
            (0..NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1)
                .map(|_| NetEvent::WebSocketFrameQueued {
                    connection,
                    queued_frames: 0,
                })
                .collect(),
        ));
        let close = NetWebSocketFrame::Close(
            zircon_runtime::core::framework::net::NetWebSocketCloseReason::normal("done"),
        );

        assert!(matches!(
            queue_reader_frame(connection, &state, &inbound, &events, close.clone()),
            ReaderFrameQueueResult::Full(frame) if frame == close
        ));
        assert_eq!(*state.lock().unwrap(), NetConnectionState::Open);
        assert!(inbound.lock().unwrap().is_empty());
        assert_eq!(
            events.lock().unwrap().len(),
            NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1
        );
    }

    #[test]
    fn failure_publication_waits_for_two_slots_and_commits_state_and_events_together() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            let connection = NetConnectionId::new(14);
            let state = Arc::new(Mutex::new(NetConnectionState::Open));
            let events = Arc::new(Mutex::new(
                (0..NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1)
                    .map(|_| NetEvent::WebSocketFrameQueued {
                        connection,
                        queued_frames: 0,
                    })
                    .collect(),
            ));
            let cancellation = Arc::new(WebSocketReaderCancellation::default());
            let task = tokio::spawn({
                let state = Arc::clone(&state);
                let events = Arc::clone(&events);
                let cancellation = Arc::clone(&cancellation);
                async move {
                    publish_reader_failure(connection, &state, &events, &cancellation).await;
                }
            });
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            assert!(
                !task.is_finished(),
                "one free slot must not partially publish a failure"
            );
            assert_eq!(*state.lock().unwrap(), NetConnectionState::Open);
            assert_eq!(
                events.lock().unwrap().len(),
                NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1
            );

            events.lock().unwrap().pop_front();
            tokio::time::timeout(std::time::Duration::from_secs(1), task)
                .await
                .expect("failure publication must resume after two slots open")
                .unwrap();
            assert_eq!(*state.lock().unwrap(), NetConnectionState::Failed);
            let events = events.lock().unwrap();
            assert_eq!(events.len(), NET_RUNTIME_EVENT_QUEUE_CAPACITY);
            assert!(matches!(
                &events[events.len() - 2],
                NetEvent::ConnectionStateChanged {
                    state: NetConnectionState::Failed,
                    ..
                }
            ));
            assert!(matches!(
                events.back(),
                Some(NetEvent::WebSocketFrameQueued {
                    queued_frames: 0,
                    ..
                })
            ));
        });
    }

    #[test]
    fn eof_publication_is_bounded_and_disconnect_cancels_a_full_queue_wait() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            let connection = NetConnectionId::new(15);
            let state = Arc::new(Mutex::new(NetConnectionState::Open));
            let events = Arc::new(Mutex::new(
                (0..NET_RUNTIME_EVENT_QUEUE_CAPACITY)
                    .map(|_| NetEvent::WebSocketFrameQueued {
                        connection,
                        queued_frames: 0,
                    })
                    .collect(),
            ));
            let cancellation = Arc::new(WebSocketReaderCancellation::default());
            let task = tokio::spawn({
                let state = Arc::clone(&state);
                let events = Arc::clone(&events);
                let cancellation = Arc::clone(&cancellation);
                async move {
                    publish_reader_closed(connection, &state, &events, &cancellation).await;
                }
            });

            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            assert!(!task.is_finished());
            assert_eq!(*state.lock().unwrap(), NetConnectionState::Open);
            assert_eq!(
                events.lock().unwrap().len(),
                NET_RUNTIME_EVENT_QUEUE_CAPACITY
            );

            *state.lock().unwrap() = NetConnectionState::Closed;
            cancellation.cancel();
            tokio::time::timeout(std::time::Duration::from_secs(1), task)
                .await
                .expect("disconnect must wake terminal publication")
                .unwrap();
            assert_eq!(*state.lock().unwrap(), NetConnectionState::Closed);
            assert_eq!(
                events.lock().unwrap().len(),
                NET_RUNTIME_EVENT_QUEUE_CAPACITY
            );
        });
    }
}
