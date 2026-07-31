use std::num::NonZeroUsize;
#[cfg(windows)]
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EventBusDiagnosticsMode, EventBusDiagnosticsSnapshot,
};
use crate::core::{EngineEvent, EventBus};

const WARMUP_SAMPLES: usize = 16;
const PUBLISH_FANOUTS: [usize; 4] = [1, 2, 5, 100];
const PUBLISH_PAYLOAD_BYTES: [usize; 3] = [64, 4_096, 262_144];

#[test]
#[ignore = "managed Runtime07 performance evidence"]
fn event_bus_runtime07_publish_p95_evidence_matrix() {
    for subscriber_count in PUBLISH_FANOUTS {
        for payload_bytes in PUBLISH_PAYLOAD_BYTES {
            let measured_samples = match payload_bytes {
                64 => 256,
                4_096 => 128,
                262_144 => 64,
                _ => unreachable!(),
            };
            let (durations, report) = publish_samples(
                EventBusDiagnosticsMode::Enabled,
                subscriber_count,
                payload_bytes,
                measured_samples,
            );

            let expected_published = (WARMUP_SAMPLES + measured_samples) as u64;
            assert_eq!(report.published, expected_published);
            assert_eq!(
                report.delivered,
                expected_published * subscriber_count as u64
            );
            assert_eq!(report.dropped, 0);
            assert_eq!(report.queued, 0);
            assert_eq!(report.peak_queued, subscriber_count as u64);
            assert_eq!(report.waiting_publishers, 0);
            assert_eq!(report.delivery_lock_wait_samples, 0);
            assert_eq!(report.total_delivery_lock_wait_ms, 0.0);
            assert_eq!(report.max_delivery_lock_wait_ms, 0.0);
            println!(
                "EVENTBUS_BENCH_V1 kind=publish mode=enabled subscribers={} payload_bytes={} samples={} p50_ns={} p95_ns={} max_ns={} delivery_lock_wait_samples={} total_delivery_lock_wait_ms={:.3} max_delivery_lock_wait_ms={:.3}",
                subscriber_count,
                payload_bytes,
                measured_samples,
                percentile_ns(&durations, 50),
                percentile_ns(&durations, 95),
                durations.iter().copied().max().unwrap_or_default(),
                report.delivery_lock_wait_samples,
                report.total_delivery_lock_wait_ms,
                report.max_delivery_lock_wait_ms,
            );
        }
    }
}

#[test]
#[ignore = "managed Runtime07 performance evidence"]
fn event_bus_runtime07_disabled_diagnostics_overhead_evidence() {
    for (case_index, subscriber_count) in [1, 100].into_iter().enumerate() {
        let measured_samples = if subscriber_count == 1 { 256 } else { 128 };
        let modes = if case_index % 2 == 0 {
            [
                EventBusDiagnosticsMode::Enabled,
                EventBusDiagnosticsMode::Disabled,
            ]
        } else {
            [
                EventBusDiagnosticsMode::Disabled,
                EventBusDiagnosticsMode::Enabled,
            ]
        };
        let mut enabled_p95 = 0;
        let mut disabled_p95 = 0;
        for mode in modes {
            let (durations, report) =
                publish_samples(mode, subscriber_count, 4_096, measured_samples);
            let p95 = percentile_ns(&durations, 95);
            match mode {
                EventBusDiagnosticsMode::Enabled => {
                    enabled_p95 = p95;
                    assert!(report.enabled);
                    assert_eq!(report.published, (WARMUP_SAMPLES + measured_samples) as u64);
                }
                EventBusDiagnosticsMode::Disabled => {
                    disabled_p95 = p95;
                    assert_disabled_report(report, subscriber_count as u64);
                }
            }
        }
        let ratio = if disabled_p95 == 0 {
            f64::INFINITY
        } else {
            enabled_p95 as f64 / disabled_p95 as f64
        };
        println!(
            "EVENTBUS_BENCH_V1 kind=diagnostics subscribers={} payload_bytes=4096 samples={} enabled_p95_ns={} disabled_p95_ns={} p95_ratio_enabled_over_disabled={:.4}",
            subscriber_count, measured_samples, enabled_p95, disabled_p95, ratio,
        );
    }
}

#[test]
#[ignore = "managed Runtime07 performance evidence"]
fn event_bus_runtime07_paused_bounded_consumer_pressure_evidence() {
    const CAPACITY: usize = 64;
    const PAYLOAD_BYTES: usize = 65_536;
    const PHASE_ONE_PUBLISHES: usize = 512;
    const PHASE_TWO_PUBLISHES: usize = 8_192;

    let bus = EventBus::default();
    let events = bus.subscribe(
        "runtime.pressure",
        EngineEventDeliveryPolicy::BoundedDropOldest {
            capacity: NonZeroUsize::new(CAPACITY).unwrap(),
        },
    );
    let rss_before = current_process_rss_bytes();
    publish_pressure_events(&bus, 0, PHASE_ONE_PUBLISHES, PAYLOAD_BYTES);
    let phase_one = bus.diagnostic_report();
    assert_eq!(phase_one.queued, CAPACITY as u64);
    assert_eq!(phase_one.peak_queued, CAPACITY as u64);
    let rss_phase_one = current_process_rss_bytes();

    std::thread::sleep(Duration::from_millis(20));
    publish_pressure_events(
        &bus,
        PHASE_ONE_PUBLISHES,
        PHASE_TWO_PUBLISHES,
        PAYLOAD_BYTES,
    );
    let before_drain = bus.diagnostic_report();
    let total_publishes = PHASE_ONE_PUBLISHES + PHASE_TWO_PUBLISHES;
    assert_eq!(before_drain.published, total_publishes as u64);
    assert_eq!(before_drain.delivered, total_publishes as u64);
    assert_eq!(before_drain.queued, CAPACITY as u64);
    assert_eq!(before_drain.peak_queued, CAPACITY as u64);
    assert_eq!(before_drain.dropped, (total_publishes - CAPACITY) as u64);
    let rss_phase_two = current_process_rss_bytes();

    for expected in total_publishes - CAPACITY..total_publishes {
        assert_eq!(
            events.recv().unwrap().payload["sequence"]
                .as_u64()
                .expect("pressure sequence must stay an integer") as usize,
            expected
        );
    }
    let after_drain = bus.diagnostic_report();
    assert_eq!(after_drain.queued, 0);
    assert_eq!(after_drain.queue_age_samples, total_publishes as u64);
    assert!(after_drain.max_queue_age_ms >= 10.0);
    let rss_after_drain = current_process_rss_bytes();
    println!(
        "EVENTBUS_BENCH_V1 kind=pressure capacity={} payload_bytes={} publishes={} retained_bytes={} rss_before={} rss_phase1={} rss_phase2={} rss_after_drain={} total_queue_age_ms={:.3} max_queue_age_ms={:.3}",
        CAPACITY,
        PAYLOAD_BYTES,
        total_publishes,
        CAPACITY * PAYLOAD_BYTES,
        rss_value(rss_before),
        rss_value(rss_phase_one),
        rss_value(rss_phase_two),
        rss_value(rss_after_drain),
        after_drain.total_queue_age_ms,
        after_drain.max_queue_age_ms,
    );
}

fn publish_samples(
    mode: EventBusDiagnosticsMode,
    subscriber_count: usize,
    payload_bytes: usize,
    measured_samples: usize,
) -> (Vec<u64>, EventBusDiagnosticsSnapshot) {
    let bus = EventBus::new(mode);
    let subscriptions = (0..subscriber_count)
        .map(|_| bus.subscribe("runtime.benchmark", EngineEventDeliveryPolicy::Lossless))
        .collect::<Vec<_>>();
    let mut durations = Vec::with_capacity(measured_samples);
    for sample in 0..WARMUP_SAMPLES + measured_samples {
        let event = EngineEvent {
            topic: "runtime.benchmark".to_string(),
            payload: serde_json::json!({ "blob": "x".repeat(payload_bytes) }),
        };
        let started = Instant::now();
        bus.publish(event);
        let elapsed = started.elapsed();
        let first = subscriptions[0].recv().unwrap();
        assert_eq!(
            first.payload["blob"].as_str().map(str::len),
            Some(payload_bytes),
            "delivered payload size must match the benchmark case",
        );
        for subscription in subscriptions.iter().skip(1) {
            assert!(Arc::ptr_eq(&first, &subscription.recv().unwrap()));
        }
        if sample >= WARMUP_SAMPLES {
            durations.push(elapsed.as_nanos().min(u128::from(u64::MAX)) as u64);
        }
    }
    (durations, bus.diagnostic_report())
}

fn publish_pressure_events(bus: &EventBus, start: usize, count: usize, payload_bytes: usize) {
    for sequence in start..start + count {
        bus.publish(EngineEvent {
            topic: "runtime.pressure".to_string(),
            payload: serde_json::json!({
                "sequence": sequence,
                "blob": "x".repeat(payload_bytes),
            }),
        });
    }
}

fn percentile_ns(samples: &[u64], percentile: usize) -> u64 {
    let mut samples = samples.to_vec();
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100).saturating_sub(1);
    samples[rank]
}

fn assert_disabled_report(report: EventBusDiagnosticsSnapshot, subscribers: u64) {
    assert_eq!(
        report,
        EventBusDiagnosticsSnapshot {
            enabled: false,
            topics: 1,
            subscribers,
            ..EventBusDiagnosticsSnapshot::default()
        },
        "disabled diagnostics must retain topology and zero every hot-path metric",
    );
}

#[cfg(windows)]
fn current_process_rss_bytes() -> Option<u64> {
    let command = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
    let output = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
        .arg(command)
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    String::from_utf8(output.stdout).ok()?.trim().parse().ok()
}

#[cfg(not(windows))]
fn current_process_rss_bytes() -> Option<u64> {
    None
}

fn rss_value(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".to_string(), |value| value.to_string())
}
