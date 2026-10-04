use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::core::runtime_event_consumer::{
    EditorRuntimeEventConsumerHost, EditorRuntimeEventPumpBudget,
};

use super::abi_fixture::{abi_gateway, AbiEventBacklog, ABI_EVENT_BACKLOG, ABI_EVENT_FIXTURE_LOCK};
use super::support::{percentile_index, register_state, RecordingState, CAPABILITY};

#[test]
#[ignore = "managed performance evidence; run alone with --test-threads=1"]
fn managed_thousand_and_ten_thousand_delivery_budget_report() {
    let _fixture_guard = ABI_EVENT_FIXTURE_LOCK
        .lock()
        .expect("lock ABI event fixture");
    let reports = [1_000_u64, 10_000]
        .into_iter()
        .map(run_abi_delivery_storm)
        .collect::<Vec<_>>();
    println!(
        "PLUGINS01_RUNTIME_EVENT_ABI_PUMP_BENCHMARK={}",
        serde_json::Value::Array(reports)
    );
}

fn run_abi_delivery_storm(delivery_count: u64) -> serde_json::Value {
    const MAX_EVENTS_PER_TICK: usize = 32;

    *ABI_EVENT_BACKLOG.lock().expect("lock ABI event backlog") = AbiEventBacklog {
        remaining: delivery_count,
        next_sequence: 1,
        oldest_pending_age_millis: 17,
    };

    let host = EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(Arc::new(
        abi_gateway(),
    )));
    let state = Arc::new(Mutex::new(RecordingState::default()));
    register_state(
        &host,
        "tests.consumer.storm",
        "tests.events.storm",
        state.clone(),
    );
    host.begin_play_session(500, &[CAPABILITY.to_string()])
        .unwrap();
    let mut tick_durations = Vec::new();
    let mut runtime_drain_durations = Vec::new();
    let mut decode_durations = Vec::new();
    let mut applied = 0_usize;
    let mut max_applied_per_tick = 0_usize;
    let mut max_drained_per_tick = 0_usize;
    let mut max_page_bytes = 0_usize;
    let mut pending_peak = 0_usize;
    let mut max_pending_sequence_span = 0_u64;
    let mut max_editor_pending_encoded_bytes_upper_bound = 0_usize;
    let mut max_editor_pending_oldest_age_millis = 0_u128;
    let mut last_observed_runtime_remaining_peak = 0_usize;
    let mut max_last_observed_runtime_oldest_pending_age_millis = 0_u64;
    while applied < delivery_count as usize {
        let started = Instant::now();
        let report = host
            .pump_with_budget(EditorRuntimeEventPumpBudget::new(
                MAX_EVENTS_PER_TICK,
                MAX_EVENTS_PER_TICK,
                Duration::from_secs(1),
                Duration::from_millis(1),
            ))
            .unwrap();
        tick_durations.push(started.elapsed());
        runtime_drain_durations.push(report.runtime_drain_elapsed());
        decode_durations.push(report.decode_elapsed());
        applied = applied.saturating_add(report.applied());
        max_applied_per_tick = max_applied_per_tick.max(report.applied());
        max_drained_per_tick = max_drained_per_tick.max(report.drained());
        max_page_bytes = max_page_bytes.max(report.drained_encoded_bytes());
        pending_peak = pending_peak.max(report.queue_depth());
        max_pending_sequence_span = max_pending_sequence_span.max(report.pending_sequence_span());
        max_editor_pending_encoded_bytes_upper_bound = max_editor_pending_encoded_bytes_upper_bound
            .max(report.pending_encoded_bytes_upper_bound());
        max_editor_pending_oldest_age_millis =
            max_editor_pending_oldest_age_millis.max(report.pending_oldest_age().as_millis());
        let runtime_backlog = report.runtime_backlog_observation();
        last_observed_runtime_remaining_peak = last_observed_runtime_remaining_peak
            .max(runtime_backlog.known_remaining_deliveries_lower_bound());
        if let Some(oldest_pending_age_millis) = runtime_backlog.max_oldest_pending_age_millis() {
            max_last_observed_runtime_oldest_pending_age_millis =
                max_last_observed_runtime_oldest_pending_age_millis.max(oldest_pending_age_millis);
        }
        assert!(report.applied() <= MAX_EVENTS_PER_TICK);
        assert!(report.drained() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1);
        assert!(
            report.drained_encoded_bytes() <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1
        );
        assert!(
            report.pending_encoded_bytes_upper_bound()
                <= ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1
        );
        assert_eq!(report.runtime_drain_p95(), report.runtime_drain_elapsed());
        assert_eq!(report.decode_p95(), report.decode_elapsed());
        assert_eq!(report.dropped(), 0);
        if report.drained() > 0 {
            assert_eq!(runtime_backlog.sampled_consumer_count(), 1);
            assert_eq!(runtime_backlog.unknown_consumer_count(), 0);
            let remaining = runtime_backlog.known_remaining_deliveries_lower_bound();
            assert_eq!(
                remaining.saturating_add(report.queue_depth()),
                delivery_count as usize - applied
            );
            assert_eq!(
                runtime_backlog.max_oldest_pending_age_millis(),
                Some(if remaining == 0 { 0 } else { 17 })
            );
            assert!(runtime_backlog.max_observation_age().is_some());
        }
    }

    assert_eq!(applied, delivery_count as usize);
    assert_eq!(
        state.lock().unwrap().sequences.len(),
        delivery_count as usize
    );
    assert_eq!(
        ABI_EVENT_BACKLOG
            .lock()
            .expect("lock ABI event backlog")
            .remaining,
        0
    );
    tick_durations.sort_unstable();
    runtime_drain_durations.sort_unstable();
    decode_durations.sort_unstable();
    let p95_index = percentile_index(tick_durations.len());
    let tick_p95_ns = u64::try_from(tick_durations[p95_index].as_nanos()).unwrap_or(u64::MAX);
    let runtime_drain_p95_ns =
        u64::try_from(runtime_drain_durations[p95_index].as_nanos()).unwrap_or(u64::MAX);
    let decode_p95_ns = u64::try_from(decode_durations[p95_index].as_nanos()).unwrap_or(u64::MAX);

    serde_json::json!({
        "deliveries": delivery_count,
        "ticks": tick_durations.len(),
        "max_events_per_tick": MAX_EVENTS_PER_TICK,
        "max_applied_per_tick": max_applied_per_tick,
        "max_drained_per_tick": max_drained_per_tick,
        "max_page_bytes": max_page_bytes,
        "pending_peak": pending_peak,
        "max_pending_sequence_span": max_pending_sequence_span,
        "max_editor_pending_encoded_bytes_upper_bound": max_editor_pending_encoded_bytes_upper_bound,
        "max_editor_pending_oldest_age_millis": max_editor_pending_oldest_age_millis,
        "last_observed_runtime_remaining_peak": last_observed_runtime_remaining_peak,
        "max_last_observed_runtime_oldest_pending_age_millis": max_last_observed_runtime_oldest_pending_age_millis,
        "tick_p95_ns": tick_p95_ns,
        "runtime_drain_p95_ns": runtime_drain_p95_ns,
        "decode_p95_ns": decode_p95_ns,
        "applied": applied,
        "dropped": host.last_pump_report().dropped(),
        "remaining_queue_depth": host.last_pump_report().queue_depth(),
    })
}
