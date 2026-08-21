use std::collections::{HashMap, VecDeque};
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::*;

struct SequencedHandler {
    snapshots: Arc<Mutex<Vec<u64>>>,
    applications: Arc<Mutex<Vec<u64>>>,
}

impl RuntimeOperationHandler for SequencedHandler {
    fn snapshot(
        &self,
        _context: RuntimeOperationContext<'_>,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, RuntimeOperationHandlerError> {
        let sequence = payload
            .as_u64()
            .expect("sequence fixture payload remains an integer");
        self.snapshots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(sequence);
        Ok(payload)
    }

    fn prepare(
        &self,
        snapshot: serde_json::Value,
    ) -> Result<RuntimeOperationPrepared, RuntimeOperationHandlerError> {
        Ok(RuntimeOperationPrepared::new(snapshot.clone(), snapshot))
    }

    fn apply(
        &self,
        _context: RuntimeOperationContext<'_>,
        command: serde_json::Value,
    ) -> Result<(), RuntimeOperationHandlerError> {
        let sequence = command
            .as_u64()
            .expect("sequence fixture command remains an integer");
        self.applications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(sequence);
        Ok(())
    }
}

#[test]
fn operation_service_dispatches_phase_work_in_submission_order() {
    const TASK_COUNT: u64 = 128;

    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let applications = Arc::new(Mutex::new(Vec::new()));
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: TASK_COUNT as usize,
        max_in_flight_prepares: 1,
        max_retained_bytes: 16 * 1024,
        max_owner_applies_per_tick: 8,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.sequence",
            Arc::new(SequencedHandler {
                snapshots: Arc::clone(&snapshots),
                applications: Arc::clone(&applications),
            }),
        )
        .unwrap();
    let handles: Vec<_> = (0..TASK_COUNT)
        .map(|sequence| {
            service
                .submit(ZrRuntimeOperationSubmitRequestV1::new(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    "test.sequence",
                    json!(sequence),
                ))
                .unwrap()
        })
        .collect();
    let runtime = CoreRuntime::new();
    let mut world = World::empty();

    for _ in 0..16_384 {
        service.tick(&runtime.handle(), &mut world);
        if handles.iter().all(|handle| {
            service
                .poll(*handle)
                .unwrap()
                .phase()
                .is_some_and(ZrRuntimeOperationPhase::is_terminal)
        }) {
            break;
        }
        std::thread::yield_now();
    }

    let expected: Vec<_> = (0..TASK_COUNT).collect();
    assert_eq!(
        *snapshots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        expected
    );
    assert_eq!(
        *applications
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        expected
    );
}

#[test]
fn operation_service_does_not_bypass_an_unarmed_fifo_head() {
    let snapshots = Arc::new(Mutex::new(Vec::new()));
    let applications = Arc::new(Mutex::new(Vec::new()));
    let mut service = RuntimeOperationService::with_limits(RuntimeOperationLimits {
        max_tasks: 2,
        max_in_flight_prepares: 1,
        max_retained_bytes: 1_024,
        max_owner_applies_per_tick: 1,
        terminal_result_ttl: Duration::from_secs(60),
    });
    service
        .register_handler(
            "test.sequence",
            Arc::new(SequencedHandler {
                snapshots: Arc::clone(&snapshots),
                applications,
            }),
        )
        .unwrap();
    let first = service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.sequence",
            json!(0),
        ))
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    service.set_deadline_state_for_test(first, Some(deadline), false);
    service
        .submit(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            "test.sequence",
            json!(1),
        ))
        .unwrap();

    let runtime = CoreRuntime::new();
    let mut world = World::empty();
    service.tick(&runtime.handle(), &mut world);
    assert!(
        snapshots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty(),
        "a later task must not bypass an unarmed FIFO head"
    );

    service.set_deadline_state_for_test(first, Some(deadline), true);
    for _ in 0..1_024 {
        service.tick(&runtime.handle(), &mut world);
        if snapshots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
            == 2
        {
            break;
        }
        std::thread::yield_now();
    }
    assert_eq!(
        *snapshots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec![0, 1]
    );
}

#[test]
fn operation_service_phase_selection_uses_indexed_queues() {
    let service = include_str!("../service.rs");
    let admission = include_str!("../service/admission.rs");
    let completion = include_str!("../service/completion.rs");
    let queued_start = service
        .find("    fn take_queued_snapshot_task(")
        .expect("queued selection owner");
    let queued_end = service[queued_start..]
        .find("    fn finish_snapshot_failed_task(")
        .map(|offset| queued_start + offset)
        .expect("queued selection boundary");
    let ready_start = service
        .find("    fn take_prepared_task(")
        .expect("ready selection owner");
    let ready_end = service[ready_start..]
        .find("    fn finish_completed_task(")
        .map(|offset| ready_start + offset)
        .expect("ready selection boundary");
    let phase_selection = format!(
        "{}{}",
        &service[queued_start..queued_end],
        &service[ready_start..ready_end]
    );

    assert!(service.contains("queued_snapshot_tasks: VecDeque<ZrRuntimeOperationHandle>"));
    assert!(service.contains("ready_apply_tasks: VecDeque<ZrRuntimeOperationHandle>"));
    assert!(admission.contains("queued_snapshot_tasks.push_back(handle)"));
    assert!(completion.contains("ready_apply_tasks.push_back(handle)"));
    assert!(phase_selection.contains("pop_front()"));
    assert!(!phase_selection.contains("state.tasks.iter().find_map"));
    assert!(service.contains("fn compact_phase_indexes("));
    assert!(service.contains("queued_snapshot_tasks.retain"));
    assert!(service.contains("ready_apply_tasks.retain"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn operation_service_phase_index_release_benchmark_evidence() {
    const TASK_COUNT: u64 = 1_024;
    const CYCLES: usize = 64;
    const SAMPLE_PAIRS: usize = 21;

    for (task, evidence_id, ready_phase) in [
        (
            "queued_snapshot_index",
            "operation_queued_snapshot_index",
            1_u8,
        ),
        ("ready_apply_index", "operation_ready_apply_index", 2_u8),
    ] {
        let evidence = measure_phase_index_workload(TASK_COUNT, CYCLES, SAMPLE_PAIRS, ready_phase);
        write_phase_index_evidence(task, evidence_id, TASK_COUNT, CYCLES, &evidence);
    }
}

struct PhaseIndexBenchmarkEvidence {
    legacy_samples_ns: Vec<u128>,
    indexed_samples_ns: Vec<u128>,
    legacy_probes: u128,
    indexed_probes: u128,
}

fn measure_phase_index_workload(
    task_count: u64,
    cycles: usize,
    sample_pairs: usize,
    ready_phase: u8,
) -> PhaseIndexBenchmarkEvidence {
    let mut legacy_samples_ns = Vec::with_capacity(sample_pairs);
    let mut indexed_samples_ns = Vec::with_capacity(sample_pairs);
    for sample_index in 0..sample_pairs {
        let mut legacy_cycles: Vec<_> = (0..cycles)
            .map(|_| {
                (1..=task_count)
                    .map(|handle| (handle, ready_phase))
                    .collect::<HashMap<_, _>>()
            })
            .collect();
        let mut indexed_cycles: Vec<_> = (0..cycles)
            .map(|_| (1..=task_count).collect::<VecDeque<_>>())
            .collect();
        let mut measure_legacy = || {
            let started = Instant::now();
            for phases in &mut legacy_cycles {
                for _ in 0..task_count {
                    let handle = phases
                        .iter()
                        .find_map(|(handle, phase)| (*phase == ready_phase).then_some(*handle))
                        .expect("one legacy phase entry remains ready");
                    *phases.get_mut(&handle).unwrap() = 0;
                    black_box(handle);
                }
            }
            legacy_samples_ns.push(started.elapsed().as_nanos());
        };
        let mut measure_indexed = || {
            let started = Instant::now();
            for handles in &mut indexed_cycles {
                while let Some(handle) = handles.pop_front() {
                    black_box(handle);
                }
            }
            indexed_samples_ns.push(started.elapsed().as_nanos());
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_indexed();
        } else {
            measure_indexed();
            measure_legacy();
        }
    }

    let legacy_probes = u128::from(task_count)
        .checked_mul(u128::from(task_count + 1))
        .and_then(|probes| probes.checked_div(2))
        .and_then(|probes| probes.checked_mul(cycles as u128))
        .unwrap();
    let indexed_probes = u128::from(task_count).checked_mul(cycles as u128).unwrap();
    PhaseIndexBenchmarkEvidence {
        legacy_samples_ns,
        indexed_samples_ns,
        legacy_probes,
        indexed_probes,
    }
}

fn write_phase_index_evidence(
    task: &str,
    evidence_id: &str,
    task_count: u64,
    cycles: usize,
    evidence: &PhaseIndexBenchmarkEvidence,
) {
    let legacy_p95_ns = nearest_rank_percentile(&evidence.legacy_samples_ns, 95);
    let indexed_p95_ns = nearest_rank_percentile(&evidence.indexed_samples_ns, 95);

    let legacy = evidence
        .legacy_samples_ns
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let indexed = evidence
        .indexed_samples_ns
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let sample_pairs = evidence.legacy_samples_ns.len();
    println!(
        "OPERATION_PHASE_INDEX_BENCH_V1 task={task} evidence_id={evidence_id} samples_shared=false \
         tasks={task_count} cycles={cycles} sample_pairs={sample_pairs} \
         legacy_probes={} indexed_probes={} \
         legacy_p95_ns={legacy_p95_ns} indexed_p95_ns={indexed_p95_ns} \
         legacy_ns={legacy} indexed_ns={indexed}",
        evidence.legacy_probes, evidence.indexed_probes,
    );
    assert!(
        indexed_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed P95 {indexed_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}
