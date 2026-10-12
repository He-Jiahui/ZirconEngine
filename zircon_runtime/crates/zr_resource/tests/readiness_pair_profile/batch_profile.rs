use super::{
    insert, legacy_vec_sort_refresh, percentile, profile_record, ResourceAuthority, ResourceId,
    ResourceKind, MEASURED_PAIRS, WARMUP_PAIRS,
};
use crate::test_profile::{
    begin_allocation_profile, finish_allocation_profile, AllocationSnapshot,
};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

const DISTINCT_COUNTS: [usize; 3] = [3, 64, 1_024];

#[derive(Clone, Copy, Debug)]
enum BatchScenario {
    Changed,
    ChangedDuplicates,
    Revised,
    RevisedDuplicates,
    Unchanged,
    UnchangedDuplicates,
}

const BATCH_SCENARIOS: [BatchScenario; 6] = [
    BatchScenario::Changed,
    BatchScenario::ChangedDuplicates,
    BatchScenario::Revised,
    BatchScenario::RevisedDuplicates,
    BatchScenario::Unchanged,
    BatchScenario::UnchangedDuplicates,
];

impl BatchScenario {
    fn label(self) -> &'static str {
        match self {
            Self::Changed => "changed",
            Self::ChangedDuplicates => "changed_duplicates",
            Self::Revised => "revised",
            Self::RevisedDuplicates => "revised_duplicates",
            Self::Unchanged => "unchanged",
            Self::UnchangedDuplicates => "unchanged_duplicates",
        }
    }

    fn unchanged(self) -> bool {
        matches!(self, Self::Unchanged | Self::UnchangedDuplicates)
    }

    fn revised(self) -> bool {
        matches!(self, Self::Revised | Self::RevisedDuplicates)
    }

    fn duplicate_inputs(self) -> bool {
        matches!(
            self,
            Self::ChangedDuplicates | Self::RevisedDuplicates | Self::UnchangedDuplicates
        )
    }
}

struct BatchFixture {
    authority: ResourceAuthority,
    update_ids: Vec<ResourceId>,
    observed_ids: Vec<ResourceId>,
}

fn batch_fixture(distinct_count: usize, scenario: BatchScenario) -> BatchFixture {
    assert!(distinct_count >= 3);
    let mut authority = ResourceAuthority::default();
    let mut observed_ids = Vec::with_capacity(distinct_count);
    let mut previous_id = None;
    for index in 0..distinct_count {
        let mut record = profile_record(
            &format!("res://profile/batch/resource-{index}.png"),
            ResourceKind::Texture,
        );
        // A dependency chain exercises update ordering and recursive readiness at every scale.
        if let Some(previous_id) = previous_id {
            record = record.with_dependency_ids(vec![previous_id]);
        }
        let id = record.id;
        insert(&mut authority, record);
        observed_ids.push(id);
        previous_id = Some(id);
    }

    let mut update_ids = observed_ids.iter().copied().rev().collect::<Vec<_>>();
    if scenario.duplicate_inputs() {
        update_ids.extend(observed_ids.iter().copied());
    }
    if scenario.unchanged() || scenario.revised() {
        // Each arm starts with the same published state; warm-up is outside the sample.
        authority.refresh_readiness_many(update_ids.iter().copied());
    }
    if scenario.revised() {
        // Genuine replacement of already published sources, unlike initial "changed".
        // Record replacement and its allocation stay outside all measurement windows.
        for &id in &observed_ids {
            let mut record = authority.registry.get(id).expect("seeded source").clone();
            record.revision += 1;
            assert!(authority.registry.insert_unchecked(record).is_some());
        }
    }
    BatchFixture {
        authority,
        update_ids,
        observed_ids,
    }
}

fn assert_batch_equivalent(
    legacy: &ResourceAuthority,
    candidate: &ResourceAuthority,
    observed_ids: &[ResourceId],
) {
    let legacy_generation = legacy.readiness.generation();
    let candidate_generation = candidate.readiness.generation();
    assert_eq!(
        legacy_generation.diagnostics(),
        candidate_generation.diagnostics()
    );
    for id in observed_ids {
        let legacy_row = legacy_generation
            .row_identity(*id)
            .expect("legacy batch row");
        let candidate_row = candidate_generation
            .row_identity(*id)
            .expect("candidate batch row");
        let legacy_row = legacy_row.row();
        let candidate_row = candidate_row.row();
        assert_eq!(legacy_row.record, candidate_row.record);
        assert_eq!(legacy_row.load_state, candidate_row.load_state);
        assert_eq!(
            legacy_row.direct_dependency_state,
            candidate_row.direct_dependency_state
        );
        assert_eq!(
            legacy_row.recursive_dependency_state,
            candidate_row.recursive_dependency_state
        );
        assert_eq!(
            legacy_row.dependency_revision,
            candidate_row.dependency_revision
        );
        assert_eq!(
            legacy_row.dependency_fingerprint,
            candidate_row.dependency_fingerprint
        );
        assert_eq!(legacy_row.payload_type_id, candidate_row.payload_type_id);
    }
}

fn assert_batch_parity(distinct_count: usize, scenario: BatchScenario) {
    let mut legacy = batch_fixture(distinct_count, scenario);
    let mut candidate = batch_fixture(distinct_count, scenario);
    let legacy_before = legacy.authority.readiness.generation();
    let candidate_before = candidate.authority.readiness.generation();
    assert_eq!(legacy_before.diagnostics(), candidate_before.diagnostics());
    legacy_vec_sort_refresh(&mut legacy.authority, legacy.update_ids.iter().copied());
    candidate
        .authority
        .refresh_readiness_many(candidate.update_ids.iter().copied());
    assert_batch_equivalent(
        &legacy.authority,
        &candidate.authority,
        &legacy.observed_ids,
    );
    let legacy_after = legacy.authority.readiness.generation();
    let candidate_after = candidate.authority.readiness.generation();
    if scenario.unchanged() {
        assert!(Arc::ptr_eq(&legacy_before, &legacy_after));
        assert!(Arc::ptr_eq(&candidate_before, &candidate_after));
    } else {
        assert!(!Arc::ptr_eq(&legacy_before, &legacy_after));
        assert!(!Arc::ptr_eq(&candidate_before, &candidate_after));
        assert_eq!(legacy_after.diagnostics().changed_row_count, distinct_count);
        assert_eq!(
            candidate_after.diagnostics().changed_row_count,
            distinct_count
        );
    }
}

#[derive(Clone, Copy)]
struct TimedSample {
    elapsed_ns: u128,
    allocations: AllocationSnapshot,
}

fn refresh_batch(fixture: &mut BatchFixture, legacy: bool) {
    let update_ids = black_box(fixture.update_ids.as_slice());
    if legacy {
        legacy_vec_sort_refresh(&mut fixture.authority, update_ids.iter().copied());
    } else {
        fixture
            .authority
            .refresh_readiness_many(update_ids.iter().copied());
    }
    black_box(fixture.authority.readiness.generation().diagnostics());
}

fn measure_batch(distinct_count: usize, scenario: BatchScenario, legacy: bool) -> TimedSample {
    let mut latency_fixture = batch_fixture(distinct_count, scenario);
    let start = Instant::now();
    refresh_batch(&mut latency_fixture, legacy);
    let elapsed_ns = start.elapsed().as_nanos();

    // Allocation counting uses a separate equivalent invocation. Its atomic counters
    // stay inactive during the latency sample, avoiding measurement overhead in that timer.
    let mut allocation_fixture = batch_fixture(distinct_count, scenario);
    begin_allocation_profile();
    refresh_batch(&mut allocation_fixture, legacy);
    let allocations = finish_allocation_profile();
    // Fixture creation/drop, allocation counter setup, and output are outside the timer.
    TimedSample {
        elapsed_ns,
        allocations,
    }
}

fn paired_batch_sample(
    distinct_count: usize,
    scenario: BatchScenario,
    legacy_first: bool,
) -> (TimedSample, TimedSample) {
    if legacy_first {
        (
            measure_batch(distinct_count, scenario, true),
            measure_batch(distinct_count, scenario, false),
        )
    } else {
        let candidate = measure_batch(distinct_count, scenario, false);
        let legacy = measure_batch(distinct_count, scenario, true);
        (legacy, candidate)
    }
}

#[test]
fn batch_scale_parity_covers_changed_unchanged_and_duplicate_dependency_chains() {
    for distinct_count in DISTINCT_COUNTS {
        for scenario in BATCH_SCENARIOS {
            assert_batch_parity(distinct_count, scenario);
        }
    }
}

#[test]
#[ignore = "isolated paired Release profile; use --release --ignored --nocapture --test-threads=1"]
fn batch_refresh_release_profile_emits_raw_paired_samples_and_allocations() {
    assert!(!cfg!(debug_assertions), "requires a Release test build");
    for distinct_count in DISTINCT_COUNTS {
        for scenario in BATCH_SCENARIOS {
            assert_batch_parity(distinct_count, scenario);
        }
    }
    println!(
        "batch_environment,os={},arch={},release_build=true,allocation_scope=process_window,allocation_bytes=requested_not_RSS,allocation_sample=separate_equivalent_invocation",
        std::env::consts::OS,
        std::env::consts::ARCH,
    );
    println!(
        "batch_config,warmup_pairs={},measured_pairs={},unit=nanoseconds,changed=initial_unpublished_sources,revised=seeded_record_revision_changes",
        WARMUP_PAIRS, MEASURED_PAIRS,
    );
    println!(
        "batch_raw,distinct_ids,scenario,pair,legacy_first,legacy_ns,candidate_ns,legacy_allocations,candidate_allocations,legacy_requested_bytes,candidate_requested_bytes,legacy_peak_live_bytes,candidate_peak_live_bytes"
    );
    for distinct_count in DISTINCT_COUNTS {
        for scenario in BATCH_SCENARIOS {
            for warmup in 0..WARMUP_PAIRS {
                let _ = paired_batch_sample(distinct_count, scenario, warmup % 2 == 0);
            }
            let mut legacy_samples = Vec::with_capacity(MEASURED_PAIRS);
            let mut candidate_samples = Vec::with_capacity(MEASURED_PAIRS);
            let mut raw_samples = Vec::with_capacity(MEASURED_PAIRS);
            for pair in 0..MEASURED_PAIRS {
                let (legacy, candidate) =
                    paired_batch_sample(distinct_count, scenario, pair % 2 == 0);
                if scenario.unchanged() {
                    // Canonical records are borrowed for the no-op check. Only the input-ID
                    // buffer is owned; record strings and dependencies must not be cloned.
                    assert_eq!(candidate.allocations.allocation_count, 1);
                    let input_count =
                        distinct_count * if scenario.duplicate_inputs() { 2 } else { 1 };
                    assert_eq!(
                        candidate.allocations.requested_bytes,
                        (input_count * std::mem::size_of::<ResourceId>()) as u64,
                    );
                }
                if !scenario.unchanged() {
                    assert!(
                        candidate.allocations.requested_bytes < legacy.allocations.requested_bytes,
                        "owned canonical source tokens must request fewer actual allocation bytes"
                    );
                    assert!(
                        candidate.allocations.allocation_count
                            <= legacy.allocations.allocation_count,
                        "changed staging must not add successful allocation events"
                    );
                }
                if scenario.revised() {
                    assert!(
                        candidate.allocations.allocation_count
                            < legacy.allocations.allocation_count,
                        "revised staging must remove canonical comparison clones and preserve identical reverse edges"
                    );
                    assert!(
                        candidate.allocations.requested_bytes < legacy.allocations.requested_bytes,
                        "revised staging must request fewer actual allocation bytes"
                    );
                }
                legacy_samples.push(legacy.elapsed_ns);
                candidate_samples.push(candidate.elapsed_ns);
                raw_samples.push((legacy, candidate));
            }
            println!(
                "batch_summary,distinct_ids={},scenario={},samples={},legacy_p50_ns={},legacy_p95_ns={},legacy_p99_ns={},candidate_p50_ns={},candidate_p95_ns={},candidate_p99_ns={}",
                distinct_count,
                scenario.label(),
                MEASURED_PAIRS,
                percentile(&legacy_samples, 50),
                percentile(&legacy_samples, 95),
                percentile(&legacy_samples, 99),
                percentile(&candidate_samples, 50),
                percentile(&candidate_samples, 95),
                percentile(&candidate_samples, 99),
            );
            for (pair, (legacy, candidate)) in raw_samples.into_iter().enumerate() {
                println!(
                    "batch_raw,{},{},{},{},{},{},{},{},{},{},{},{}",
                    distinct_count,
                    scenario.label(),
                    pair,
                    pair % 2 == 0,
                    legacy.elapsed_ns,
                    candidate.elapsed_ns,
                    legacy.allocations.allocation_count,
                    candidate.allocations.allocation_count,
                    legacy.allocations.requested_bytes,
                    candidate.allocations.requested_bytes,
                    legacy.allocations.peak_live_bytes,
                    candidate.allocations.peak_live_bytes,
                );
            }
        }
    }
}
