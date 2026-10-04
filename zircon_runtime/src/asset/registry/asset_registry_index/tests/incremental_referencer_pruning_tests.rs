use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use crate::asset::{AssetId, AssetKind, AssetUri, AssetUuid};

use super::{source_locator, AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryIndex};

const PROFILE_MARKER: &str = "RUNTIME206_INCREMENTAL_REFERENCER_PRUNING_BENCH_V1";
const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;
const BATCH_OWNERS: usize = 128;
const MAX_NEW_P95_PERCENT: u128 = 80;

#[test]
fn runtime206_incremental_uuid_pruning_preserves_overlap_shared_self_and_empty_edges() {
    let fixture = corpus(16);
    let mut current = fixture.index.clone();
    let mut legacy = fixture.index.clone();
    let owner = fixture.uuids[0];
    let missing_target = uuid(99);
    for dependencies in [
        vec![fixture.uuids[1], fixture.uuids[2], owner, fixture.uuids[2]],
        vec![fixture.uuids[2], owner],
        vec![fixture.uuids[2], owner],
        Vec::new(),
        vec![missing_target, missing_target],
        Vec::new(),
    ] {
        current.replace_dependencies(owner, dependencies.clone());
        legacy.legacy_replace_dependencies(owner, dependencies);
        assert_eq!(current, legacy);
        assert_relations(&current);
    }
    assert_eq!(
        current.get_referencers_by_uuid(fixture.uuids[2]),
        vec![fixture.uuids[1]]
    );

    let before = current.clone();
    current.replace_dependencies(uuid(100), vec![fixture.uuids[2]]);
    legacy.legacy_replace_dependencies(uuid(100), vec![fixture.uuids[2]]);
    assert_eq!(current, before, "missing UUID owner remains a no-op");
    assert_eq!(current, legacy);
}

#[test]
fn runtime206_incremental_path_pruning_preserves_duplicate_shared_and_missing_owner_edges() {
    let fixture = corpus(16);
    let mut current = fixture.index.clone();
    let mut legacy = fixture.index.clone();
    let owner = fixture.uuids[0];
    for dependencies in [
        vec![
            fixture.paths[1].clone(),
            fixture.paths[2].clone(),
            fixture.paths[0].clone(),
            fixture.paths[2].clone(),
            path(99),
        ],
        vec![fixture.paths[2].clone(), fixture.paths[0].clone()],
        vec![fixture.paths[2].clone(), fixture.paths[0].clone()],
        Vec::new(),
    ] {
        current.replace_dependency_paths(owner, dependencies.clone());
        legacy.legacy_replace_dependency_paths(owner, dependencies);
        assert_eq!(current, legacy);
        assert_relations(&current);
    }
    // Path intent can precede a registry row. Preserve that existing contract independently
    // of replace_dependencies, whose absent owner is a no-op.
    let missing_owner = uuid(100);
    for dependencies in [vec![fixture.paths[2].clone(), path(99)], Vec::new()] {
        current.replace_dependency_paths(missing_owner, dependencies.clone());
        legacy.legacy_replace_dependency_paths(missing_owner, dependencies);
        assert_eq!(current, legacy);
        assert_relations(&current);
    }
    assert_eq!(
        current.referencers_by_path.get(&fixture.paths[2]),
        Some(&HashSet::from([fixture.uuids[1]]))
    );
}

#[test]
fn runtime206_incremental_refresh_matches_legacy_forward_reverse_and_unresolved_diagnostics() {
    let fixture = corpus(16);
    let mut current = fixture.index.clone();
    let mut legacy = fixture.index.clone();
    let owner = fixture.uuids[0];
    let shared_owner = fixture.uuids[1];
    let missing_owner = uuid(100);
    let owners = HashSet::from([owner, shared_owner, missing_owner]);
    let dependencies = vec![
        fixture.paths[2].clone(),
        fixture.paths[0].clone(),
        path(99),
        fixture.paths[2].clone(),
    ];
    current.replace_dependency_paths(owner, dependencies.clone());
    legacy.legacy_replace_dependency_paths(owner, dependencies);
    current.refresh_dependency_owners(&owners);
    legacy.legacy_refresh_dependency_owners(&owners);

    assert_eq!(current, legacy);
    assert_relations(&current);
    assert_eq!(
        current.get_dependencies_by_uuid(owner),
        vec![fixture.uuids[2], owner]
    );
    assert_eq!(
        current.referencers_by_uuid.get(&fixture.uuids[2]),
        Some(&HashSet::from([owner, shared_owner]))
    );
    assert_eq!(
        current.diagnostics(),
        &[AssetRegistryDiagnostic::UnresolvedDependency {
            owner,
            path: path(99),
        }]
    );
}

#[test]
fn runtime206_incremental_pruning_preserves_deferred_bootstrap_cleanup() {
    let fixture = corpus(16);
    assert_relations(&fixture.index);
    let mut current = fixture.index.clone();
    let mut legacy = fixture.index.clone();
    current.replace_dependency_paths_inner(fixture.uuids[0], Vec::new(), false);
    legacy.legacy_replace_dependency_paths_inner(fixture.uuids[0], Vec::new(), false);
    assert_eq!(current, legacy);
    assert!(current
        .referencers_by_path
        .get(&fixture.paths[1])
        .is_some_and(HashSet::is_empty));

    // This is the existing from_entries batch boundary, which still owns the final cleanup.
    current
        .referencers_by_path
        .retain(|_, referencers| !referencers.is_empty());
    legacy
        .referencers_by_path
        .retain(|_, referencers| !referencers.is_empty());
    assert_eq!(current, legacy);
    assert_relations(&current);
}

#[test]
fn runtime206_incremental_source_removal_matches_legacy_public_candidate() {
    let fixture = corpus(16);
    let (current, owners) = fixture.index.prepare_source_removal(&fixture.paths[0]);
    let (legacy, legacy_owners) = fixture
        .index
        .legacy_prepare_source_removal(&fixture.paths[0]);
    assert_eq!(owners, legacy_owners);
    assert_eq!(current, legacy);
    assert_relations(&current);
    assert_eq!(current.len(), 15);
    assert!(current.entry_by_uuid(fixture.uuids[0]).is_none());
    assert!(current
        .get_dependencies_by_uuid(fixture.uuids[15])
        .is_empty());
    assert_eq!(
        current.diagnostics(),
        &[AssetRegistryDiagnostic::UnresolvedDependency {
            owner: fixture.uuids[15],
            path: fixture.paths[0].clone(),
        }]
    );
    assert_relations(&fixture.index);
}

#[test]
#[ignore = "managed Windows Release comparison of sparse and batched registry dependency updates"]
fn runtime206_incremental_referencer_pruning_release_profile() {
    assert!(!cfg!(debug_assertions), "run this profile with --release");
    eprintln!(
        "{PROFILE_MARKER} os={} arch={} crate={} processor={:?} profile=release warmup_pairs={WARMUP_PAIRS} sample_pairs={SAMPLE_PAIRS}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        std::env::var("PROCESSOR_IDENTIFIER").ok()
    );
    for count in [10_000, 100_000] {
        let fixture = corpus(count);
        for lane in [Lane::SparseReplacement, Lane::OwnerRefreshBatch] {
            let owner_count = match lane {
                Lane::SparseReplacement => 1,
                Lane::OwnerRefreshBatch => BATCH_OWNERS,
            };
            let owners = fixture.uuids[..owner_count]
                .iter()
                .copied()
                .collect::<HashSet<_>>();
            let mut current = fixture.index.clone();
            let mut legacy = fixture.index.clone();
            for round in 0..WARMUP_PAIRS {
                measure_pair(round, lane, &fixture, &owners, &mut legacy, &mut current);
            }
            let mut old_ns = Vec::with_capacity(SAMPLE_PAIRS);
            let mut new_ns = Vec::with_capacity(SAMPLE_PAIRS);
            for pair in 0..SAMPLE_PAIRS {
                let (old, new) = measure_pair(
                    WARMUP_PAIRS + pair,
                    lane,
                    &fixture,
                    &owners,
                    &mut legacy,
                    &mut current,
                );
                old_ns.push(old);
                new_ns.push(new);
            }
            let old_p50_ns = percentile(&old_ns, 50);
            let old_p95_ns = percentile(&old_ns, 95);
            let old_p99_ns = percentile(&old_ns, 99);
            let new_p50_ns = percentile(&new_ns, 50);
            let new_p95_ns = percentile(&new_ns, 95);
            let new_p99_ns = percentile(&new_ns, 99);
            eprintln!(
                "{PROFILE_MARKER} lane={lane:?} registry_entries={count} changed_owners={owner_count} pair_order=alternating old_p50_ns={old_p50_ns} old_p95_ns={old_p95_ns} old_p99_ns={old_p99_ns} new_p50_ns={new_p50_ns} new_p95_ns={new_p95_ns} new_p99_ns={new_p99_ns} old_ns={old_ns:?} new_ns={new_ns:?}"
            );
            assert!(old_p95_ns > 0);
            assert!(
                new_p95_ns.saturating_mul(100)
                    <= old_p95_ns.saturating_mul(MAX_NEW_P95_PERCENT),
                "{lane:?}/{count}: new P95 {new_p95_ns} ns exceeds {MAX_NEW_P95_PERCENT}% of old {old_p95_ns} ns"
            );
        }
    }
    // These cases run sequentially so the million-entry case owns only one registry.
    profile_million_entry_sparse_update();
    profile_hundred_thousand_owner_refresh();
}

fn profile_million_entry_sparse_update() {
    let mut fixture = corpus(1_000_000);
    let owners = HashSet::from([fixture.uuids[0]]);
    assert_ring_relations(&fixture, 1, false);
    let old_functions = (
        AssetRegistryIndex::legacy_replace_dependency_paths as ReplacePaths,
        AssetRegistryIndex::legacy_refresh_dependency_owners as RefreshOwners,
    );
    let new_functions = (
        AssetRegistryIndex::replace_dependency_paths as ReplacePaths,
        AssetRegistryIndex::refresh_dependency_owners as RefreshOwners,
    );
    let mut old_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut new_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for round in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
        let (old, new) = if round % 2 == 0 {
            (
                measure_restored_sparse(&mut fixture, &owners, old_functions),
                measure_restored_sparse(&mut fixture, &owners, new_functions),
            )
        } else {
            let new = measure_restored_sparse(&mut fixture, &owners, new_functions);
            let old = measure_restored_sparse(&mut fixture, &owners, old_functions);
            (old, new)
        };
        if round >= WARMUP_PAIRS {
            old_ns.push(old);
            new_ns.push(new);
        }
    }
    let old_p50_ns = percentile(&old_ns, 50);
    let old_p95_ns = percentile(&old_ns, 95);
    let old_p99_ns = percentile(&old_ns, 99);
    let new_p50_ns = percentile(&new_ns, 50);
    let new_p95_ns = percentile(&new_ns, 95);
    let new_p99_ns = percentile(&new_ns, 99);
    eprintln!(
        "{PROFILE_MARKER} lane=SparseReplacement registry_entries=1000000 changed_owners=1 pair_order=alternating restored_start=true old_p50_ns={old_p50_ns} old_p95_ns={old_p95_ns} old_p99_ns={old_p99_ns} new_p50_ns={new_p50_ns} new_p95_ns={new_p95_ns} new_p99_ns={new_p99_ns} old_ns={old_ns:?} new_ns={new_ns:?}"
    );
    assert!(old_p95_ns > 0);
    assert!(
        new_p95_ns.saturating_mul(100) <= old_p95_ns.saturating_mul(MAX_NEW_P95_PERCENT),
        "million-entry sparse update: new P95 {new_p95_ns} ns exceeds {MAX_NEW_P95_PERCENT}% of old {old_p95_ns} ns"
    );
}

fn measure_restored_sparse(
    fixture: &mut Corpus,
    owners: &HashSet<AssetUuid>,
    functions: (ReplacePaths, RefreshOwners),
) -> u128 {
    let owner = fixture.uuids[0];
    let paths = vec![fixture.paths[2].clone()];
    let elapsed = measure_sparse(&mut fixture.index, owner, owners, paths, functions);
    // Both implementations must match every forward and reverse edge of the same
    // expected graph. This avoids a second million-entry snapshot or digest oracle.
    assert_ring_relations(fixture, 2, true);
    fixture
        .index
        .replace_dependency_paths(owner, vec![fixture.paths[1].clone()]);
    fixture.index.refresh_dependency_owners(owners);
    elapsed
}

fn profile_hundred_thousand_owner_refresh() {
    let mut fixture = corpus(100_000);
    let owners = fixture.uuids.iter().copied().collect::<HashSet<_>>();
    let mut new_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for round in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
        let offset = if round % 2 == 0 { 2 } else { 1 };
        // Prepare a real dependency change for every owner outside the refresh timer.
        for (i, owner) in fixture.uuids.iter().copied().enumerate() {
            let dependency = fixture.paths[(i + offset) % fixture.paths.len()].clone();
            fixture
                .index
                .replace_dependency_paths(owner, vec![dependency]);
        }
        let elapsed = measure_refresh(
            &mut fixture.index,
            &owners,
            AssetRegistryIndex::refresh_dependency_owners,
        );
        assert_ring_relations(&fixture, offset, false);
        if round >= WARMUP_PAIRS {
            new_ns.push(elapsed);
        }
    }
    let new_p50_ns = percentile(&new_ns, 50);
    let new_p95_ns = percentile(&new_ns, 95);
    let new_p99_ns = percentile(&new_ns, 99);
    eprintln!(
        "{PROFILE_MARKER} lane=FullOwnerRefresh registry_entries=100000 changed_owners=100000 baseline=not_run_quadratic warmup_samples={WARMUP_PAIRS} sample_count={SAMPLE_PAIRS} product_budget=pending new_p50_ns={new_p50_ns} new_p95_ns={new_p95_ns} new_p99_ns={new_p99_ns} new_ns={new_ns:?}"
    );
}

fn assert_ring_relations(fixture: &Corpus, offset: usize, sparse: bool) {
    let count = fixture.uuids.len();
    let index = &fixture.index;
    let sparse_changed = sparse && offset == 2;
    assert_eq!(index.entries_by_uuid.len(), count);
    assert_eq!(index.dependency_paths_by_uuid.len(), count);
    assert_eq!(
        index.referencers_by_uuid.len(),
        count - usize::from(sparse_changed)
    );
    assert_eq!(
        index.referencers_by_path.len(),
        count - usize::from(sparse_changed)
    );
    assert!(index.diagnostics.is_empty());
    for i in 0..count {
        let owner_offset = if sparse && i != 0 { 1 } else { offset };
        let dependency = (i + owner_offset) % count;
        assert_eq!(
            index
                .entries_by_uuid
                .get(&fixture.uuids[i])
                .unwrap()
                .dependencies(),
            std::slice::from_ref(&fixture.uuids[dependency])
        );
        assert_eq!(
            index
                .dependency_paths_by_uuid
                .get(&fixture.uuids[i])
                .unwrap()
                .as_slice(),
            std::slice::from_ref(&fixture.paths[dependency])
        );
        let uuid_owners = index.referencers_by_uuid.get(&fixture.uuids[i]);
        let path_owners = index.referencers_by_path.get(&fixture.paths[i]);
        if sparse_changed && i == 1 {
            assert!(uuid_owners.is_none());
            assert!(path_owners.is_none());
            continue;
        }
        let predecessor = (i + count - if sparse { 1 } else { offset }) % count;
        let owner_count = if sparse_changed && i == 2 { 2 } else { 1 };
        for actual in [uuid_owners.unwrap(), path_owners.unwrap()] {
            assert_eq!(actual.len(), owner_count);
            assert!(actual.contains(&fixture.uuids[predecessor]));
            if owner_count == 2 {
                assert!(actual.contains(&fixture.uuids[0]));
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Lane {
    SparseReplacement,
    OwnerRefreshBatch,
}

type ReplacePaths = fn(&mut AssetRegistryIndex, AssetUuid, Vec<AssetUri>);
type RefreshOwners = fn(&mut AssetRegistryIndex, &HashSet<AssetUuid>);

fn measure_pair(
    round: usize,
    lane: Lane,
    fixture: &Corpus,
    owners: &HashSet<AssetUuid>,
    legacy: &mut AssetRegistryIndex,
    current: &mut AssetRegistryIndex,
) -> (u128, u128) {
    let offset = if round % 2 == 0 { 2 } else { 1 };
    let (old_ns, new_ns) = match lane {
        Lane::SparseReplacement => {
            let old_paths = vec![fixture.paths[offset].clone()];
            let new_paths = old_paths.clone();
            let old_functions = (
                AssetRegistryIndex::legacy_replace_dependency_paths as ReplacePaths,
                AssetRegistryIndex::legacy_refresh_dependency_owners as RefreshOwners,
            );
            let new_functions = (
                AssetRegistryIndex::replace_dependency_paths as ReplacePaths,
                AssetRegistryIndex::refresh_dependency_owners as RefreshOwners,
            );
            let owner = fixture.uuids[0];
            if round % 2 == 0 {
                (
                    measure_sparse(legacy, owner, owners, old_paths, old_functions),
                    measure_sparse(current, owner, owners, new_paths, new_functions),
                )
            } else {
                let new = measure_sparse(current, owner, owners, new_paths, new_functions);
                let old = measure_sparse(legacy, owner, owners, old_paths, old_functions);
                (old, new)
            }
        }
        Lane::OwnerRefreshBatch => {
            for owner_index in 0..BATCH_OWNERS {
                let owner = fixture.uuids[owner_index];
                let paths = vec![fixture.paths[owner_index + offset].clone()];
                legacy.legacy_replace_dependency_paths(owner, paths.clone());
                current.replace_dependency_paths(owner, paths);
            }
            if round % 2 == 0 {
                (
                    measure_refresh(
                        legacy,
                        owners,
                        AssetRegistryIndex::legacy_refresh_dependency_owners,
                    ),
                    measure_refresh(
                        current,
                        owners,
                        AssetRegistryIndex::refresh_dependency_owners,
                    ),
                )
            } else {
                let new = measure_refresh(
                    current,
                    owners,
                    AssetRegistryIndex::refresh_dependency_owners,
                );
                let old = measure_refresh(
                    legacy,
                    owners,
                    AssetRegistryIndex::legacy_refresh_dependency_owners,
                );
                (old, new)
            }
        }
    };
    assert!(
        current == legacy,
        "complete registry state differs after {lane:?} round {round}"
    );
    assert_relations(current);
    (old_ns, new_ns)
}

fn measure_sparse(
    index: &mut AssetRegistryIndex,
    owner: AssetUuid,
    owners: &HashSet<AssetUuid>,
    paths: Vec<AssetUri>,
    functions: (ReplacePaths, RefreshOwners),
) -> u128 {
    let (replace_paths, refresh_owners) = black_box(functions);
    let index = black_box(index);
    let owner = black_box(owner);
    let owners = black_box(owners);
    let paths = black_box(paths);
    let started = Instant::now();
    replace_paths(index, owner, paths);
    refresh_owners(index, owners);
    started.elapsed().as_nanos()
}

fn measure_refresh(
    index: &mut AssetRegistryIndex,
    owners: &HashSet<AssetUuid>,
    refresh_owners: RefreshOwners,
) -> u128 {
    let refresh_owners = black_box(refresh_owners);
    let index = black_box(index);
    let owners = black_box(owners);
    let started = Instant::now();
    refresh_owners(index, owners);
    started.elapsed().as_nanos()
}

fn assert_relations(index: &AssetRegistryIndex) {
    let mut reverse_uuid = HashMap::<AssetUuid, HashSet<AssetUuid>>::new();
    for entry in index.entries_by_uuid.values() {
        for dependency in entry.dependencies() {
            reverse_uuid
                .entry(*dependency)
                .or_default()
                .insert(entry.uuid());
        }
    }
    let mut reverse_path = HashMap::<AssetUri, HashSet<AssetUuid>>::new();
    for (owner, paths) in &index.dependency_paths_by_uuid {
        for path in paths {
            reverse_path.entry(path.clone()).or_default().insert(*owner);
        }
    }
    assert!(
        index.referencers_by_uuid == reverse_uuid,
        "UUID reverse index differs from complete forward edges"
    );
    assert!(
        index.referencers_by_path == reverse_path,
        "path reverse index differs from complete forward intent"
    );
}

struct Corpus {
    index: AssetRegistryIndex,
    uuids: Vec<AssetUuid>,
    paths: Vec<AssetUri>,
}

fn corpus(count: usize) -> Corpus {
    let uuids = (0..count).map(uuid).collect::<Vec<_>>();
    let paths = (0..count).map(path).collect::<Vec<_>>();
    let entries = (0..count).map(|i| {
        AssetRegistryEntry::new(uuids[i], paths[i].clone(), AssetKind::Data, "source")
            .with_dependencies(vec![uuids[(i + 1) % count]])
    });
    Corpus {
        index: AssetRegistryIndex::from_entries(entries).expect("unique fixture entries"),
        uuids,
        paths,
    }
}

fn uuid(index: usize) -> AssetUuid {
    AssetUuid::from_stable_label(&format!("runtime206-referencer-pruning-{index}"))
}

fn path(index: usize) -> AssetUri {
    AssetUri::parse(&format!("res://runtime206/node-{index:06}.asset")).expect("fixture URI")
}

fn percentile(samples: &[u128], percent: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percent).div_ceil(100).saturating_sub(1)]
}

// Frozen pre-change implementations follow. Only method names and their calls to
// these frozen support functions change; the full original operation bodies remain.

#[rustfmt::skip]
impl AssetRegistryIndex {
    pub(super) fn legacy_remove_source_path(&mut self, path: &AssetUri) {
        let removed = self
            .entry_uuids_by_source
            .remove(&source_locator(path))
            .unwrap_or_default();
        for uuid in removed {
            if let Some(entry) = self.entries_by_uuid.remove(&uuid) {
                self.uuids_by_path.remove(entry.path());
                self.uuids_by_canonical_path.remove(entry.path());
                let type_marker = entry.type_marker();
                let remove_type_bucket =
                    self.uuids_by_type
                        .get_mut(&type_marker)
                        .is_some_and(|uuids| {
                            uuids.remove(&uuid);
                            uuids.is_empty()
                        });
                if remove_type_bucket {
                    self.uuids_by_type.remove(&type_marker);
                }
                for tag in entry.tags() {
                    let remove_tag_bucket = self.uuids_by_tag.get_mut(tag).is_some_and(|uuids| {
                        uuids.remove(&uuid);
                        uuids.is_empty()
                    });
                    if remove_tag_bucket {
                        self.uuids_by_tag.remove(tag);
                    }
                }
                if let Some(package_id) = entry.path().package_id() {
                    let remove_package_bucket = self
                        .uuids_by_package
                        .get_mut(package_id)
                        .is_some_and(|uuids| {
                            uuids.remove(&uuid);
                            uuids.is_empty()
                        });
                    if remove_package_bucket {
                        self.uuids_by_package.remove(package_id);
                    }
                }
                let path_prefix = entry.path().path().to_owned();
                let remove_path_prefix_bucket = self
                    .uuids_by_path_prefix
                    .get_mut(&path_prefix)
                    .is_some_and(|uuids| {
                        uuids.remove(&uuid);
                        uuids.is_empty()
                    });
                if remove_path_prefix_bucket {
                    self.uuids_by_path_prefix.remove(&path_prefix);
                }
                self.uuid_by_asset_id
                    .remove(&AssetId::from_asset_uuid(uuid));
                for dependency in entry.dependencies() {
                    if let Some(referencers) = self.referencers_by_uuid.get_mut(dependency) {
                        referencers.remove(&uuid);
                    }
                }
                self.legacy_replace_dependency_paths(uuid, Vec::new());
            }
        }
        self.referencers_by_uuid
            .retain(|_, referencers| !referencers.is_empty());
    }

    pub(super) fn legacy_replace_dependency_paths(
        &mut self,
        uuid: AssetUuid,
        dependencies: Vec<AssetUri>,
    ) {
        self.legacy_replace_dependency_paths_inner(uuid, dependencies, true);
    }

    fn legacy_replace_dependency_paths_inner(
        &mut self,
        uuid: AssetUuid,
        dependencies: Vec<AssetUri>,
        prune_empty_buckets: bool,
    ) {
        for dependency in self
            .dependency_paths_by_uuid
            .remove(&uuid)
            .unwrap_or_default()
        {
            if let Some(referencers) = self.referencers_by_path.get_mut(&dependency) {
                referencers.remove(&uuid);
            }
        }
        for dependency in &dependencies {
            self.referencers_by_path
                .entry(dependency.clone())
                .or_default()
                .insert(uuid);
        }
        if !dependencies.is_empty() {
            self.dependency_paths_by_uuid.insert(uuid, dependencies);
        }
        if prune_empty_buckets {
            self.referencers_by_path
                .retain(|_, referencers| !referencers.is_empty());
        }
    }

    pub(super) fn legacy_replace_dependencies(&mut self, uuid: AssetUuid, dependencies: Vec<AssetUuid>) {
        let Some(entry) = self.entries_by_uuid.get_mut(&uuid) else {
            return;
        };
        let previous = entry.dependencies().to_vec();
        entry.set_dependencies(dependencies);
        let current = entry.dependencies().to_vec();

        for dependency in previous {
            if let Some(referencers) = self.referencers_by_uuid.get_mut(&dependency) {
                referencers.remove(&uuid);
            }
        }
        for dependency in current {
            self.referencers_by_uuid
                .entry(dependency)
                .or_default()
                .insert(uuid);
        }
        self.referencers_by_uuid
            .retain(|_, referencers| !referencers.is_empty());
    }

    pub(crate) fn legacy_prepare_source_removal(&self, source: &AssetUri) -> (Self, HashSet<AssetUuid>) {
        let source = source_locator(source);
        let removed_paths = self
            .source_entries(&source)
            .into_iter()
            .map(|entry| entry.path().clone())
            .collect::<HashSet<_>>();
        let affected_owners = removed_paths
            .iter()
            .filter_map(|path| self.referencers_by_path.get(path))
            .flatten()
            .copied()
            .collect::<HashSet<_>>();
        let mut candidate = self.clone();
        candidate.legacy_remove_source_path(&source);
        candidate.legacy_refresh_dependency_owners(&affected_owners);
        (candidate, affected_owners)
    }

    pub(super) fn legacy_refresh_dependency_owners(&mut self, owners: &HashSet<AssetUuid>) {
        let mut unresolved = Vec::new();
        let resolved = owners
            .iter()
            .filter(|owner| self.entries_by_uuid.contains_key(owner))
            .map(|owner| {
                let paths = self
                    .dependency_paths_by_uuid
                    .get(owner)
                    .map(Vec::as_slice)
                    .unwrap_or_default();
                let (dependencies, unresolved_paths) =
                    legacy_resolve_unique_dependencies(paths, &self.uuids_by_path);
                unresolved.extend(unresolved_paths.into_iter().map(|path| {
                    AssetRegistryDiagnostic::UnresolvedDependency {
                        owner: *owner,
                        path,
                    }
                }));
                (*owner, dependencies)
            })
            .collect::<Vec<_>>();
        for (owner, dependencies) in resolved {
            self.legacy_replace_dependencies(owner, dependencies);
        }
        self.diagnostics.retain(|diagnostic| {
            !matches!(
                diagnostic,
                AssetRegistryDiagnostic::UnresolvedDependency { owner, .. }
                    if owners.contains(owner)
            )
        });
        self.diagnostics.extend(unresolved);
    }
}

fn legacy_resolve_unique_dependencies(
    paths: &[AssetUri],
    uuids_by_path: &HashMap<AssetUri, AssetUuid>,
) -> (Vec<AssetUuid>, Vec<AssetUri>) {
    let unique_capacity = paths.len().min(uuids_by_path.len());
    let mut dependencies = Vec::with_capacity(unique_capacity);
    let mut seen = HashSet::with_capacity(unique_capacity);
    let mut unresolved = Vec::new();
    for path in paths {
        if let Some(dependency) = uuids_by_path.get(path).copied() {
            if seen.insert(dependency) {
                dependencies.push(dependency);
            }
        } else {
            unresolved.push(path.clone());
        }
    }
    (dependencies, unresolved)
}
