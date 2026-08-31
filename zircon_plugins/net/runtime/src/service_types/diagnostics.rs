use std::collections::VecDeque;

use zircon_runtime::core::framework::net::{NetDiagnostics, NetEvent};

use crate::poison_recovery::lock_recover;

use super::DefaultNetManager;

impl DefaultNetManager {
    pub(in crate::service_types) fn backend_name_impl(&self) -> String {
        let mut name = "tokio-net".to_string();
        if lock_recover(&self.state.http_backend).is_some() {
            name.push_str("+http");
        }
        if lock_recover(&self.state.websocket_backend).is_some() {
            name.push_str("+websocket");
        }
        name
    }

    pub(in crate::service_types) fn drain_events_impl(&self, max_events: usize) -> Vec<NetEvent> {
        self.state.poll_worker_ingress(max_events);
        let mut events = lock_recover(&self.state.events);
        drain_bounded_events(&mut events, max_events)
    }

    pub(in crate::service_types) fn diagnostics_impl(&self) -> NetDiagnostics {
        self.state.poll_worker_ingress(usize::MAX);
        let (outbound_bytes, inbound_bytes, last_observed_latency_ms) =
            self.state.diagnostic_counters();
        NetDiagnostics {
            backend_name: self.backend_name_impl(),
            mode: self.state.mode,
            outbound_bytes,
            inbound_bytes,
            last_observed_latency_ms,
            open_udp_sockets: lock_recover(&self.state.udp_sockets).len(),
            open_tcp_listeners: lock_recover(&self.state.tcp_listeners).len(),
            open_http_listeners: lock_recover(&self.state.http_listeners).len(),
            open_websocket_listeners: lock_recover(&self.state.websocket_listeners).len(),
            open_tcp_connections: lock_recover(&self.state.tcp_connections).len(),
            open_http_routes: lock_recover(&self.state.http_routes).len(),
            open_websocket_connections: lock_recover(&self.state.websocket_connections).len(),
            queued_events: lock_recover(&self.state.events).len(),
        }
    }
}

fn drain_bounded_events(events: &mut VecDeque<NetEvent>, max_events: usize) -> Vec<NetEvent> {
    let drain_count = max_events.min(events.len());
    let mut drained = Vec::with_capacity(drain_count);
    drained.extend(events.drain(..drain_count));
    drained
}

#[cfg(test)]
mod bounded_event_drain_tests {
    use std::{collections::VecDeque, hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{NetEvent, NetRouteId};

    use super::drain_bounded_events;

    const BENCHMARK_EVENT_COUNT: usize = 8_192;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn bounded_event_drain_preserves_fifo_limit_and_tail() {
        let mut events = (1..=5).map(route_event).collect::<VecDeque<_>>();

        let drained = drain_bounded_events(&mut events, 3);

        assert_eq!(drained, (1..=3).map(route_event).collect::<Vec<NetEvent>>());
        assert_eq!(
            events.into_iter().collect::<Vec<_>>(),
            (4..=5).map(route_event).collect::<Vec<NetEvent>>()
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn bounded_event_drain_release_benchmark_evidence() {
        let mut legacy_queue = benchmark_queue();
        let mut optimized_queue = benchmark_queue();
        let legacy_growth_events = legacy_growth_events(benchmark_queue());
        let (legacy_samples, optimized_samples) =
            benchmark_paired_samples(&mut legacy_queue, &mut optimized_queue);
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_raw_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_raw_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT task=plugins10_bounded_event_drain events={BENCHMARK_EVENT_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_vec_growth_events_per_sample={legacy_growth_events} optimized_vec_reservations_per_sample=1 threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_raw_ns} optimized_raw_ns={optimized_raw_ns}"
        );
        assert!(legacy_growth_events > 1);
        assert!(
            optimized_p95 * 100 <= legacy_p95 * 85,
            "optimized P95 {optimized_p95}ns must be at least 15% lower than legacy P95 {legacy_p95}ns"
        );
    }

    fn route_event(raw: u64) -> NetEvent {
        NetEvent::HttpRouteUnregistered {
            route: NetRouteId::new(raw),
        }
    }

    fn benchmark_queue() -> VecDeque<NetEvent> {
        (1..=BENCHMARK_EVENT_COUNT as u64)
            .map(route_event)
            .collect()
    }

    fn legacy_drain_events(events: &mut VecDeque<NetEvent>, max_events: usize) -> Vec<NetEvent> {
        let mut drained = Vec::new();
        while drained.len() < max_events {
            match events.pop_front() {
                Some(event) => drained.push(event),
                None => break,
            }
        }
        drained
    }

    fn legacy_growth_events(mut events: VecDeque<NetEvent>) -> usize {
        let mut drained = Vec::new();
        let mut growth_events = 0;
        while let Some(event) = events.pop_front() {
            if drained.len() == drained.capacity() {
                growth_events += 1;
            }
            drained.push(event);
        }
        growth_events
    }

    fn benchmark_paired_samples(
        legacy_queue: &mut VecDeque<NetEvent>,
        optimized_queue: &mut VecDeque<NetEvent>,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(benchmark_sample(legacy_queue, legacy_drain_events));
        black_box(benchmark_sample(optimized_queue, drain_bounded_events));
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(legacy_queue, legacy_drain_events));
                optimized_samples.push(benchmark_sample(optimized_queue, drain_bounded_events));
            } else {
                optimized_samples.push(benchmark_sample(optimized_queue, drain_bounded_events));
                legacy_samples.push(benchmark_sample(legacy_queue, legacy_drain_events));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(
        events: &mut VecDeque<NetEvent>,
        drain: fn(&mut VecDeque<NetEvent>, usize) -> Vec<NetEvent>,
    ) -> u128 {
        assert_eq!(events.len(), BENCHMARK_EVENT_COUNT);
        let started = Instant::now();
        let drained = black_box(drain(events, BENCHMARK_EVENT_COUNT));
        let elapsed = started.elapsed().as_nanos();
        assert!(events.is_empty());
        assert_eq!(drained.len(), BENCHMARK_EVENT_COUNT);
        events.extend(drained);
        elapsed
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }
}
