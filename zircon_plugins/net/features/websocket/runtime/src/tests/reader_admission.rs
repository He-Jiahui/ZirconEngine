use super::{
    publish_reader_closed, publish_reader_failure, queue_reader_frame, read_websocket_frames,
    ReaderFrameQueueResult,
};
use crate::backend::connection::{
    lock_recover, TungsteniteWebSocketConnection, WebSocketReaderCancellation,
};
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

#[test]
fn capacity_wakeup_is_retained_without_completing_cancellation() {
    test_runtime().block_on(async {
        let cancellation = WebSocketReaderCancellation::default();
        let capacity = cancellation.capacity.notified();
        tokio::pin!(capacity);
        capacity.as_mut().enable();
        let cancelled = cancellation.cancelled();
        tokio::pin!(cancelled);
        assert!(futures_util::poll!(cancelled.as_mut()).is_pending());

        // Drain races after interest is registered but before the reader awaits.
        cancellation.notify_capacity();
        tokio::time::timeout(std::time::Duration::from_secs(1), capacity)
            .await
            .expect("the capacity wakeup must survive until the reader waits");
        assert!(futures_util::poll!(cancelled.as_mut()).is_pending());
        cancellation.cancel();
        tokio::time::timeout(std::time::Duration::from_secs(1), cancelled)
            .await
            .expect("cancellation must wake independently of capacity");
    });
}

#[test]
fn poisoned_primitive_buffers_do_not_cascade_into_reader_panics() {
    fn poison<T>(mutex: &Mutex<T>) {
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("poison a primitive reader buffer");
        }))
        .is_err());
    }
    let connection = NetConnectionId::new(18);
    let state = Arc::new(Mutex::new(NetConnectionState::Open));
    let inbound = Arc::new(Mutex::new(VecDeque::new()));
    let events = Arc::new(Mutex::new(VecDeque::new()));
    poison(&state);
    poison(&inbound);
    poison(&events);
    assert!(matches!(
        queue_reader_frame(
            connection,
            &state,
            &inbound,
            &events,
            NetWebSocketFrame::Text("survives".into()),
        ),
        ReaderFrameQueueResult::Queued
    ));
    assert_eq!(lock_recover(&inbound).len(), 1);
    assert_eq!(lock_recover(&events).len(), 1);
}

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

async fn wait_for_inbound_len(inbound: &Arc<Mutex<VecDeque<NetWebSocketFrame>>>, expected: usize) {
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
            let cancellation = Arc::clone(&network.reader_cancel);
            async move {
                while keep_draining_events.load(Ordering::Acquire) {
                    events.lock().unwrap().clear();
                    cancellation.notify_capacity();
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
        network.notify_event_capacity();
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
        assert!(remaining[..NET_RUNTIME_EVENT_QUEUE_CAPACITY - 1]
            .iter()
            .enumerate()
            .all(|(offset, frame)| frame == &NetWebSocketFrame::Text((offset + 2).to_string())));
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
        cancellation.notify_capacity();
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

#[test]
fn root_disconnect_stops_real_reader_without_post_close_orphan_events() {
    use zircon_plugin_net_runtime::{
        DefaultNetManager, WebSocketRuntimeBackend, WebSocketRuntimeListener,
    };
    use zircon_runtime::core::framework::net::{
        NetError, NetManager, NetWebSocketConnectDescriptor, NetWebSocketListenerDescriptor,
    };

    #[derive(Debug)]
    struct Observation {
        state: Arc<Mutex<NetConnectionState>>,
        inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
        cancellation: Arc<WebSocketReaderCancellation>,
        reader: Option<tokio::task::JoinHandle<()>>,
        peer: Option<tokio::task::JoinHandle<()>>,
        release: Option<tokio::sync::oneshot::Sender<()>>,
    }

    // Keep every task owned even when an assertion or the root call fails.
    impl Drop for Observation {
        fn drop(&mut self) {
            if let Some(release) = self.release.take() {
                let _ = release.send(());
            }
            for task in [&self.reader, &self.peer].into_iter().flatten() {
                task.abort();
            }
            test_runtime().block_on(async {
                for task in [&mut self.reader, &mut self.peer] {
                    if let Some(task) = task.take() {
                        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), task).await;
                    }
                }
            });
        }
    }

    #[derive(Debug)]
    struct ObservedBackend(Arc<Mutex<Option<Observation>>>);

    impl WebSocketRuntimeBackend for ObservedBackend {
        fn listen_websocket(
            &self,
            _runtime: &Runtime,
            _descriptor: NetWebSocketListenerDescriptor,
        ) -> Result<Box<dyn WebSocketRuntimeListener>, NetError> {
            Err(NetError::Io("this fixture only connects".to_string()))
        }

        fn connect_websocket(
            &self,
            runtime: &Runtime,
            connection: NetConnectionId,
            _descriptor: NetWebSocketConnectDescriptor,
            events: Arc<Mutex<VecDeque<NetEvent>>>,
        ) -> Result<Box<dyn WebSocketRuntimeConnection>, NetError> {
            // Use a real TCP/WebSocket pair, retaining the actual production
            // connection and reader rather than a simulated set_state callback.
            let (network, read_half, mut peer) = runtime.block_on(async {
                tokio::time::timeout(std::time::Duration::from_secs(5), connected_pair(runtime))
                    .await
                    .expect("local WebSocket handshake must finish")
            });
            let (release, released) = tokio::sync::oneshot::channel();
            let peer = runtime.spawn(async move {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    peer.send(TungsteniteMessage::Text("before close".into())),
                )
                .await;
                if tokio::time::timeout(std::time::Duration::from_secs(5), released)
                    .await
                    .is_ok_and(|result| result.is_ok())
                {
                    // The peer still owns its socket after root close returns.
                    // Sending may fail because the cancelled reader dropped it.
                    let _ = tokio::time::timeout(
                        std::time::Duration::from_secs(2),
                        peer.send(TungsteniteMessage::Text("after close".into())),
                    )
                    .await;
                }
            });
            let reader = runtime.spawn(read_websocket_frames(
                connection,
                read_half,
                Arc::clone(&network.state),
                Arc::clone(&network.inbound),
                Arc::clone(&network.reader_cancel),
                events,
            ));
            *self.0.lock().unwrap() = Some(Observation {
                state: Arc::clone(&network.state),
                inbound: Arc::clone(&network.inbound),
                cancellation: Arc::clone(&network.reader_cancel),
                reader: Some(reader),
                peer: Some(peer),
                release: Some(release),
            });
            Ok(Box::new(network))
        }
    }

    let observation = Arc::new(Mutex::new(None));
    let net = DefaultNetManager::default()
        .with_websocket_backend(Arc::new(ObservedBackend(Arc::clone(&observation))));
    let connected = net.connect_websocket(NetWebSocketConnectDescriptor::new(
        "ws://127.0.0.1/observed-reader",
    ));
    // Take task ownership before checking the root result so failure also cleans up.
    let mut observed = observation
        .lock()
        .unwrap()
        .take()
        .expect("backend must run");
    let connection = connected.unwrap();
    let runtime = test_runtime();
    runtime.block_on(wait_for_inbound_len(&observed.inbound, 1));
    assert_eq!(*observed.state.lock().unwrap(), NetConnectionState::Open);
    assert!(net.drain_events(usize::MAX).iter().any(|event| matches!(
        event,
        NetEvent::WebSocketFrameQueued { connection: id, .. } if *id == connection
    )));

    net.close_connection(connection).unwrap();
    assert_eq!(*observed.state.lock().unwrap(), NetConnectionState::Closed);
    assert!(observed.cancellation.is_cancelled());
    assert!(matches!(
        net.connection_state(connection),
        Err(NetError::UnknownConnection { connection: id }) if id == connection
    ));
    runtime.block_on(async {
        let reader = observed.reader.as_mut().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), reader)
            .await
            .expect("root close must terminate the actual reader")
            .unwrap();
    });
    observed.reader.take();
    let events = net.drain_events(usize::MAX);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(
                event,
                NetEvent::ConnectionClosed { connection: id, .. } if *id == connection
            ))
            .count(),
        1
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        NetEvent::WebSocketFrameQueued { connection: id, .. } if *id == connection
    )));
    let inbound_after_close = observed.inbound.lock().unwrap().clone();
    observed.release.take().unwrap().send(()).unwrap();
    runtime.block_on(async {
        let peer = observed.peer.as_mut().unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), peer)
            .await
            .expect("the controlled peer must finish after release")
            .unwrap();
    });
    observed.peer.take();
    assert!(net.drain_events(usize::MAX).is_empty());
    assert_eq!(*observed.inbound.lock().unwrap(), inbound_after_close);
    assert_eq!(*observed.state.lock().unwrap(), NetConnectionState::Closed);
}
