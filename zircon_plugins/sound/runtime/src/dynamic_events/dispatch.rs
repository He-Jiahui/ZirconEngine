use zircon_runtime::core::framework::sound::{
    SoundDynamicEventDelivery, SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation,
};

use super::handlers::DynamicEventHandlerRegistry;

pub(crate) fn dispatch_dynamic_events(
    handlers: &DynamicEventHandlerRegistry,
    pending: &mut Vec<SoundDynamicEventInvocation>,
) -> Vec<SoundDynamicEventDelivery> {
    let delivery_capacity = pending.iter().fold(0_usize, |count, invocation| {
        count.saturating_add(
            handlers
                .indices_for_event(invocation.event_id.as_str())
                .map_or(0, <[usize]>::len),
        )
    });
    let mut deliveries = Vec::with_capacity(delivery_capacity);
    for invocation in pending.drain(..) {
        let Some((last_handler_index, leading_handler_indices)) = handlers
            .indices_for_event(invocation.event_id.as_str())
            .and_then(<[usize]>::split_last)
        else {
            continue;
        };
        deliveries.extend(leading_handler_indices.iter().map(|handler_index| {
            SoundDynamicEventDelivery {
                handler: handlers.handler(*handler_index).clone(),
                invocation: invocation.clone(),
            }
        }));
        deliveries.push(SoundDynamicEventDelivery {
            handler: handlers.handler(*last_handler_index).clone(),
            invocation,
        });
    }
    deliveries
}

#[cfg(test)]
mod tests {
    use std::{hint::black_box, time::Instant};

    use zircon_runtime::core::framework::sound::{
        SoundDynamicEventDelivery, SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation,
    };

    use crate::dynamic_events::handlers::{
        DynamicEventHandlerRegistry, dynamic_event_handler_dispatch_order,
    };

    use super::dispatch_dynamic_events;

    const BENCHMARK_EVENT_COUNT: usize = 2_048;
    const BENCHMARK_HANDLER_COUNT: usize = 64;
    const BENCHMARK_EVENT_ID_COUNT: usize = 16;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const BENCHMARK_ITERATIONS_PER_SAMPLE: usize = 4;

    #[test]
    fn preordered_dispatch_index_preserves_per_event_order() {
        let handlers = vec![
            handler("music.stop", "music", "fade", 10),
            handler("weapon.fire", "telemetry", "count", 20),
            handler("weapon.fire", "audio", "foley", 20),
            handler("music.stop", "music", "flush", 30),
        ];
        let handlers = DynamicEventHandlerRegistry::from_handlers(handlers);
        let mut pending = vec![invocation("weapon.fire"), invocation("music.stop")];

        let deliveries = dispatch_dynamic_events(&handlers, &mut pending);

        assert!(pending.is_empty());
        assert_eq!(
            deliveries
                .iter()
                .map(|delivery| format!(
                    "{}/{}/{}",
                    delivery.invocation.event_id,
                    delivery.handler.plugin_id,
                    delivery.handler.handler_id
                ))
                .collect::<Vec<_>>(),
            [
                "weapon.fire/audio/foley",
                "weapon.fire/telemetry/count",
                "music.stop/music/flush",
                "music.stop/music/fade",
            ]
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn preordered_handler_dispatch_release_benchmark_evidence() {
        let mut legacy_handlers = (0..BENCHMARK_HANDLER_COUNT)
            .map(benchmark_handler)
            .collect::<Vec<_>>();
        legacy_handlers.sort_by(dynamic_event_handler_dispatch_order);
        let handlers = DynamicEventHandlerRegistry::from_handlers(legacy_handlers.clone());
        let pending = (0..BENCHMARK_EVENT_COUNT)
            .map(benchmark_invocation)
            .collect::<Vec<_>>();

        let mut legacy_pending = pending.clone();
        let mut optimized_pending = pending.clone();
        assert_eq!(
            legacy_dispatch_dynamic_events(&legacy_handlers, &mut legacy_pending),
            dispatch_dynamic_events(&handlers, &mut optimized_pending)
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            &pending,
            |events| legacy_dispatch_dynamic_events(&legacy_handlers, events),
            |events| dispatch_dynamic_events(&handlers, events),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);

        println!(
            "PERF_RESULT plugins11_preordered_dynamic_event_handlers events={} registered_handlers={} event_ids={} matching_handlers_per_event={} deliveries={} samples={} iterations_per_sample={} sample_order=alternating legacy_handler_sorts={} optimized_handler_sorts=0 legacy_per_event_matching_vec_allocations={} optimized_per_event_matching_vec_allocations=0 optimized_handler_index_buckets={} optimized_handler_index_rebuilds_per_dispatch=0 legacy_handler_match_comparisons={} optimized_handler_index_entries={} optimized_event_lookups={} legacy_invocation_clones={} optimized_invocation_clones={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={}",
            BENCHMARK_EVENT_COUNT,
            BENCHMARK_HANDLER_COUNT,
            BENCHMARK_EVENT_ID_COUNT,
            BENCHMARK_HANDLER_COUNT / BENCHMARK_EVENT_ID_COUNT,
            BENCHMARK_EVENT_COUNT * BENCHMARK_HANDLER_COUNT / BENCHMARK_EVENT_ID_COUNT,
            BENCHMARK_SAMPLE_COUNT,
            BENCHMARK_ITERATIONS_PER_SAMPLE,
            BENCHMARK_EVENT_COUNT,
            BENCHMARK_EVENT_COUNT,
            BENCHMARK_EVENT_ID_COUNT,
            BENCHMARK_EVENT_COUNT * BENCHMARK_HANDLER_COUNT,
            BENCHMARK_HANDLER_COUNT,
            BENCHMARK_EVENT_COUNT,
            BENCHMARK_EVENT_COUNT * BENCHMARK_HANDLER_COUNT / BENCHMARK_EVENT_ID_COUNT,
            BENCHMARK_EVENT_COUNT * (BENCHMARK_HANDLER_COUNT / BENCHMARK_EVENT_ID_COUNT - 1),
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95
        );
        assert!(
            optimized_p95 * 4 <= legacy_p95 * 3,
            "optimized P95 {optimized_p95}ns must be no more than 75% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_handler(index: usize) -> SoundDynamicEventHandlerDescriptor {
        handler(
            format!("benchmark.event.{:02}", index % BENCHMARK_EVENT_ID_COUNT).as_str(),
            format!("plugin-{index:03}").as_str(),
            format!("handler-{index:03}").as_str(),
            (index % 8) as i32,
        )
    }

    fn benchmark_invocation(index: usize) -> SoundDynamicEventInvocation {
        invocation(format!("benchmark.event.{:02}", index % BENCHMARK_EVENT_ID_COUNT).as_str())
    }

    fn handler(
        event_id: &str,
        plugin_id: &str,
        handler_id: &str,
        priority: i32,
    ) -> SoundDynamicEventHandlerDescriptor {
        SoundDynamicEventHandlerDescriptor {
            plugin_id: plugin_id.to_string(),
            handler_id: handler_id.to_string(),
            event_id: event_id.to_string(),
            display_name: handler_id.to_string(),
            priority,
        }
    }

    fn invocation(event_id: &str) -> SoundDynamicEventInvocation {
        SoundDynamicEventInvocation {
            event_id: event_id.to_string(),
            source_path: Some("Benchmark/Dispatch".to_string()),
            time_seconds: 1.0,
            payload_schema: "benchmark/v1".to_string(),
            payload: vec![1, 2, 3, 4],
        }
    }

    fn benchmark_paired_samples(
        pending: &[SoundDynamicEventInvocation],
        mut legacy: impl FnMut(&mut Vec<SoundDynamicEventInvocation>) -> Vec<SoundDynamicEventDelivery>,
        mut optimized: impl FnMut(
            &mut Vec<SoundDynamicEventInvocation>,
        ) -> Vec<SoundDynamicEventDelivery>,
    ) -> (Vec<u128>, Vec<u128>) {
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(pending, &mut legacy));
                optimized_samples.push(benchmark_sample(pending, &mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(pending, &mut optimized));
                legacy_samples.push(benchmark_sample(pending, &mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(
        pending: &[SoundDynamicEventInvocation],
        operation: &mut impl FnMut(
            &mut Vec<SoundDynamicEventInvocation>,
        ) -> Vec<SoundDynamicEventDelivery>,
    ) -> u128 {
        let mut elapsed = 0_u128;
        for _ in 0..BENCHMARK_ITERATIONS_PER_SAMPLE {
            let mut events = pending.to_vec();
            let started = Instant::now();
            black_box(operation(&mut events));
            elapsed += started.elapsed().as_nanos();
        }
        elapsed / BENCHMARK_ITERATIONS_PER_SAMPLE as u128
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }

    fn legacy_dispatch_dynamic_events(
        handlers: &[SoundDynamicEventHandlerDescriptor],
        pending: &mut Vec<SoundDynamicEventInvocation>,
    ) -> Vec<SoundDynamicEventDelivery> {
        let pending_events = pending.drain(..).collect::<Vec<_>>();
        let mut deliveries = Vec::new();
        for invocation in pending_events {
            let mut matching_handlers = handlers
                .iter()
                .filter(|handler| handler.event_id == invocation.event_id)
                .cloned()
                .collect::<Vec<_>>();
            matching_handlers.sort_by(dynamic_event_handler_dispatch_order);
            deliveries.extend(matching_handlers.into_iter().map(|handler| {
                SoundDynamicEventDelivery {
                    handler,
                    invocation: invocation.clone(),
                }
            }));
        }
        deliveries
    }
}
