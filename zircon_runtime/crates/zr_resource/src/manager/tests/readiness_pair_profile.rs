use std::hint::black_box;
use std::time::Instant;

use crate::{ResourceId, ResourceKind, ResourceLocator, ResourceReadinessState, ResourceRecord};

use super::ResourceAuthority;

const WARMUP_PAIRS: usize = 3;
const MEASURED_PAIRS: usize = 101;

fn profile_record(locator_text: &str, kind: ResourceKind) -> ResourceRecord {
    let locator = ResourceLocator::parse(locator_text).expect("valid locator");
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator)
}

#[derive(Clone, Copy, Debug)]
enum Scenario {
    DistinctPair,
    DuplicatePair,
    DanglingDependency,
    LostDependencyRow,
}

const SCENARIOS: [Scenario; 4] = [
    Scenario::DistinctPair,
    Scenario::DuplicatePair,
    Scenario::DanglingDependency,
    Scenario::LostDependencyRow,
];

impl Scenario {
    fn label(self) -> &'static str {
        match self {
            Self::DistinctPair => "distinct_pair",
            Self::DuplicatePair => "duplicate_pair",
            Self::DanglingDependency => "dangling_dependency",
            Self::LostDependencyRow => "lost_dependency_row",
        }
    }
}

struct Fixture {
    authority: ResourceAuthority,
    update_ids: [ResourceId; 2],
    observed_ids: [ResourceId; 3],
    expected_failed: Option<ResourceId>,
    expected_absent: Option<ResourceId>,
}

fn reverse_pair(first: ResourceId, second: ResourceId) -> [ResourceId; 2] {
    if first < second {
        [second, first]
    } else {
        [first, second]
    }
}

fn insert(authority: &mut ResourceAuthority, record: crate::ResourceRecord) {
    assert!(authority.registry.insert_unchecked(record).is_none());
}

fn fixture(scenario: Scenario) -> Fixture {
    let mut authority = ResourceAuthority::default();

    match scenario {
        Scenario::DistinctPair => {
            let model = profile_record("res://profile/distinct.glb", ResourceKind::Model);
            let texture = profile_record("res://profile/distinct.png", ResourceKind::Texture);
            let model_id = model.id;
            let texture_id = texture.id;
            insert(&mut authority, model);
            insert(&mut authority, texture);
            Fixture {
                authority,
                update_ids: reverse_pair(model_id, texture_id),
                observed_ids: [model_id, texture_id, texture_id],
                expected_failed: None,
                expected_absent: None,
            }
        }
        Scenario::DuplicatePair => {
            let model = profile_record("res://profile/duplicate.glb", ResourceKind::Model);
            let id = model.id;
            insert(&mut authority, model);
            Fixture {
                authority,
                update_ids: [id, id],
                observed_ids: [id, id, id],
                expected_failed: None,
                expected_absent: None,
            }
        }
        Scenario::DanglingDependency => {
            let missing_id = ResourceId::from_stable_label("runtime51-profile-missing-dependency");
            let parent = profile_record("res://profile/dangling-parent.glb", ResourceKind::Model)
                .with_dependency_ids(vec![missing_id]);
            let peer = profile_record("res://profile/dangling-peer.png", ResourceKind::Texture);
            let parent_id = parent.id;
            let peer_id = peer.id;
            insert(&mut authority, parent);
            insert(&mut authority, peer);
            Fixture {
                authority,
                update_ids: reverse_pair(parent_id, peer_id),
                observed_ids: [parent_id, peer_id, missing_id],
                expected_failed: Some(parent_id),
                expected_absent: Some(missing_id),
            }
        }
        Scenario::LostDependencyRow => {
            let lost = profile_record("res://profile/lost-dependency.png", ResourceKind::Texture);
            let lost_id = lost.id;
            let survivor = profile_record("res://profile/lost-dependent.glb", ResourceKind::Model)
                .with_dependency_ids(vec![lost_id]);
            let survivor_id = survivor.id;
            insert(&mut authority, lost);
            insert(&mut authority, survivor);

            // Seed the same published state for both algorithms before timing, then lose one row.
            authority.refresh_readiness_many([lost_id, survivor_id]);
            assert!(authority.registry.remove_by_id(lost_id).is_some());

            Fixture {
                authority,
                update_ids: reverse_pair(lost_id, survivor_id),
                observed_ids: [survivor_id, lost_id, lost_id],
                expected_failed: Some(survivor_id),
                expected_absent: Some(lost_id),
            }
        }
    }
}

fn legacy_vec_sort_refresh(
    authority: &mut ResourceAuthority,
    ids: impl IntoIterator<Item = ResourceId>,
) {
    let mut ids = ids.into_iter();
    let Some(first) = ids.next() else {
        return;
    };
    let Some(second) = ids.next() else {
        let update = authority.readiness_source_update(first);
        authority.readiness.apply_updates([update]);
        return;
    };

    // Mirrors the pre-fast-path multi-ID algorithm for an exact pair.
    let (remaining_lower_bound, _) = ids.size_hint();
    let mut unique_ids = Vec::with_capacity(remaining_lower_bound.saturating_add(2));
    unique_ids.push(first);
    unique_ids.push(second);
    unique_ids.extend(ids);
    unique_ids.sort_unstable();
    unique_ids.dedup();

    let updates = unique_ids
        .into_iter()
        .map(|id| authority.readiness_source_update(id))
        .collect::<Vec<_>>();
    authority.readiness.apply_updates(updates);
}

fn assert_equivalent(
    scenario: Scenario,
    phase: &str,
    legacy: &ResourceAuthority,
    candidate: &ResourceAuthority,
    observed_ids: [ResourceId; 3],
) {
    let legacy_generation = legacy.readiness.generation();
    let candidate_generation = candidate.readiness.generation();
    assert_eq!(
        legacy_generation.diagnostics(),
        candidate_generation.diagnostics(),
        "{} {phase} diagnostics",
        scenario.label()
    );

    for id in observed_ids {
        let legacy_row = legacy_generation.row_identity(id);
        let candidate_row = candidate_generation.row_identity(id);
        match (legacy_row, candidate_row) {
            (Some(legacy_row), Some(candidate_row)) => {
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
            (None, None) => {}
            _ => panic!(
                "{} {phase} row presence differs for {id:?}",
                scenario.label()
            ),
        }
    }
}

fn assert_expected_outcome(scenario: Scenario, fixture: &Fixture) {
    let generation = fixture.authority.readiness.generation();
    if let Some(id) = fixture.expected_absent {
        assert!(
            generation.row_identity(id).is_none(),
            "{} should prune lost row {id:?}",
            scenario.label()
        );
    }
    if let Some(id) = fixture.expected_failed {
        let row = generation
            .row_identity(id)
            .expect("failed dependency fixture row");
        assert_eq!(
            row.row().direct_dependency_state,
            ResourceReadinessState::Failed
        );
        assert_eq!(
            row.row().recursive_dependency_state,
            ResourceReadinessState::Failed
        );
    }
}

fn assert_semantic_parity(scenario: Scenario) {
    let mut legacy = fixture(scenario);
    let mut candidate = fixture(scenario);
    assert_equivalent(
        scenario,
        "before",
        &legacy.authority,
        &candidate.authority,
        legacy.observed_ids,
    );

    legacy_vec_sort_refresh(&mut legacy.authority, legacy.update_ids);
    candidate
        .authority
        .refresh_readiness_many(candidate.update_ids);

    assert_equivalent(
        scenario,
        "after",
        &legacy.authority,
        &candidate.authority,
        legacy.observed_ids,
    );
    assert_expected_outcome(scenario, &legacy);
    assert_expected_outcome(scenario, &candidate);
}

fn measure_legacy(scenario: Scenario) -> u128 {
    let mut fixture = fixture(scenario);
    let start = Instant::now();
    let update_ids = black_box(fixture.update_ids);
    legacy_vec_sort_refresh(&mut fixture.authority, update_ids);
    black_box(fixture.authority.readiness.generation().diagnostics());
    let elapsed = start.elapsed();
    elapsed.as_nanos()
}

fn measure_candidate(scenario: Scenario) -> u128 {
    let mut fixture = fixture(scenario);
    let start = Instant::now();
    let update_ids = black_box(fixture.update_ids);
    fixture.authority.refresh_readiness_many(update_ids);
    black_box(fixture.authority.readiness.generation().diagnostics());
    let elapsed = start.elapsed();
    elapsed.as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = ((sorted.len() * percentile + 99) / 100).saturating_sub(1);
    sorted[rank]
}

#[test]
fn nearest_rank_p95_uses_sample_96_of_101() {
    let samples = (1u128..=101).collect::<Vec<_>>();

    assert_eq!(percentile(&samples, 50), 51);
    assert_eq!(percentile(&samples, 95), 96);
}

fn timed_pair(scenario: Scenario, legacy_first: bool) -> (u128, u128) {
    if legacy_first {
        (measure_legacy(scenario), measure_candidate(scenario))
    } else {
        let candidate = measure_candidate(scenario);
        let legacy = measure_legacy(scenario);
        (legacy, candidate)
    }
}

#[test]
fn exact_two_pair_api_matches_legacy_vec_algorithm_semantics() {
    for scenario in SCENARIOS {
        assert_semantic_parity(scenario);
    }
}

#[test]
fn multi_id_refresh_matches_legacy_with_duplicates_and_retains_unchanged_generation() {
    for scenario in SCENARIOS {
        let mut legacy = fixture(scenario);
        let mut candidate = fixture(scenario);
        let [first, second] = legacy.update_ids;
        let update_ids = [second, first, second, first];

        legacy_vec_sort_refresh(&mut legacy.authority, update_ids);
        candidate.authority.refresh_readiness_many(update_ids);

        assert_equivalent(
            scenario,
            "multi-ID refresh",
            &legacy.authority,
            &candidate.authority,
            legacy.observed_ids,
        );
        assert_expected_outcome(scenario, &legacy);
        assert_expected_outcome(scenario, &candidate);

        let published = candidate.authority.readiness.generation();
        candidate
            .authority
            .refresh_readiness_many([first, second, first, second]);
        assert!(std::sync::Arc::ptr_eq(
            &published,
            &candidate.authority.readiness.generation(),
        ));
    }
}

#[test]
fn canonical_noop_filter_preserves_unsorted_duplicate_dependency_semantics() {
    let make_fixture = || {
        let mut authority = ResourceAuthority::default();
        let first = profile_record("res://profile/canonical-first.png", ResourceKind::Texture);
        let second = profile_record("res://profile/canonical-second.png", ResourceKind::Texture);
        let ids = [first.id, second.id];
        let [lower, higher] = if ids[0] < ids[1] {
            ids
        } else {
            [ids[1], ids[0]]
        };
        let parent = profile_record("res://profile/canonical-parent.glb", ResourceKind::Model)
            .with_dependency_ids(vec![higher, lower, higher]);
        let observed_ids = [parent.id, first.id, second.id];
        for record in [parent, second, first] {
            insert(&mut authority, record);
        }
        (authority, observed_ids)
    };
    let (mut legacy, ids) = make_fixture();
    let (mut candidate, _) = make_fixture();
    for phase in ["initial canonicalization", "canonical no-op"] {
        let input = [ids[2], ids[0], ids[1], ids[0]];
        let before = candidate.readiness.generation();
        legacy_vec_sort_refresh(&mut legacy, input);
        candidate.refresh_readiness_many(input);
        assert_equivalent(Scenario::DistinctPair, phase, &legacy, &candidate, ids);
        if phase == "canonical no-op" {
            assert!(std::sync::Arc::ptr_eq(
                &before,
                &candidate.readiness.generation()
            ));
        }
    }
}

#[test]
fn seeded_noncanonical_revision_removal_and_recovery_match_legacy_projection() {
    let make_fixture = || {
        let mut authority = ResourceAuthority::default();
        let first = profile_record("res://profile/staged-first.png", ResourceKind::Texture);
        let second = profile_record("res://profile/staged-second.png", ResourceKind::Texture);
        let [lower, higher] = if first.id < second.id {
            [first.id, second.id]
        } else {
            [second.id, first.id]
        };
        let parent = profile_record("res://profile/staged-parent.glb", ResourceKind::Model)
            .with_dependency_ids(vec![higher, lower, higher]);
        let ids = [parent.id, lower, higher];
        for record in [parent, first, second] {
            insert(&mut authority, record);
        }
        (authority, ids)
    };
    let (mut legacy, ids) = make_fixture();
    let (mut candidate, _) = make_fixture();
    let input = [ids[2], ids[0], ids[1], ids[0], ids[2]];
    let missing = ResourceId::from_stable_label("runtime51-staged-missing");
    for phase in [
        "seed",
        "canonical no-op",
        "revised leaf",
        "revised parent and removal",
        "recovery",
    ] {
        for authority in [&mut legacy, &mut candidate] {
            if phase == "revised leaf" {
                let mut record = authority.registry.get(ids[1]).unwrap().clone();
                record.revision += 1;
                assert!(authority.registry.insert_unchecked(record).is_some());
            } else if phase == "revised parent and removal" {
                let mut record = authority.registry.get(ids[0]).unwrap().clone();
                record.revision += 1;
                record.dependency_ids = vec![missing, ids[1], missing, ids[1]];
                assert!(authority.registry.insert_unchecked(record).is_some());
                assert!(authority.registry.remove_by_id(ids[2]).is_some());
            } else if phase == "recovery" {
                let mut record = authority.registry.get(ids[0]).unwrap().clone();
                record.revision += 1;
                record.dependency_ids = vec![ids[1], ids[1]];
                assert!(authority.registry.insert_unchecked(record).is_some());
            }
        }
        let before = candidate.readiness.generation();
        legacy_vec_sort_refresh(&mut legacy, input);
        candidate.refresh_readiness_many(input);
        assert_equivalent(Scenario::DistinctPair, phase, &legacy, &candidate, ids);
        let generation = candidate.readiness.generation();
        if phase == "canonical no-op" {
            assert!(std::sync::Arc::ptr_eq(&before, &generation));
        } else {
            assert!(!std::sync::Arc::ptr_eq(&before, &generation));
        }
        let parent = generation.row(ids[0]).unwrap();
        assert!(parent
            .record
            .dependency_ids
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        if phase == "revised parent and removal" {
            assert!(generation.row(ids[2]).is_none());
            assert_eq!(
                parent.direct_dependency_state,
                ResourceReadinessState::Failed
            );
            assert_eq!(
                parent.recursive_dependency_state,
                ResourceReadinessState::Failed
            );
        } else if phase == "recovery" {
            assert_ne!(
                parent.direct_dependency_state,
                ResourceReadinessState::Failed
            );
            assert_ne!(
                parent.recursive_dependency_state,
                ResourceReadinessState::Failed
            );
        }
    }
}

#[test]
fn source_only_changes_keep_reverse_edges_for_selective_failure_and_recovery() {
    let make_fixture = || {
        let mut authority = ResourceAuthority::default();
        let leaf = profile_record(
            "res://profile/retained-edge-leaf.png",
            ResourceKind::Texture,
        );
        let middle = profile_record(
            "res://profile/retained-edge-middle.glb",
            ResourceKind::Model,
        )
        .with_dependency_ids(vec![leaf.id, leaf.id]);
        let parent = profile_record(
            "res://profile/retained-edge-parent.glb",
            ResourceKind::Model,
        )
        .with_dependency_ids(vec![middle.id]);
        let ids = [parent.id, middle.id, leaf.id];
        for record in [parent, middle, leaf] {
            insert(&mut authority, record);
        }
        (authority, ids)
    };
    let (mut legacy, ids) = make_fixture();
    let (mut candidate, _) = make_fixture();
    legacy_vec_sort_refresh(&mut legacy, ids);
    candidate.refresh_readiness_many(ids);
    let mut removed_leaf = None;
    for phase in [
        "source revision",
        "runtime failure",
        "runtime recovery",
        "leaf removal",
        "leaf recovery",
        "edge removal",
        "disconnected leaf removal",
    ] {
        let before = candidate.readiness.generation();
        let previous_record = before.row(ids[1]).unwrap().record.clone();
        for authority in [&mut legacy, &mut candidate] {
            match phase {
                "source revision" | "edge removal" => {
                    let mut record = authority.registry.get(ids[1]).unwrap().clone();
                    record.revision += 1;
                    if phase == "edge removal" {
                        record.dependency_ids.clear();
                    }
                    assert!(authority.registry.insert_unchecked(record).is_some());
                }
                "runtime failure" | "runtime recovery" => {
                    authority.runtime.entry(ids[1]).or_default().state =
                        if phase == "runtime failure" {
                            crate::RuntimeResourceState::Error
                        } else {
                            crate::RuntimeResourceState::Unloaded
                        };
                }
                "leaf removal" | "disconnected leaf removal" => {
                    let record = authority.registry.remove_by_id(ids[2]).unwrap();
                    if phase == "leaf removal" {
                        removed_leaf = Some(record);
                    }
                }
                "leaf recovery" => {
                    assert!(authority
                        .registry
                        .insert_unchecked(removed_leaf.as_ref().unwrap().clone())
                        .is_none());
                }
                _ => unreachable!(),
            }
        }
        let changed_id = if phase.contains("leaf") {
            ids[2]
        } else {
            ids[1]
        };
        legacy_vec_sort_refresh(&mut legacy, [changed_id, changed_id]);
        candidate.refresh_readiness_many([changed_id, changed_id]);
        assert_equivalent(Scenario::DistinctPair, phase, &legacy, &candidate, ids);
        let after = candidate.readiness.generation();
        if matches!(phase, "runtime failure" | "runtime recovery") {
            assert!(std::sync::Arc::ptr_eq(
                &previous_record,
                &after.row(ids[1]).unwrap().record
            ));
        }
        if phase == "runtime failure" || phase == "leaf removal" {
            assert_eq!(
                after.row(ids[0]).unwrap().recursive_dependency_state,
                ResourceReadinessState::Failed
            );
        } else {
            assert_ne!(
                after.row(ids[0]).unwrap().recursive_dependency_state,
                ResourceReadinessState::Failed
            );
        }
        if phase == "disconnected leaf removal" {
            assert!(std::sync::Arc::ptr_eq(
                before.row(ids[0]).unwrap(),
                after.row(ids[0]).unwrap()
            ));
        }
    }
}

#[test]
fn full_source_closure_shortcut_keeps_removed_roots_and_unmodified_incoming_rows() {
    let make_fixture = || {
        let mut authority = ResourceAuthority::default();
        let leaf = profile_record("res://profile/full-cover-leaf.png", ResourceKind::Texture);
        let middle = profile_record("res://profile/full-cover-middle.glb", ResourceKind::Model)
            .with_dependency_ids(vec![leaf.id]);
        let parent = profile_record("res://profile/full-cover-parent.glb", ResourceKind::Model)
            .with_dependency_ids(vec![middle.id]);
        let ids = [parent.id, middle.id, leaf.id];
        for record in [parent, middle, leaf] {
            insert(&mut authority, record);
        }
        (authority, ids)
    };
    let (mut legacy, ids) = make_fixture();
    let (mut candidate, _) = make_fixture();
    let mut removed_leaf = None;
    for phase in [
        "initial",
        "all revised",
        "removed root plus revised parent",
        "leaf recovery",
        "all removed",
    ] {
        for authority in [&mut legacy, &mut candidate] {
            match phase {
                "all revised" => {
                    for id in ids {
                        let mut record = authority.registry.get(id).unwrap().clone();
                        record.revision += 1;
                        assert!(authority.registry.insert_unchecked(record).is_some());
                    }
                }
                "removed root plus revised parent" => {
                    removed_leaf = Some(authority.registry.remove_by_id(ids[2]).unwrap());
                    let mut record = authority.registry.get(ids[0]).unwrap().clone();
                    record.revision += 1;
                    assert!(authority.registry.insert_unchecked(record).is_some());
                }
                "leaf recovery" => {
                    assert!(authority
                        .registry
                        .insert_unchecked(removed_leaf.as_ref().unwrap().clone())
                        .is_none());
                }
                "all removed" => {
                    for id in ids {
                        assert!(authority.registry.remove_by_id(id).is_some());
                    }
                }
                "initial" => {}
                _ => unreachable!(),
            }
        }
        let updates = match phase {
            "removed root plus revised parent" => vec![ids[2], ids[0], ids[2]],
            "leaf recovery" => vec![ids[2]],
            _ => vec![ids[2], ids[0], ids[1], ids[0]],
        };
        legacy_vec_sort_refresh(&mut legacy, updates.iter().copied());
        candidate.refresh_readiness_many(updates.iter().copied());
        assert_equivalent(Scenario::DistinctPair, phase, &legacy, &candidate, ids);
        let generation = candidate.readiness.generation();
        if phase == "removed root plus revised parent" {
            // Two changed roots and two remaining sources are not full coverage:
            // the removed leaf is absent, and the unmodified middle must be reached.
            assert_eq!(generation.diagnostics().affected_closure_count, 3);
            assert_eq!(
                generation.row(ids[1]).unwrap().direct_dependency_state,
                ResourceReadinessState::Failed
            );
            assert_eq!(
                generation.row(ids[0]).unwrap().recursive_dependency_state,
                ResourceReadinessState::Failed
            );
        } else if phase == "all removed" {
            assert_eq!(generation.diagnostics().row_count, 0);
            assert_eq!(generation.diagnostics().affected_closure_count, 3);
        } else {
            assert_ne!(
                generation.row(ids[0]).unwrap().recursive_dependency_state,
                ResourceReadinessState::Failed
            );
        }
    }
}

#[test]
fn sorted_many_refresh_filters_missing_and_no_op_ids_and_keeps_partial_closure() {
    let make_fixture = || {
        let mut authority = ResourceAuthority::default();
        let leaf = profile_record("res://profile/sorted-leaf.png", ResourceKind::Texture);
        let middle = profile_record("res://profile/sorted-middle.glb", ResourceKind::Model)
            .with_dependency_ids(vec![leaf.id]);
        let parent = profile_record("res://profile/sorted-parent.glb", ResourceKind::Model)
            .with_dependency_ids(vec![middle.id]);
        let ids = [parent.id, middle.id, leaf.id];
        for record in [parent, middle, leaf] {
            insert(&mut authority, record);
        }
        (authority, ids)
    };
    let (mut legacy, ids) = make_fixture();
    let (mut candidate, _) = make_fixture();
    let missing = ResourceId::from_stable_label("runtime51-sorted-missing-source");
    assert!(!ids.contains(&missing));
    let input = [missing, ids[2], ids[0], ids[1], ids[2], missing];
    for phase in [
        "initial",
        "no op",
        "leaf revised",
        "all revised",
        "all removed",
    ] {
        for authority in [&mut legacy, &mut candidate] {
            match phase {
                "leaf revised" | "all revised" => {
                    let revised = if phase == "leaf revised" {
                        &ids[2..]
                    } else {
                        &ids[..]
                    };
                    for id in revised {
                        let mut record = authority.registry.get(*id).unwrap().clone();
                        record.revision += 1;
                        assert!(authority.registry.insert_unchecked(record).is_some());
                    }
                }
                "all removed" => {
                    for id in ids {
                        assert!(authority.registry.remove_by_id(id).is_some());
                    }
                }
                _ => {}
            }
        }
        let before = candidate.readiness.generation();
        legacy_vec_sort_refresh(&mut legacy, input);
        candidate.refresh_readiness_many(input);
        assert_equivalent(Scenario::DistinctPair, phase, &legacy, &candidate, ids);
        let after = candidate.readiness.generation();
        assert!(after.row(missing).is_none());
        if phase == "no op" {
            assert!(std::sync::Arc::ptr_eq(&before, &after));
        } else {
            assert_eq!(after.diagnostics().affected_closure_count, 3);
            assert_eq!(after.diagnostics().changed_row_count, 3);
        }
        if phase == "all removed" {
            assert_eq!(after.diagnostics().row_count, 0);
        } else {
            assert_eq!(after.diagnostics().row_count, 3);
        }
    }
}

#[test]
fn a_panicking_input_iterator_leaves_the_published_generation_unchanged() {
    let mut fixture = fixture(Scenario::DistinctPair);
    let before = fixture.authority.readiness.generation();
    let ids = fixture.update_ids;
    let input = (0..8).map(|index| {
        assert_ne!(index, 7, "input iterator failure before staging");
        ids[index % 2]
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        fixture.authority.refresh_readiness_many(input);
    }));
    assert!(result.is_err());
    assert!(std::sync::Arc::ptr_eq(
        &before,
        &fixture.authority.readiness.generation()
    ));
    assert_eq!(
        fixture
            .authority
            .readiness
            .generation()
            .diagnostics()
            .row_count,
        0
    );
}

#[test]
#[ignore = "manual paired Release profile; run with --release --ignored --nocapture"]
fn exact_two_refresh_release_profile_emits_raw_paired_samples() {
    assert!(
        !cfg!(debug_assertions),
        "this ignored profile is meaningful only in a Release test build"
    );

    // Verify every scenario before collecting any timing samples.
    for scenario in SCENARIOS {
        assert_semantic_parity(scenario);
    }

    println!(
        "environment,os={},arch={},release_build={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        !cfg!(debug_assertions)
    );
    println!(
        "config,scenario_count={},warmup_pairs={},measured_pairs={},unit=nanoseconds",
        SCENARIOS.len(),
        WARMUP_PAIRS,
        MEASURED_PAIRS
    );
    println!(
        "summary,scenario,samples,legacy_p50_ns,legacy_p95_ns,candidate_p50_ns,candidate_p95_ns"
    );
    println!("raw,scenario,pair,legacy_first,legacy_ns,candidate_ns");
    for scenario in SCENARIOS {
        for warmup in 0..WARMUP_PAIRS {
            let _ = timed_pair(scenario, warmup % 2 == 0);
        }

        let mut legacy_samples = Vec::with_capacity(MEASURED_PAIRS);
        let mut candidate_samples = Vec::with_capacity(MEASURED_PAIRS);
        for pair in 0..MEASURED_PAIRS {
            let (legacy_ns, candidate_ns) = timed_pair(scenario, pair % 2 == 0);
            legacy_samples.push(legacy_ns);
            candidate_samples.push(candidate_ns);
        }

        println!(
            "summary,{},{},legacy_p50_ns={},legacy_p95_ns={},candidate_p50_ns={},candidate_p95_ns={}",
            scenario.label(),
            MEASURED_PAIRS,
            percentile(&legacy_samples, 50),
            percentile(&legacy_samples, 95),
            percentile(&candidate_samples, 50),
            percentile(&candidate_samples, 95),
        );
        for (pair, (legacy_ns, candidate_ns)) in legacy_samples
            .into_iter()
            .zip(candidate_samples)
            .enumerate()
        {
            println!(
                "raw,{},{},{},{},{}",
                scenario.label(),
                pair,
                pair % 2 == 0,
                legacy_ns,
                candidate_ns
            );
        }
    }
}

#[path = "readiness_pair_profile/batch_profile.rs"]
mod batch_profile;
