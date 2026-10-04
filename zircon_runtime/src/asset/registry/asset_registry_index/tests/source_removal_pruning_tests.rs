use std::collections::{BTreeSet, HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use crate::asset::{AssetId, AssetKind, AssetUri, AssetUuid};

use super::{source_locator, AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryIndex};

const PROFILE_MARKER: &str = "RUNTIME206_SOURCE_REMOVAL_PRUNING_BENCH_V1";
const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;
const BATCH_SOURCES: usize = 128;
const MAX_LOCAL_P95_PERCENT: u128 = 80;

#[test]
fn runtime206_source_removal_preserves_shared_incoming_and_all_secondary_indexes() {
    let (before, uuids, paths) = mixed_fixture();
    let mut current = before.clone();
    let mut baseline = before.clone();
    // A label identifies the same source: both root and subasset must be removed.
    current.remove_source_path(&paths[1]);
    baseline.source_removal_baseline_remove(&paths[1]);

    assert_eq!(current, baseline);
    assert_removed_rows(&before, &current, &HashSet::from([uuids[0], uuids[1]]));
    assert_eq!(
        current.referencers_by_uuid[&uuids[0]],
        HashSet::from([uuids[4]])
    );
    assert_eq!(
        current.referencers_by_uuid[&uuids[1]],
        HashSet::from([uuids[4]])
    );
    assert_eq!(
        current.referencers_by_uuid[&uuids[2]],
        HashSet::from([uuids[4]])
    );
    assert!(!current.referencers_by_uuid.contains_key(&uuids[3]));
    assert!(!current.referencers_by_uuid.contains_key(&uuid(99)));
    assert!(before.entry_by_uuid(uuids[0]).is_some());
    assert_relations(&before);
}

#[test]
fn runtime206_source_removal_absent_repeated_and_final_sources_are_exact() {
    let (before, _, paths) = mixed_fixture();
    let mut current = before.clone();
    let mut baseline = before.clone();
    let missing = uri("res://missing/source.json#part");
    current.remove_source_path(&missing);
    baseline.source_removal_baseline_remove(&missing);
    assert_eq!(current, before);
    assert_eq!(current, baseline);

    for source in &paths {
        current.remove_source_path(source);
        baseline.source_removal_baseline_remove(source);
        assert_eq!(current, baseline);
        assert_relations(&current);
        let once = current.clone();
        current.remove_source_path(source);
        assert_eq!(current, once, "repeating a source deletion is a no-op");
    }
    assert_eq!(current, AssetRegistryIndex::default());
}

#[test]
fn runtime206_source_removal_candidate_refreshes_real_owners_without_mutating_source() {
    let (before, uuids, paths) = mixed_fixture();
    let saved = before.clone();
    let (current, owners) = before.prepare_source_removal(&paths[1]);
    let (baseline, baseline_owners) = before.source_removal_baseline_prepare(&paths[1]);

    assert_eq!(owners, baseline_owners);
    assert_eq!(owners, HashSet::from([uuids[0], uuids[1], uuids[4]]));
    assert_eq!(current, baseline);
    assert_eq!(before, saved);
    assert_eq!(
        current.entry_by_uuid(uuids[4]).unwrap().dependencies(),
        &[uuids[2]]
    );
    assert!(current.entry_by_uuid(uuids[0]).is_none());
    assert!(current.entry_by_uuid(uuids[1]).is_none());
    assert_eq!(
        current.diagnostics(),
        &[
            AssetRegistryDiagnostic::UnresolvedDependency {
                owner: uuids[4],
                path: paths[0].clone()
            },
            AssetRegistryDiagnostic::UnresolvedDependency {
                owner: uuids[4],
                path: paths[1].clone()
            },
        ]
    );
    assert_relations(&current);
}

#[test]
fn runtime206_source_removal_allows_reinsertion_and_exact_reverse_edge_recovery() {
    let (before, uuids, paths) = mixed_fixture();
    let mut current = before.clone();
    let mut baseline = before.clone();
    current.remove_source_path(&paths[0]);
    baseline.source_removal_baseline_remove(&paths[0]);
    for owner in &uuids[..2] {
        let entry = before.entry_by_uuid(*owner).unwrap().clone();
        let intent = before.dependency_paths_by_uuid[owner].clone();
        current.insert_checked(entry.clone()).unwrap();
        baseline.insert_checked(entry).unwrap();
        current.replace_dependency_paths(*owner, intent.clone());
        baseline.replace_dependency_paths(*owner, intent);
    }
    assert_eq!(current, baseline);
    assert_eq!(current, before);
    assert_relations(&current);
    current.remove_source_path(&paths[1]);
    baseline.source_removal_baseline_remove(&paths[1]);
    assert_eq!(current, baseline);
    assert_removed_rows(&before, &current, &HashSet::from([uuids[0], uuids[1]]));
}

#[test]
#[ignore = "managed Windows Release source-removal and real candidate profiles"]
fn runtime206_source_removal_pruning_release_profile() {
    assert!(!cfg!(debug_assertions), "run this profile with --release");
    eprintln!(
        "{PROFILE_MARKER} os={} arch={} processor={:?} crate={} warmup_pairs={WARMUP_PAIRS} sample_pairs={SAMPLE_PAIRS}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::var("PROCESSOR_IDENTIFIER").ok(),
        env!("CARGO_PKG_VERSION")
    );
    for count in [10_000, 100_000, 1_000_000] {
        // The million-row case reuses one real index instead of retaining three copies.
        let mut fixture = RingFixture::new(count);
        for lane in [RemovalLane::Absent, RemovalLane::Sparse, RemovalLane::Batch] {
            let mut old_ns = Vec::with_capacity(SAMPLE_PAIRS);
            let mut new_ns = Vec::with_capacity(SAMPLE_PAIRS);
            for round in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
                let pair = if round % 2 == 0 {
                    let old = fixture.measure_removal(lane, true);
                    let new = fixture.measure_removal(lane, false);
                    (old, new)
                } else {
                    let new = fixture.measure_removal(lane, false);
                    let old = fixture.measure_removal(lane, true);
                    (old, new)
                };
                if round >= WARMUP_PAIRS {
                    old_ns.push(pair.0);
                    new_ns.push(pair.1);
                }
            }
            report_pair(&format!("{lane:?}"), count, &old_ns, &new_ns);
            assert!(
                percentile(&new_ns, 95).saturating_mul(100)
                    <= percentile(&old_ns, 95).saturating_mul(MAX_LOCAL_P95_PERCENT),
                "source-removal local p95 guard failed: count={count} lane={lane:?}"
            );
        }
        if count <= 100_000 {
            profile_real_candidate(&fixture);
        }
    }
    // Measure the actual scale without repeating the old quadratic 100K x 100K scan.
    let mut fixture = RingFixture::new(100_000);
    let mut samples = Vec::with_capacity(SAMPLE_PAIRS);
    for round in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
        let elapsed = fixture.measure_removal(RemovalLane::All, false);
        if round >= WARMUP_PAIRS {
            samples.push(elapsed);
        }
    }
    eprintln!(
        "{PROFILE_MARKER} lane=all_sources registry_entries=100000 removed_sources=100000 comparison=current_only p50_ns={} p95_ns={} p99_ns={} raw_ns={samples:?} product_gate=pending",
        percentile(&samples, 50), percentile(&samples, 95), percentile(&samples, 99)
    );
}

fn mixed_fixture() -> (AssetRegistryIndex, Vec<AssetUuid>, Vec<AssetUri>) {
    let uuids = (0..5).map(uuid).collect::<Vec<_>>();
    let paths = [
        "package://game/model.mesh",
        "package://game/model.mesh#preview",
        "package://game/shared.json",
        "res://textures/lonely.png",
        "res://scene/owner.json",
    ]
    .map(uri)
    .to_vec();
    let entries = [
        AssetRegistryEntry::new(uuids[0], paths[0].clone(), AssetKind::Material, "root")
            .with_tags(BTreeSet::from(["hero".to_owned(), "root-only".to_owned()]))
            .with_dependencies(vec![uuids[0], uuids[1], uuids[2], uuid(99), uuids[2]]),
        AssetRegistryEntry::new(uuids[1], paths[1].clone(), AssetKind::Material, "preview")
            .with_tags(BTreeSet::from(["hero".to_owned()]))
            .with_dependencies(vec![uuids[2], uuids[3], uuids[0]]),
        AssetRegistryEntry::new(uuids[2], paths[2].clone(), AssetKind::Data, "shared")
            .with_tags(BTreeSet::from(["hero".to_owned()])),
        AssetRegistryEntry::new(uuids[3], paths[3].clone(), AssetKind::Texture, "lonely"),
        AssetRegistryEntry::new(uuids[4], paths[4].clone(), AssetKind::Data, "owner")
            .with_dependencies(vec![uuids[0], uuids[1], uuids[2]]),
    ];
    (
        AssetRegistryIndex::from_entries(entries).unwrap(),
        uuids,
        paths,
    )
}

fn assert_removed_rows(
    before: &AssetRegistryIndex,
    after: &AssetRegistryIndex,
    removed: &HashSet<AssetUuid>,
) {
    let rebuilt = AssetRegistryIndex::from_entries(
        before
            .entries_by_uuid
            .values()
            .filter(|entry| !removed.contains(&entry.uuid()))
            .cloned(),
    )
    .unwrap();
    assert_eq!(after.entries_by_uuid, rebuilt.entries_by_uuid);
    assert_eq!(after.uuids_by_path, rebuilt.uuids_by_path);
    assert_eq!(
        after.uuids_by_canonical_path,
        rebuilt.uuids_by_canonical_path
    );
    assert_eq!(after.uuids_by_type, rebuilt.uuids_by_type);
    assert_eq!(after.uuids_by_tag, rebuilt.uuids_by_tag);
    assert_eq!(after.uuids_by_package, rebuilt.uuids_by_package);
    assert_eq!(after.uuids_by_path_prefix, rebuilt.uuids_by_path_prefix);
    assert_eq!(after.uuid_by_asset_id, rebuilt.uuid_by_asset_id);
    assert_eq!(after.entry_uuids_by_source, rebuilt.entry_uuids_by_source);
    let retained_paths = before
        .dependency_paths_by_uuid
        .iter()
        .filter(|(owner, _)| !removed.contains(*owner))
        .map(|(owner, paths)| (*owner, paths.clone()))
        .collect::<HashMap<_, _>>();
    assert_eq!(after.dependency_paths_by_uuid, retained_paths);
    assert_eq!(after.diagnostics, before.diagnostics);
    assert_relations(after);
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
    assert_eq!(index.referencers_by_uuid, reverse_uuid);
    assert_eq!(index.referencers_by_path, reverse_path);
}

#[derive(Clone, Copy, Debug)]
enum RemovalLane {
    Absent,
    Sparse,
    Batch,
    All,
}

struct RingFixture {
    index: AssetRegistryIndex,
    uuids: Vec<AssetUuid>,
    paths: Vec<AssetUri>,
    absent: AssetUri,
}

impl RingFixture {
    fn new(count: usize) -> Self {
        let uuids = (0..count).map(uuid).collect::<Vec<_>>();
        let paths = (0..count)
            .map(|n| uri(&format!("res://removal/asset-{n:07}.json")))
            .collect::<Vec<_>>();
        let entries = (0..count).map(|n| {
            AssetRegistryEntry::new(uuids[n], paths[n].clone(), AssetKind::Data, "ring")
                .with_dependencies(vec![uuids[(n + 1) % count]])
        });
        let index = AssetRegistryIndex::from_entries(entries).unwrap();
        Self {
            index,
            uuids,
            paths,
            absent: uri("res://removal/absent.json"),
        }
    }

    fn measure_removal(&mut self, lane: RemovalLane, baseline: bool) -> u128 {
        let removed_count = match lane {
            RemovalLane::Absent => 0,
            RemovalLane::Sparse => 1,
            RemovalLane::Batch => BATCH_SOURCES,
            RemovalLane::All => self.uuids.len(),
        };
        let saved = self.uuids[..removed_count]
            .iter()
            .map(|owner| {
                (
                    self.index.entry_by_uuid(*owner).unwrap().clone(),
                    self.index.dependency_paths_by_uuid[owner].clone(),
                )
            })
            .collect::<Vec<_>>();
        let sources = if matches!(lane, RemovalLane::Absent) {
            std::slice::from_ref(&self.absent)
        } else {
            &self.paths[..removed_count]
        };
        let started = Instant::now();
        if baseline {
            for source in sources {
                self.index.source_removal_baseline_remove(black_box(source));
            }
        } else {
            for source in sources {
                self.index.remove_source_path(black_box(source));
            }
        }
        black_box(&self.index);
        let elapsed = started.elapsed().as_nanos();
        self.assert_ring(removed_count);
        for (entry, intent) in saved {
            let owner = entry.uuid();
            self.index.insert_checked(entry).unwrap();
            self.index.replace_dependency_paths(owner, intent);
        }
        self.assert_ring(0);
        elapsed
    }

    fn assert_ring(&self, removed_count: usize) {
        let count = self.uuids.len();
        let retained = count - removed_count;
        assert_eq!(self.index.len(), retained);
        assert_eq!(self.index.dependency_paths_by_uuid.len(), retained);
        assert_eq!(self.index.referencers_by_uuid.len(), retained);
        assert_eq!(self.index.referencers_by_path.len(), retained);
        assert_eq!(self.index.entry_uuids_by_source.len(), retained);
        assert_eq!(self.index.uuids_by_path.len(), retained);
        assert_eq!(self.index.uuids_by_canonical_path.len(), retained);
        assert_eq!(self.index.uuid_by_asset_id.len(), retained);
        assert_eq!(self.index.uuids_by_path_prefix.len(), retained);
        assert_eq!(self.index.uuids_by_type.len(), usize::from(retained != 0));
        assert!(self.index.uuids_by_tag.is_empty());
        assert!(self.index.uuids_by_package.is_empty());
        assert!(self.index.diagnostics.is_empty());
        for n in 0..count {
            let owner = self.uuids[n];
            let entry = self.index.entry_by_uuid(owner);
            if n < removed_count {
                assert!(entry.is_none());
                assert!(self.index.entry_by_path(&self.paths[n]).is_none());
                assert!(!self.index.dependency_paths_by_uuid.contains_key(&owner));
                assert!(!self
                    .index
                    .entry_uuids_by_source
                    .contains_key(&self.paths[n]));
            } else {
                let entry = entry.unwrap();
                let next = (n + 1) % count;
                assert_eq!(entry.path(), &self.paths[n]);
                assert_eq!(entry.dependencies(), &[self.uuids[next]]);
                assert_eq!(entry.type_marker(), AssetKind::Data);
                assert_eq!(entry.source_digest(), "ring");
                assert_eq!(self.index.entry_by_path(&self.paths[n]), Some(entry));
                assert_eq!(
                    self.index.dependency_paths_by_uuid[&owner].as_slice(),
                    std::slice::from_ref(&self.paths[next])
                );
                assert_single_owner(
                    self.index.entry_uuids_by_source.get(&self.paths[n]),
                    Some(owner),
                );
            }
            let previous = (n + count - 1) % count;
            let expected_owner = (previous >= removed_count).then_some(self.uuids[previous]);
            assert_single_owner(self.index.referencers_by_uuid.get(&owner), expected_owner);
            assert_single_owner(
                self.index.referencers_by_path.get(&self.paths[n]),
                expected_owner,
            );
        }
        if retained != 0 {
            let typed = &self.index.uuids_by_type[&AssetKind::Data];
            assert_eq!(typed.len(), retained);
            assert!(self.uuids[removed_count..]
                .iter()
                .all(|owner| typed.contains(owner)));
        }
    }
}

fn assert_single_owner(actual: Option<&HashSet<AssetUuid>>, expected: Option<AssetUuid>) {
    match expected {
        Some(owner) => {
            let actual = actual.unwrap();
            assert_eq!(actual.len(), 1);
            assert!(actual.contains(&owner));
        }
        None => assert!(actual.is_none()),
    }
}

fn profile_real_candidate(fixture: &RingFixture) {
    let mut old_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut new_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for round in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
        let measure = |baseline: bool| {
            let started = Instant::now();
            let result = if baseline {
                fixture
                    .index
                    .source_removal_baseline_prepare(black_box(&fixture.paths[0]))
            } else {
                fixture
                    .index
                    .prepare_source_removal(black_box(&fixture.paths[0]))
            };
            black_box(&result);
            (result, started.elapsed().as_nanos())
        };
        let (old, new) = if round % 2 == 0 {
            let old = measure(true);
            let new = measure(false);
            (old, new)
        } else {
            let new = measure(false);
            let old = measure(true);
            (old, new)
        };
        assert_eq!(old.0, new.0);
        let (candidate, owners) = &new.0;
        let incoming_owner = *fixture.uuids.last().unwrap();
        assert_eq!(owners, &HashSet::from([incoming_owner]));
        assert!(candidate.entry_by_uuid(fixture.uuids[0]).is_none());
        assert!(candidate
            .entry_by_uuid(incoming_owner)
            .unwrap()
            .dependencies()
            .is_empty());
        assert_eq!(
            candidate.diagnostics(),
            &[AssetRegistryDiagnostic::UnresolvedDependency {
                owner: incoming_owner,
                path: fixture.paths[0].clone()
            }]
        );
        assert_relations(candidate);
        if round >= WARMUP_PAIRS {
            old_ns.push(old.1);
            new_ns.push(new.1);
        }
        // The real operation's clone is timed; candidate validation and teardown are not.
    }
    fixture.assert_ring(0);
    report_pair(
        "real_prepare_source_removal",
        fixture.uuids.len(),
        &old_ns,
        &new_ns,
    );
    eprintln!("{PROFILE_MARKER} lane=real_prepare_source_removal product_gate=pending includes_registry_clone=true");
}

fn report_pair(lane: &str, count: usize, old: &[u128], new: &[u128]) {
    eprintln!(
        "{PROFILE_MARKER} lane={lane} registry_entries={count} pair_order=alternating old_p50_ns={} old_p95_ns={} old_p99_ns={} new_p50_ns={} new_p95_ns={} new_p99_ns={} old_raw_ns={old:?} new_raw_ns={new:?}",
        percentile(old, 50), percentile(old, 95), percentile(old, 99),
        percentile(new, 50), percentile(new, 95), percentile(new, 99)
    );
}

fn percentile(samples: &[u128], percent: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percent).div_ceil(100) - 1]
}

fn uuid(index: usize) -> AssetUuid {
    AssetUuid::from_stable_label(&format!("source-removal-{index}"))
}
fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

// Frozen pre-edit implementations; only their test-local names/call change.
impl AssetRegistryIndex {
    pub(super) fn source_removal_baseline_remove(&mut self, path: &AssetUri) {
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
                self.replace_dependency_paths(uuid, Vec::new());
            }
        }
        self.referencers_by_uuid
            .retain(|_, referencers| !referencers.is_empty());
    }

    pub(crate) fn source_removal_baseline_prepare(
        &self,
        source: &AssetUri,
    ) -> (Self, HashSet<AssetUuid>) {
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
        candidate.source_removal_baseline_remove(&source);
        candidate.refresh_dependency_owners(&affected_owners);
        (candidate, affected_owners)
    }
}
