use zircon_runtime::core::framework::net::{
    NetObjectId, SyncDelta, SyncFieldValue, SyncObjectSnapshot,
};

use super::NetReplicationRuntimeManager;

pub const DEFAULT_TRANSFORM_INTERPOLATION_DELAY_MS: u64 = 100;
const INTERPOLATION_SAMPLE_LIMIT: usize = 8;

impl NetReplicationRuntimeManager {
    pub fn apply_delta(&self, delta: SyncDelta) -> Option<SyncObjectSnapshot> {
        self.apply_delta_impl(delta)
    }

    pub fn apply_delta_at(
        &self,
        delta: SyncDelta,
        receive_time_ms: u64,
    ) -> Option<SyncObjectSnapshot> {
        self.apply_delta_at_impl(delta, receive_time_ms)
    }

    pub fn interpolated_f32_field(
        &self,
        object: NetObjectId,
        component_type: &str,
        field_name: &str,
        render_time_ms: u64,
    ) -> Option<f32> {
        self.interpolated_f32_field_with_delay(
            object,
            component_type,
            field_name,
            render_time_ms,
            DEFAULT_TRANSFORM_INTERPOLATION_DELAY_MS,
        )
    }

    pub fn interpolated_f32_field_with_delay(
        &self,
        object: NetObjectId,
        component_type: &str,
        field_name: &str,
        render_time_ms: u64,
        delay_ms: u64,
    ) -> Option<f32> {
        let target_time_ms = render_time_ms.saturating_sub(delay_ms);
        let state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let samples = state
            .interpolation_samples
            .get(component_type)?
            .get(&object)?
            .get(field_name)?;
        interpolate_f32_samples(samples, target_time_ms)
    }

    pub(in crate::manager) fn apply_delta_impl(
        &self,
        delta: SyncDelta,
    ) -> Option<SyncObjectSnapshot> {
        self.apply_delta_at_impl(delta, 0)
    }

    pub(in crate::manager) fn apply_delta_at_impl(
        &self,
        delta: SyncDelta,
        receive_time_ms: u64,
    ) -> Option<SyncObjectSnapshot> {
        let mut state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let key = (delta.object, delta.component_type.clone());
        if state
            .sequences
            .get(&key)
            .is_some_and(|sequence| delta.sequence <= *sequence)
        {
            return state.snapshots.get(&key).cloned();
        }

        if delta.is_despawn() {
            state.sequences.insert(key.clone(), delta.sequence);
            state.remove_replication_times(delta.object, &delta.component_type);
            state.remove_interpolation_samples(delta.object, &delta.component_type);
            state.snapshots.remove(&key);
            return None;
        }

        let descriptor = state.descriptors.get(&delta.component_type)?.clone();
        let mut fields = state
            .snapshots
            .get(&key)
            .map(|snapshot| snapshot.fields.clone())
            .unwrap_or_default();
        let changed_fields = delta.changed_fields;
        if should_record_interpolation(&delta.component_type) {
            record_interpolation_samples(
                &mut state,
                delta.object,
                &delta.component_type,
                receive_time_ms,
                &changed_fields,
            );
        }
        merge_delta_fields(&mut fields, changed_fields);

        let snapshot = SyncObjectSnapshot::new(delta.object, &descriptor, fields);
        state.sequences.insert(key.clone(), delta.sequence);
        state.snapshots.insert(key, snapshot.clone());
        Some(snapshot)
    }
}

fn should_record_interpolation(component_type: &str) -> bool {
    component_type.eq_ignore_ascii_case("Transform")
        || component_type
            .split("::")
            .any(|segment| segment.eq_ignore_ascii_case("Transform"))
        || component_type.to_ascii_lowercase().contains("transform")
}

fn record_interpolation_samples(
    state: &mut super::state::NetReplicationRuntimeState,
    object: NetObjectId,
    component_type: &str,
    time_ms: u64,
    changed_fields: &[SyncFieldValue],
) {
    for field in changed_fields {
        if f32_from_bytes(&field.bytes).is_none() {
            continue;
        }
        let samples = state
            .interpolation_samples
            .entry(component_type.to_owned())
            .or_default()
            .entry(object)
            .or_default()
            .entry(field.name.clone())
            .or_default();
        samples.push(super::state::NetReplicationInterpolationSample {
            time_ms,
            bytes: field.bytes.clone(),
        });
        samples.sort_by(|left, right| left.time_ms.cmp(&right.time_ms));
        if samples.len() > INTERPOLATION_SAMPLE_LIMIT {
            let overflow = samples.len() - INTERPOLATION_SAMPLE_LIMIT;
            samples.drain(0..overflow);
        }
    }
}

fn merge_delta_fields(fields: &mut Vec<SyncFieldValue>, changed_fields: Vec<SyncFieldValue>) {
    for changed in changed_fields {
        if let Some(existing) = fields.iter_mut().find(|field| field.name == changed.name) {
            *existing = changed;
        } else {
            fields.push(changed);
        }
    }
}

fn interpolate_f32_samples(
    samples: &[super::state::NetReplicationInterpolationSample],
    target_time_ms: u64,
) -> Option<f32> {
    let mut previous = None;
    for sample in samples {
        if sample.time_ms == target_time_ms {
            return f32_from_bytes(&sample.bytes);
        }
        if sample.time_ms < target_time_ms {
            previous = Some(sample);
            continue;
        }

        let next_value = f32_from_bytes(&sample.bytes)?;
        let Some(previous) = previous else {
            return Some(next_value);
        };
        let previous_value = f32_from_bytes(&previous.bytes)?;
        let span = sample.time_ms.saturating_sub(previous.time_ms);
        if span == 0 {
            return Some(next_value);
        }
        let alpha = target_time_ms.saturating_sub(previous.time_ms) as f32 / span as f32;
        return Some(previous_value + (next_value - previous_value) * alpha);
    }

    previous.and_then(|sample| f32_from_bytes(&sample.bytes))
}

fn f32_from_bytes(bytes: &[u8]) -> Option<f32> {
    let bytes = bytes.get(..4)?;
    Some(f32::from_le_bytes(bytes.try_into().ok()?))
}

#[cfg(test)]
mod borrowed_interpolation_index_tests {
    use std::{collections::HashMap, hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{
        NetObjectId, SyncAuthority, SyncComponentDescriptor, SyncDelta, SyncFieldDescriptor,
        SyncFieldValue,
    };

    use super::super::state::{NetReplicationInterpolationSample, NetReplicationRuntimeState};
    use super::NetReplicationRuntimeManager;

    const BENCHMARK_KEY_COUNT: usize = 4_096;
    const BENCHMARK_LOOKUP_REPEATS: usize = 64;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    type LegacyInterpolationSamples =
        HashMap<(NetObjectId, String, String), Vec<NetReplicationInterpolationSample>>;

    #[test]
    fn component_scoped_cleanup_preserves_sibling_interpolation_samples() {
        let manager = NetReplicationRuntimeManager::new();
        for component_type in ["Transform", "TransformProxy"] {
            manager.register_component(
                SyncComponentDescriptor::new(component_type, SyncAuthority::Server)
                    .with_field(SyncFieldDescriptor::new("x", "f32")),
            );
        }

        let object = NetObjectId::new(17);
        for (sequence, component_type, value) in
            [(1, "Transform", 3.0_f32), (1, "TransformProxy", 9.0_f32)]
        {
            manager
                .apply_delta_at(
                    SyncDelta::new(
                        object,
                        component_type,
                        sequence,
                        [SyncFieldValue::new("x", value.to_le_bytes())],
                    ),
                    100,
                )
                .expect("registered component accepts delta");
        }

        manager.apply_delta_at(SyncDelta::despawn(object, "Transform", 2), 200);

        assert_eq!(
            manager.interpolated_f32_field_with_delay(object, "Transform", "x", 100, 0),
            None
        );
        assert_eq!(
            manager.interpolated_f32_field_with_delay(object, "TransformProxy", "x", 100, 0),
            Some(9.0)
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn borrowed_interpolation_index_release_benchmark_evidence() {
        let (legacy, optimized, queries) = benchmark_indexes();
        assert_eq!(
            legacy_lookup_checksum(&legacy, &queries),
            optimized_lookup_checksum(&optimized, &queries)
        );

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_lookup_checksum(&legacy, &queries),
            || optimized_lookup_checksum(&optimized, &queries),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        let lookups_per_sample = BENCHMARK_KEY_COUNT * BENCHMARK_LOOKUP_REPEATS;

        println!(
            "PERF_RESULT task=plugins10_borrowed_interpolation_index keys={BENCHMARK_KEY_COUNT} lookups_per_sample={lookups_per_sample} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_query_string_allocations_per_sample={} optimized_query_string_allocations_per_sample=0 threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_ns} optimized_raw_ns={optimized_ns}",
            lookups_per_sample * 2,
        );
        assert!(
            optimized_p95 * 100 <= legacy_p95 * 85,
            "optimized P95 {optimized_p95}ns must be at least 15% lower than legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_indexes() -> (
        LegacyInterpolationSamples,
        NetReplicationRuntimeState,
        Vec<(NetObjectId, String, String)>,
    ) {
        let mut legacy = LegacyInterpolationSamples::new();
        let mut optimized = NetReplicationRuntimeState::default();
        let mut queries = Vec::with_capacity(BENCHMARK_KEY_COUNT);
        for raw in 1..=BENCHMARK_KEY_COUNT as u64 {
            let object = NetObjectId::new(raw);
            let component_type = "TransformBenchmarkComponent_LongBorrowedLookupKey".to_string();
            let field_name = "position_axis_LongBorrowedLookupKey".to_string();
            let samples = vec![NetReplicationInterpolationSample {
                time_ms: raw,
                bytes: (raw as f32).to_le_bytes().to_vec(),
            }];
            legacy.insert(
                (object, component_type.clone(), field_name.clone()),
                samples.clone(),
            );
            optimized
                .interpolation_samples
                .entry(component_type.clone())
                .or_default()
                .entry(object)
                .or_default()
                .insert(field_name.clone(), samples);
            queries.push((object, component_type, field_name));
        }
        (legacy, optimized, queries)
    }

    fn legacy_lookup_checksum(
        samples: &LegacyInterpolationSamples,
        queries: &[(NetObjectId, String, String)],
    ) -> usize {
        let mut matched = 0;
        for _ in 0..BENCHMARK_LOOKUP_REPEATS {
            for (object, component_type, field_name) in black_box(queries) {
                let key = (
                    *object,
                    black_box(component_type.as_str()).to_string(),
                    black_box(field_name.as_str()).to_string(),
                );
                matched += black_box(samples.get(&key).map_or(0, Vec::len));
            }
        }
        black_box(matched)
    }

    fn optimized_lookup_checksum(
        state: &NetReplicationRuntimeState,
        queries: &[(NetObjectId, String, String)],
    ) -> usize {
        let mut matched = 0;
        for _ in 0..BENCHMARK_LOOKUP_REPEATS {
            for (object, component_type, field_name) in black_box(queries) {
                matched += black_box(
                    state
                        .interpolation_samples
                        .get(black_box(component_type.as_str()))
                        .and_then(|objects| objects.get(object))
                        .and_then(|fields| fields.get(black_box(field_name.as_str())))
                        .map_or(0, Vec::len),
                );
            }
        }
        black_box(matched)
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> usize,
        mut optimized: impl FnMut() -> usize,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        let matched = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(matched);
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
