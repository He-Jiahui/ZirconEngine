use super::*;
use crate::memory_validation::{live_bytes, measure};
#[test]
fn memory_profile_slow_subscribers() {
    const SUBSCRIBERS: usize = 8;
    const FIRST: usize = 20_000;
    const TOTAL: usize = 40_000;
    let event = AssetChange {
        kind: super::super::AssetChangeKind::Removed,
        uri: crate::asset::AssetUri::parse("res://memory/event.model").unwrap(),
        previous_uri: None,
    };
    let baseline = live_bytes();
    let (legacy, old) = measure(|| {
        let channels: Vec<_> = (0..SUBSCRIBERS)
            .map(|_| crossbeam_channel::unbounded())
            .collect();
        for _ in 0..FIRST {
            for (sender, _) in &channels {
                sender.send(event.clone()).unwrap();
            }
        }
        channels
    });
    old.report(
        "8_paused_subscribers_20000_publications",
        "per_subscriber_unbounded_baseline",
    );
    drop(legacy);
    assert_eq!(live_bytes(), baseline);
    let ((log, subscribers), new) = measure(|| {
        let log = AssetEventLog::new(CHANGE_LOG_LIMITS.0, CHANGE_LOG_LIMITS.1);
        let subscribers: Vec<_> = (0..SUBSCRIBERS).map(|_| log.subscribe(None)).collect();
        for _ in 0..FIRST {
            log.publish([event.clone()], change_bytes);
        }
        (log, subscribers)
    });
    new.report(
        "8_paused_subscribers_20000_publications",
        "bounded_shared_log",
    );
    assert!(new.live_bytes < old.live_bytes / 4);
    let ((), steady) = measure(|| {
        for _ in FIRST..TOTAL {
            log.publish([event.clone()], change_bytes);
        }
    });
    steady.report(
        "next_20000_publications_consumers_still_paused",
        "bounded_shared_log_steady",
    );
    assert_eq!(steady.live_bytes, 0);
    assert!(steady.peak_bytes < 1024);
    assert_eq!(subscribers[0].diagnostics().depth, CHANGE_LOG_LIMITS.0);
    for subscriber in &subscribers {
        let Err(AssetEventTryRecvError::Lagged(gap)) = subscriber.try_recv() else {
            panic!("expected loss");
        };
        assert_eq!(gap.missed_count, (TOTAL - CHANGE_LOG_LIMITS.0) as u64);
    }
    drop(log);
    drop(subscribers);
    assert_eq!(live_bytes(), baseline);
}
