use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncObjectSnapshot, SyncReplicationBudget,
    SyncReplicationScheduleReport,
};

use super::NetReplicationRuntimeManager;
use super::budget::update_interval_ms;
use super::snapshot::snapshot_payload_bytes;

#[derive(Clone, Debug)]
struct ScheduledSnapshotCandidate {
    key: (NetObjectId, String),
    priority: u16,
    update_interval_ms: u64,
}

impl NetReplicationRuntimeManager {
    pub(in crate::manager) fn scheduled_snapshots_impl(
        &self,
        session: NetSessionId,
        tick_time_ms: u64,
        budget: SyncReplicationBudget,
    ) -> SyncReplicationScheduleReport {
        let mut state = self
            .state
            .lock()
            .expect("net replication state mutex poisoned");
        let mut report = SyncReplicationScheduleReport::new(session, tick_time_ms, budget);
        let candidates = ordered_snapshot_candidates(&state);

        for candidate in candidates {
            let Some(snapshot) = state.snapshots.get(&candidate.key) else {
                continue;
            };
            let snapshot = if !state.allows_interest(session, snapshot) {
                report.skipped_by_interest += 1;
                continue;
            } else if !state.snapshot_due(
                session,
                snapshot,
                tick_time_ms,
                candidate.update_interval_ms,
            ) {
                report.skipped_not_due += 1;
                continue;
            } else {
                let snapshot_bytes = snapshot_payload_bytes(snapshot);
                if !budget.allows_snapshot_count(report.sent_snapshots.len())
                    || !budget.allows_byte_count(report.used_bytes, snapshot_bytes)
                {
                    report.deferred_snapshots += 1;
                    continue;
                }
                report.used_bytes += snapshot_bytes;
                snapshot.clone()
            };

            state.mark_snapshot_replicated(session, &snapshot, tick_time_ms);
            report.sent_snapshots.push(snapshot);
        }
        report
    }
}

fn ordered_snapshot_candidates(
    state: &super::state::NetReplicationRuntimeState,
) -> Vec<ScheduledSnapshotCandidate> {
    let mut candidates = state
        .snapshots
        .iter()
        .map(|(key, snapshot)| {
            let descriptor = state.descriptors.get(&snapshot.component_type);
            ScheduledSnapshotCandidate {
                key: key.clone(),
                priority: descriptor
                    .map(|descriptor| descriptor.replication_priority)
                    .unwrap_or_default(),
                update_interval_ms: descriptor
                    .map(update_interval_ms)
                    .unwrap_or(super::MILLIS_PER_SECOND),
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.key.0.raw().cmp(&right.key.0.raw()))
            .then_with(|| left.key.1.cmp(&right.key.1))
    });
    candidates
}

impl super::state::NetReplicationRuntimeState {
    fn snapshot_due(
        &self,
        session: NetSessionId,
        snapshot: &SyncObjectSnapshot,
        tick_time_ms: u64,
        update_interval_ms: u64,
    ) -> bool {
        let key = replication_time_key(session, snapshot);
        self.last_replication_ms
            .get(&key)
            .is_none_or(|last_time_ms| {
                tick_time_ms.saturating_sub(*last_time_ms) >= update_interval_ms
            })
    }

    fn mark_snapshot_replicated(
        &mut self,
        session: NetSessionId,
        snapshot: &SyncObjectSnapshot,
        tick_time_ms: u64,
    ) {
        self.last_replication_ms
            .insert(replication_time_key(session, snapshot), tick_time_ms);
    }
}

pub(in crate::manager) fn replication_time_key(
    session: NetSessionId,
    snapshot: &SyncObjectSnapshot,
) -> (NetSessionId, NetObjectId, String) {
    (session, snapshot.object, snapshot.component_type.clone())
}

#[cfg(test)]
mod payload_clone_tests {
    use std::{hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{
        NetObjectId, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor, SyncFieldValue,
        SyncObjectSnapshot,
    };

    use super::{
        NetReplicationRuntimeManager, ordered_snapshot_candidates, snapshot_payload_bytes,
        update_interval_ms,
    };

    const BENCHMARK_SNAPSHOT_COUNT: usize = 8_192;
    const BENCHMARK_PAYLOAD_BYTES: usize = 4_096;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[derive(Clone)]
    struct LegacyScheduledSnapshotCandidate {
        snapshot: SyncObjectSnapshot,
        priority: u16,
        update_interval_ms: u64,
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn lazy_replication_payload_clone_release_benchmark_evidence() {
        let manager = benchmark_manager();
        let state = manager
            .state
            .lock()
            .expect("net replication state mutex poisoned");

        let legacy_order = legacy_ordered_snapshot_candidates(&state)
            .into_iter()
            .map(|candidate| (candidate.snapshot.object, candidate.snapshot.component_type))
            .collect::<Vec<_>>();
        let optimized_order = ordered_snapshot_candidates(&state)
            .into_iter()
            .map(|candidate| candidate.key)
            .collect::<Vec<_>>();
        assert_eq!(legacy_order, optimized_order);

        let cloned_payload_bytes = state
            .snapshots
            .values()
            .map(snapshot_payload_bytes)
            .sum::<usize>();
        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_ordered_snapshot_candidates(&state),
            || ordered_snapshot_candidates(&state),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins10_lazy_replication_payload_clones snapshots={} payload_bytes_per_snapshot={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_eager_payload_clone_bytes={} optimized_eager_payload_clone_bytes=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_SNAPSHOT_COUNT,
            BENCHMARK_PAYLOAD_BYTES,
            BENCHMARK_SAMPLE_COUNT,
            cloned_payload_bytes,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95
        );
        assert_eq!(
            cloned_payload_bytes,
            BENCHMARK_SNAPSHOT_COUNT * BENCHMARK_PAYLOAD_BYTES
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn benchmark_manager() -> NetReplicationRuntimeManager {
        let manager = NetReplicationRuntimeManager::new();
        manager.register_component(
            SyncComponentDescriptor::new("BenchmarkState", SyncAuthority::Server)
                .with_field(SyncFieldDescriptor::new("payload", "bytes"))
                .with_replication_priority(7),
        );
        for raw in 1..=BENCHMARK_SNAPSHOT_COUNT as u64 {
            manager.publish_snapshot(
                NetObjectId::new(raw),
                "BenchmarkState",
                [SyncFieldValue::new(
                    "payload",
                    vec![raw as u8; BENCHMARK_PAYLOAD_BYTES],
                )],
            );
        }
        manager
    }

    fn legacy_ordered_snapshot_candidates(
        state: &super::super::state::NetReplicationRuntimeState,
    ) -> Vec<LegacyScheduledSnapshotCandidate> {
        let mut candidates = state
            .snapshots
            .values()
            .map(|snapshot| {
                let descriptor = state.descriptors.get(&snapshot.component_type);
                LegacyScheduledSnapshotCandidate {
                    snapshot: snapshot.clone(),
                    priority: descriptor
                        .map(|descriptor| descriptor.replication_priority)
                        .unwrap_or_default(),
                    update_interval_ms: descriptor
                        .map(update_interval_ms)
                        .unwrap_or(super::super::MILLIS_PER_SECOND),
                }
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.snapshot.object.raw().cmp(&right.snapshot.object.raw()))
                .then_with(|| {
                    left.snapshot
                        .component_type
                        .cmp(&right.snapshot.component_type)
                })
        });
        black_box(
            candidates
                .iter()
                .map(|candidate| candidate.update_interval_ms)
                .sum::<u64>(),
        );
        candidates
    }

    fn benchmark_paired_samples<L, O>(
        mut legacy: impl FnMut() -> L,
        mut optimized: impl FnMut() -> O,
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

    fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(&result);
        elapsed
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
