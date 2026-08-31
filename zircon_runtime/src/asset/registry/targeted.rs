use std::collections::{HashMap, HashSet};

use crate::asset::project::{AssetMetaDocument, AssetMetaEntry};
use crate::asset::{AssetId, AssetUri, AssetUuid};

use super::asset_registry_index::source_locator;
use super::rebuild::registry_entries;
use super::{AssetRegistryDiagnostic, AssetRegistryEntry, AssetRegistryError, AssetRegistryIndex};

impl AssetRegistryIndex {
    pub(crate) fn prepare_source_removal(&self, source: &AssetUri) -> (Self, HashSet<AssetUuid>) {
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
        candidate.remove_source_path(&source);
        candidate.refresh_dependency_owners(&affected_owners);
        (candidate, affected_owners)
    }

    pub(crate) fn prepare_source_replacement(
        &self,
        meta: &mut AssetMetaDocument,
    ) -> Result<Self, AssetRegistryError> {
        Ok(self.prepare_source_replacement_generation(meta)?.0)
    }

    pub(crate) fn prepare_source_replacement_generation(
        &self,
        meta: &mut AssetMetaDocument,
    ) -> Result<(Self, HashSet<AssetUuid>), AssetRegistryError> {
        let source = source_locator(&meta.url);
        let identity_diagnostics = self.normalize_source_identities(meta, &source);
        let entries = registry_entries(meta);
        self.preflight_source_paths(&source, &entries)?;

        let mut affected_paths = self
            .entry_uuids_by_source
            .get(&source)
            .into_iter()
            .flatten()
            .filter_map(|uuid| self.entries_by_uuid.get(uuid))
            .map(|entry| entry.path().clone())
            .collect::<HashSet<_>>();
        affected_paths.extend(entries.iter().map(|entry| entry.path().clone()));
        let mut affected_owners = affected_paths
            .iter()
            .filter_map(|path| self.referencers_by_path.get(path))
            .flatten()
            .copied()
            .collect::<HashSet<_>>();

        let mut candidate = self.clone();
        candidate.remove_source_path(&source);
        for entry in entries {
            affected_owners.insert(entry.uuid());
            candidate.insert_checked(entry)?;
        }
        for (uuid, paths) in dependency_paths(meta) {
            candidate.replace_dependency_paths(uuid, paths);
        }
        candidate.refresh_dependency_owners(&affected_owners);
        candidate.diagnostics.extend(identity_diagnostics);
        Ok((candidate, affected_owners))
    }

    pub(crate) fn source_entries(&self, locator: &AssetUri) -> Vec<AssetRegistryEntry> {
        let source = source_locator(locator);
        let uuids = self.entry_uuids_by_source.get(&source);
        let mut entries = Vec::with_capacity(uuids.map_or(0, HashSet::len));
        if let Some(uuids) = uuids {
            for uuid in uuids {
                if let Some(entry) = self.entries_by_uuid.get(uuid) {
                    entries.push(entry.clone());
                }
            }
        }
        entries
    }

    pub(crate) fn retarget_runtime_dependency_paths(
        &mut self,
        changes: impl IntoIterator<Item = (AssetId, Vec<AssetUri>, Vec<AssetUri>)>,
    ) -> HashSet<AssetUuid> {
        let mut owners = HashSet::new();
        for (id, removed, added) in changes {
            let Some(owner) = self.uuid_by_asset_id.get(&id).copied() else {
                continue;
            };
            let mut paths = self
                .dependency_paths_by_uuid
                .get(&owner)
                .cloned()
                .unwrap_or_default();
            for path in removed {
                if let Some(index) = paths.iter().position(|candidate| candidate == &path) {
                    paths.remove(index);
                }
            }
            for path in added {
                paths.push(path);
            }
            self.replace_dependency_paths(owner, paths);
            owners.insert(owner);
        }
        self.refresh_dependency_owners(&owners);
        owners
    }

    fn normalize_source_identities(
        &self,
        meta: &mut AssetMetaDocument,
        source: &AssetUri,
    ) -> Vec<AssetRegistryDiagnostic> {
        let mut owners = HashMap::new();
        let mut diagnostics = Vec::new();
        let original_root = meta.uuid;
        if let Some(first_path) = self.identity_owner(&owners, source, original_root) {
            let replacement = self.unique_uuid(&owners);
            diagnostics.push(AssetRegistryDiagnostic::DuplicateGuidReminted {
                original: original_root,
                first_path,
                path: meta.url.clone(),
                replacement,
            });
            meta.uuid = replacement;
            for entry in &mut meta.entries {
                if entry.url.label().is_none() && entry.uuid == original_root {
                    entry.uuid = replacement;
                }
            }
        }
        owners.insert(meta.uuid, meta.url.clone());

        for entry in &mut meta.entries {
            if entry.url.label().is_none() {
                entry.uuid = meta.uuid;
                continue;
            }
            let Some(first_path) = self.identity_owner(&owners, source, entry.uuid) else {
                owners.insert(entry.uuid, entry.url.clone());
                continue;
            };
            let original = entry.uuid;
            let replacement = self.unique_uuid(&owners);
            diagnostics.push(AssetRegistryDiagnostic::DuplicateGuidReminted {
                original,
                first_path,
                path: entry.url.clone(),
                replacement,
            });
            entry.uuid = replacement;
            owners.insert(replacement, entry.url.clone());
        }
        diagnostics
    }

    fn identity_owner(
        &self,
        prepared: &HashMap<AssetUuid, AssetUri>,
        source: &AssetUri,
        uuid: AssetUuid,
    ) -> Option<AssetUri> {
        prepared.get(&uuid).cloned().or_else(|| {
            self.entries_by_uuid
                .get(&uuid)
                .filter(|entry| source_locator(entry.path()) != *source)
                .map(|entry| entry.path().clone())
        })
    }

    fn unique_uuid(&self, prepared: &HashMap<AssetUuid, AssetUri>) -> AssetUuid {
        loop {
            let candidate = AssetUuid::new();
            if !prepared.contains_key(&candidate) && !self.entries_by_uuid.contains_key(&candidate)
            {
                return candidate;
            }
        }
    }

    fn preflight_source_paths(
        &self,
        source: &AssetUri,
        entries: &[AssetRegistryEntry],
    ) -> Result<(), AssetRegistryError> {
        let mut paths = HashMap::with_capacity(entries.len());
        for entry in entries {
            if let Some(first) = paths.insert(entry.path().clone(), entry.uuid()) {
                return Err(AssetRegistryError::DuplicatePath {
                    path: entry.path().clone(),
                    first,
                    second: entry.uuid(),
                });
            }
            if let Some(existing) = self.entry_by_path(entry.path()) {
                if source_locator(existing.path()) != *source {
                    return Err(AssetRegistryError::DuplicatePath {
                        path: entry.path().clone(),
                        first: existing.uuid(),
                        second: entry.uuid(),
                    });
                }
            }
        }
        Ok(())
    }

    pub(super) fn refresh_dependency_owners(&mut self, owners: &HashSet<AssetUuid>) {
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
                    resolve_unique_dependencies(paths, &self.uuids_by_path);
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
            self.replace_dependencies(owner, dependencies);
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

fn resolve_unique_dependencies(
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

fn dependency_paths(meta: &AssetMetaDocument) -> Vec<(AssetUuid, Vec<AssetUri>)> {
    let has_root_dependencies = !meta.entries.iter().any(|entry| entry.url.label().is_none());
    let mut dependencies =
        Vec::with_capacity(meta.entries.len() + usize::from(has_root_dependencies));
    if has_root_dependencies {
        dependencies.push((meta.uuid, meta.dependencies.clone()));
    }
    dependencies.extend(
        meta.entries
            .iter()
            .map(|entry: &AssetMetaEntry| (entry.uuid, entry.dependencies.clone())),
    );
    dependencies
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::asset::{AssetKind, AssetUri, AssetUuid};

    use super::super::{AssetRegistryEntry, AssetRegistryIndex};
    use super::resolve_unique_dependencies;

    const DEPENDENCY_PATHS: usize = 4_096;
    const UNIQUE_DEPENDENCIES: usize = 256;
    const BENCHMARK_ITERATIONS: usize = 16;
    const SAMPLE_PAIRS: usize = 21;

    #[test]
    fn dependency_owner_refresh_deduplicates_in_first_path_order() {
        let (mut index, owner, mut paths, expected) = dependency_fixture(9, 3);
        let missing = AssetUri::parse("res://dependency/missing").unwrap();
        paths.push(missing.clone());
        index.dependency_paths_by_uuid.insert(owner, paths);

        index.refresh_dependency_owners(&HashSet::from([owner]));

        assert_eq!(index.get_dependencies_by_uuid(owner), expected);
        assert!(index.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            super::super::AssetRegistryDiagnostic::UnresolvedDependency {
                owner: diagnostic_owner,
                path,
            } if *diagnostic_owner == owner && path == &missing
        )));
    }

    #[test]
    #[ignore = "release performance gate; run through the Runtime51 managed validator"]
    fn asset_registry_dependency_owner_refresh_benchmark() {
        let (index, _owner, paths, expected) =
            dependency_fixture(DEPENDENCY_PATHS, UNIQUE_DEPENDENCIES);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            let measure_legacy = || {
                measure_ns(BENCHMARK_ITERATIONS, || {
                    legacy_resolve_unique_dependencies(&paths, &index.uuids_by_path)
                })
            };
            let measure_optimized = || {
                measure_ns(BENCHMARK_ITERATIONS, || {
                    resolve_unique_dependencies(&paths, &index.uuids_by_path).0
                })
            };
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        assert_eq!(
            resolve_unique_dependencies(&paths, &index.uuids_by_path).0,
            expected
        );
        println!(
            "PERF-MVP-556 task=asset_registry_dependency_owner_refresh sample_pairs={} dependency_paths={} unique_dependencies={} iterations={} legacy_samples_ns={} optimized_samples_ns={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={}",
            SAMPLE_PAIRS,
            DEPENDENCY_PATHS,
            UNIQUE_DEPENDENCIES,
            BENCHMARK_ITERATIONS,
            sample_csv(&legacy_samples),
            sample_csv(&optimized_samples),
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
            "optimized P95 {optimized_p95}ns must be at most 75% of legacy P95 {legacy_p95}ns"
        );
    }

    fn dependency_fixture(
        path_count: usize,
        unique_count: usize,
    ) -> (AssetRegistryIndex, AssetUuid, Vec<AssetUri>, Vec<AssetUuid>) {
        assert!(unique_count > 0);
        let owner = AssetUuid::new();
        let dependencies = (0..unique_count)
            .map(|index| {
                let uuid = AssetUuid::new();
                let path = AssetUri::parse(&format!("res://dependency/{index}")).unwrap();
                (uuid, path)
            })
            .collect::<Vec<_>>();
        let entries = std::iter::once(AssetRegistryEntry::new(
            owner,
            AssetUri::parse("res://dependency/owner").unwrap(),
            AssetKind::Data,
            "owner",
        ))
        .chain(dependencies.iter().map(|(uuid, path)| {
            AssetRegistryEntry::new(*uuid, path.clone(), AssetKind::Data, "dependency")
        }));
        let index = AssetRegistryIndex::from_entries(entries).unwrap();
        let paths = (0..path_count)
            .map(|index| dependencies[index % unique_count].1.clone())
            .collect::<Vec<_>>();
        let expected = dependencies
            .iter()
            .map(|(uuid, _)| *uuid)
            .collect::<Vec<_>>();
        (index, owner, paths, expected)
    }

    fn legacy_resolve_unique_dependencies(
        paths: &[AssetUri],
        uuids_by_path: &std::collections::HashMap<AssetUri, AssetUuid>,
    ) -> Vec<AssetUuid> {
        let mut dependencies = Vec::new();
        for path in paths {
            if let Some(dependency) = uuids_by_path.get(path).copied() {
                if !dependencies.contains(&dependency) {
                    dependencies.push(dependency);
                }
            }
        }
        dependencies
    }

    fn measure_ns(iterations: usize, mut resolve: impl FnMut() -> Vec<AssetUuid>) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..iterations {
            checksum ^= black_box(resolve()).len();
        }
        black_box(checksum);
        started.elapsed().as_nanos()
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
#[path = "targeted/optimization_tests.rs"]
mod optimization_tests;
