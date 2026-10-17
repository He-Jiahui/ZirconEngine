use super::*;
use crate::asset::{watch::AssetChangeKind, AssetUri};

#[test]
fn slow_consumer_reports_loss_while_fast_consumer_preserves_order_and_shares_payload() {
    let log = AssetEventLog::new(3, 1024);
    let fast = log.subscribe(None);
    let slow = log.subscribe(None);
    let mirror = log.subscribe(None);
    log.publish([0_u32], |_| 4);
    let a = fast.try_recv().unwrap();
    let b = mirror.try_recv().unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    for value in 1..1000 {
        log.publish([value], |_| 4);
        assert_eq!(*fast.try_recv().unwrap(), value);
    }
    let Err(AssetEventTryRecvError::Lagged(gap)) = slow.try_recv() else {
        panic!("missing explicit lag");
    };
    assert_eq!(gap.missed_count, 997);
    assert_eq!(*slow.try_recv().unwrap(), 997);
    let diagnostics = slow.diagnostics();
    assert_eq!(diagnostics.depth, 3);
    assert!(diagnostics.approximate_bytes <= 1024);
}

#[test]
fn oversized_events_expiry_and_sequence_exhaustion_are_explicit() {
    let log = AssetEventLog::new(3, 128);
    let subscriber = log.subscribe(None);
    log.publish([1_u32], |_| 1000);
    assert!(matches!(
        subscriber.try_recv(),
        Err(AssetEventTryRecvError::Lagged(_))
    ));
    log.publish([2_u32], |_| 4);
    {
        let mut state = log.shared.state.lock().unwrap();
        state.entries.front_mut().unwrap().at = Instant::now() - MAX_AGE;
    }
    assert!(matches!(
        subscriber.try_recv(),
        Err(AssetEventTryRecvError::Lagged(_))
    ));
    {
        let mut state = log.shared.state.lock().unwrap();
        state.tail = u64::MAX;
        subscriber.cursor.next.store(u64::MAX, Ordering::Relaxed);
    }
    log.publish([3], |_| 4);
    assert_eq!(
        subscriber.try_recv(),
        Err(AssetEventTryRecvError::SequenceExhausted)
    );
}

#[test]
fn callbacks_are_coalesced_reentrant_and_last_publisher_disconnects() {
    let log = AssetEventLog::new(3, 1024);
    let subscriber = log.subscribe(None);
    let observer = subscriber.clone();
    let callback = Arc::new(move || {
        let _ = observer.diagnostics();
    });
    let callback_subscriber = log.subscribe(Some(callback));
    let wakes = log.publish([1_u32, 2], |_| 4);
    assert_eq!(wakes.len(), 1);
    for wake in wakes {
        wake();
    }
    assert!(log.publish([3], |_| 4).is_empty());
    assert_eq!(*callback_subscriber.try_recv().unwrap(), 1);
    let other_owner = log.clone();
    drop(log);
    assert_eq!(*subscriber.try_recv().unwrap(), 1);
    drop(other_owner);
    assert_eq!(
        subscriber.try_recv(),
        Err(AssetEventTryRecvError::Disconnected)
    );
}

#[test]
fn native_waits_do_not_miss_publication_or_disconnect() {
    let log = AssetEventLog::new(3, 1024);
    let subscriber = log.subscribe(None);
    let publisher = log.clone();
    let worker = std::thread::spawn(move || {
        publisher.publish([7_u32], |_| 4);
    });
    assert_eq!(*subscriber.recv_timeout(Duration::from_secs(1)).unwrap(), 7);
    worker.join().unwrap();
    drop(log);
    assert_eq!(
        subscriber.recv_timeout(Duration::from_millis(1)),
        Err(AssetEventRecvTimeoutError::Disconnected)
    );
}

#[test]
fn structural_events_remain_ordered_and_error_log_stays_bounded() {
    use super::super::AssetChangeKind;
    use crate::asset::AssetUri;
    let log = AssetEventLog::new(CHANGE_LOG_LIMITS.0, CHANGE_LOG_LIMITS.1);
    let reader = log.subscribe(None);
    let old = AssetUri::parse("res://old.mesh").unwrap();
    let new = AssetUri::parse("res://new.mesh").unwrap();
    let changes = [
        AssetChange {
            kind: AssetChangeKind::Renamed,
            uri: new.clone(),
            previous_uri: Some(old),
        },
        AssetChange {
            kind: AssetChangeKind::Removed,
            uri: new,
            previous_uri: None,
        },
    ];
    let _ = log.publish(changes.clone(), change_bytes);
    for expected in &changes {
        assert_eq!(&*reader.try_recv().unwrap(), expected);
    }

    let errors = AssetEventLog::new(ERROR_LOG_LIMITS.0, ERROR_LOG_LIMITS.1);
    let paused = errors.subscribe(None);
    for index in 0..10_000 {
        let _ = errors.publish(
            [AssetWatchError {
                assets_root: "assets".into(),
                paths: vec!["assets/failed.mesh".into()],
                message: format!("failure {index} {}", "x".repeat(1024)),
            }],
            error_bytes,
        );
        let state = paused.diagnostics();
        assert!(state.depth <= ERROR_LOG_LIMITS.0);
        assert!(state.approximate_bytes <= ERROR_LOG_LIMITS.1);
    }
    let Err(AssetEventTryRecvError::Lagged(gap)) = paused.try_recv() else {
        panic!("loss must be observable")
    };
    assert_eq!(gap.missed_count, paused.diagnostics().dropped_count);
}

#[test]
fn evicted_and_closed_payloads_release_their_heap_storage() {
    let log = AssetEventLog::new(1, 4096);
    let reader = log.subscribe(None);
    let _ = log.publish([vec![1_u8; 1024]], Vec::capacity);
    let held = reader.try_recv().unwrap();
    let weak = Arc::downgrade(&held);
    let _ = log.publish([vec![2_u8; 1024]], Vec::capacity);
    assert!(weak.upgrade().is_some());
    drop(held);
    assert!(weak.upgrade().is_none());
    let last = reader.try_recv().unwrap();
    let weak = Arc::downgrade(&last);
    drop(last);
    drop(log);
    assert!(weak.upgrade().is_none());
}

#[test]
fn resync_rearms_a_coalesced_wake_without_reading_an_old_event() {
    let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = count.clone();
    let log = AssetEventLog::new(4, 1024);
    let reader = log.subscribe(Some(Arc::new(move || {
        observed.fetch_add(1, Ordering::Relaxed);
    })));
    for wake in log.publish([1_u32, 2], |_| 4) {
        wake();
    }
    assert_eq!(count.load(Ordering::Relaxed), 1);
    reader.reset_to_tail().unwrap();
    for wake in log.publish([3_u32], |_| 4) {
        wake();
    }
    assert_eq!(count.load(Ordering::Relaxed), 2);
    assert_eq!(*reader.try_recv().unwrap(), 3);
}

#[test]
fn byte_caps_include_reserved_labels_and_path_buffers() {
    let base = AssetUri::parse("res://short.mesh").unwrap();
    let mut label = String::with_capacity(CHANGE_LOG_LIMITS.1 + 1);
    label.push('x');
    let uri = AssetUri::new(base.scheme(), base.path(), Some(label)).unwrap();
    let log = AssetEventLog::new(CHANGE_LOG_LIMITS.0, CHANGE_LOG_LIMITS.1);
    let reader = log.subscribe(None);
    let _ = log.publish(
        [AssetChange {
            kind: AssetChangeKind::Added,
            uri,
            previous_uri: None,
        }],
        change_bytes,
    );
    assert!(matches!(
        reader.try_recv(),
        Err(AssetEventTryRecvError::Lagged(AssetEventGap {
            missed_count: 1,
            ..
        }))
    ));
    assert_eq!(reader.diagnostics().depth, 0);

    let mut root = std::path::PathBuf::with_capacity(ERROR_LOG_LIMITS.1 + 1);
    root.push("assets");
    let log = AssetEventLog::new(ERROR_LOG_LIMITS.0, ERROR_LOG_LIMITS.1);
    let reader = log.subscribe(None);
    let _ = log.publish(
        [AssetWatchError {
            assets_root: root,
            paths: Vec::new(),
            message: String::new(),
        }],
        error_bytes,
    );
    assert!(matches!(
        reader.try_recv(),
        Err(AssetEventTryRecvError::Lagged(AssetEventGap {
            missed_count: 1,
            ..
        }))
    ));
    assert_eq!(reader.diagnostics().depth, 0);
}
