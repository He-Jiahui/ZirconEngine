use std::num::{NonZeroU64, NonZeroUsize};
#[cfg(windows)]
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EventBusDiagnosticsMode, EventBusDiagnosticsSnapshot,
    DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL,
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
            let arc_clones_before = subscriber_count;
            let arc_clones_after = subscriber_count.saturating_sub(1);
            let arc_clone_reduction_percent =
                (1.0 - arc_clones_after as f64 / arc_clones_before as f64) * 100.0;
            println!(
                "EVENTBUS_BENCH_V2 kind=publish mode=enabled subscribers={} payload_bytes={} samples={} payload_arc_clones_before={} payload_arc_clones_after={} payload_arc_clone_reduction_percent={:.4} p50_ns={} p95_ns={} p99_ns={} max_ns={} delivery_lock_wait_samples={} total_delivery_lock_wait_ms={:.3} max_delivery_lock_wait_ms={:.3}",
                subscriber_count,
                payload_bytes,
                measured_samples,
                arc_clones_before,
                arc_clones_after,
                arc_clone_reduction_percent,
                percentile_ns(&durations, 50),
                percentile_ns(&durations, 95),
                percentile_ns(&durations, 99),
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
fn event_bus_runtime07_diagnostics_sampling_evidence() {
    const REPEATS: usize = 5;
    const SAMPLE_INTERVAL: u64 = 64;

    for subscriber_count in [1, 100] {
        let measured_samples = if subscriber_count == 1 { 256 } else { 128 };
        let sampled_mode = EventBusDiagnosticsMode::Sampled {
            every: NonZeroU64::new(SAMPLE_INTERVAL).unwrap(),
        };
        let modes = [
            EventBusDiagnosticsMode::Enabled,
            sampled_mode,
            EventBusDiagnosticsMode::Disabled,
        ];
        let mut enabled_samples = Vec::with_capacity(measured_samples * REPEATS);
        let mut sampled_samples = Vec::with_capacity(measured_samples * REPEATS);
        let mut disabled_samples = Vec::with_capacity(measured_samples * REPEATS);

        for repeat in 0..REPEATS {
            for mode_offset in 0..modes.len() {
                let mode = modes[(mode_offset + repeat) % modes.len()];
                let (mut durations, report) =
                    publish_samples(mode, subscriber_count, 4_096, measured_samples);
                match mode {
                    EventBusDiagnosticsMode::Enabled => {
                        assert_full_report(report, subscriber_count as u64, measured_samples);
                        enabled_samples.append(&mut durations);
                    }
                    EventBusDiagnosticsMode::Sampled { every } => {
                        assert_sampled_report(
                            report,
                            subscriber_count as u64,
                            measured_samples,
                            every,
                        );
                        sampled_samples.append(&mut durations);
                    }
                    EventBusDiagnosticsMode::Disabled => {
                        assert_disabled_report(report, subscriber_count as u64);
                        disabled_samples.append(&mut durations);
                    }
                }
            }
        }

        let delivered = (WARMUP_SAMPLES + measured_samples) as u64 * subscriber_count as u64;
        let full_captures =
            ((WARMUP_SAMPLES + measured_samples) as u64 + delivered) * REPEATS as u64;
        let sampled_captures =
            (sample_count((WARMUP_SAMPLES + measured_samples) as u64, SAMPLE_INTERVAL)
                + sample_count(delivered, SAMPLE_INTERVAL))
                * REPEATS as u64;
        let capture_reduction_percent =
            (1.0 - sampled_captures as f64 / full_captures as f64) * 100.0;
        println!(
            "EVENTBUS_BENCH_V2 kind=diagnostics_sampling subscribers={} payload_bytes=4096 repeats={} samples_per_mode={} sample_interval={} full_timing_captures={} sampled_timing_captures={} capture_reduction_percent={:.4} full_p50_ns={} full_p95_ns={} full_p99_ns={} full_throughput_per_second={:.2} sampled_p50_ns={} sampled_p95_ns={} sampled_p99_ns={} sampled_throughput_per_second={:.2} disabled_p50_ns={} disabled_p95_ns={} disabled_p99_ns={} disabled_throughput_per_second={:.2}",
            subscriber_count,
            REPEATS,
            measured_samples * REPEATS,
            SAMPLE_INTERVAL,
            full_captures,
            sampled_captures,
            capture_reduction_percent,
            percentile_ns(&enabled_samples, 50),
            percentile_ns(&enabled_samples, 95),
            percentile_ns(&enabled_samples, 99),
            throughput_per_second(&enabled_samples),
            percentile_ns(&sampled_samples, 50),
            percentile_ns(&sampled_samples, 95),
            percentile_ns(&sampled_samples, 99),
            throughput_per_second(&sampled_samples),
            percentile_ns(&disabled_samples, 50),
            percentile_ns(&disabled_samples, 95),
            percentile_ns(&disabled_samples, 99),
            throughput_per_second(&disabled_samples),
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
    assert_eq!(
        after_drain.routine_timing_sample_interval,
        DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL.get()
    );
    assert_eq!(
        after_drain.queue_age_samples,
        sample_count(
            total_publishes as u64,
            DEFAULT_EVENT_BUS_TIMING_SAMPLE_INTERVAL.get()
        )
    );
    assert!(after_drain.max_queue_age_ms >= 10.0);
    let rss_after_drain = current_process_rss_bytes();
    let replacements = total_publishes - CAPACITY;
    println!(
        "EVENTBUS_BENCH_V2 kind=pressure capacity={} payload_bytes={} publishes={} retained_bytes={} replacements={} replacement_depth_rmw_before={} replacement_depth_rmw_after=0 replacement_depth_rmw_reduction_percent=100.0000 rss_before={} rss_phase1={} rss_phase2={} rss_after_drain={} total_queue_age_ms={:.3} max_queue_age_ms={:.3}",
        CAPACITY,
        PAYLOAD_BYTES,
        total_publishes,
        CAPACITY * PAYLOAD_BYTES,
        replacements,
        replacements * 2,
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

fn assert_full_report(
    report: EventBusDiagnosticsSnapshot,
    subscribers: u64,
    measured_samples: usize,
) {
    let publishes = (WARMUP_SAMPLES + measured_samples) as u64;
    assert!(report.enabled);
    assert_eq!(report.routine_timing_sample_interval, 1);
    assert_eq!(report.published, publishes);
    assert_eq!(report.delivered, publishes * subscribers);
    assert_eq!(report.publish_samples, publishes);
    assert_eq!(report.queue_age_samples, publishes * subscribers);
}

fn assert_sampled_report(
    report: EventBusDiagnosticsSnapshot,
    subscribers: u64,
    measured_samples: usize,
    every: NonZeroU64,
) {
    let publishes = (WARMUP_SAMPLES + measured_samples) as u64;
    assert!(report.enabled);
    assert_eq!(report.routine_timing_sample_interval, every.get());
    assert_eq!(report.published, publishes);
    assert_eq!(report.delivered, publishes * subscribers);
    assert_eq!(report.publish_samples, sample_count(publishes, every.get()));
    assert_eq!(
        report.queue_age_samples,
        sample_count(publishes * subscribers, every.get())
    );
}

fn sample_count(total: u64, every: u64) -> u64 {
    total.div_ceil(every)
}

fn throughput_per_second(samples_ns: &[u64]) -> f64 {
    let total_ns = samples_ns
        .iter()
        .map(|sample| u128::from(*sample))
        .sum::<u128>();
    if total_ns == 0 {
        return f64::INFINITY;
    }
    samples_ns.len() as f64 * 1_000_000_000.0 / total_ns as f64
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
