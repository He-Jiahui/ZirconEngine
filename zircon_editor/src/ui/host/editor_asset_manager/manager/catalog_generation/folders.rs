use std::collections::{BTreeMap, HashMap, HashSet};

use zircon_runtime::asset::AssetUuid;
use zircon_runtime_interface::resource::ResourceScheme;

use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, EditorAssetFolderRecord};

#[derive(Clone, Debug, Default)]
struct FolderBuilder {
    parent_folder_id: Option<String>,
    locator_prefix: String,
    display_name: String,
    child_folder_ids: HashSet<String>,
    direct_asset_uuids: Vec<String>,
    recursive_asset_count: usize,
}

pub(super) fn build_folder_records(
    catalog_by_uuid: &HashMap<AssetUuid, AssetCatalogRecord>,
) -> Vec<EditorAssetFolderRecord> {
    let mut folders = BTreeMap::<String, FolderBuilder>::new();
    folders.insert(
        "res://".to_string(),
        FolderBuilder {
            parent_folder_id: None,
            locator_prefix: "res://".to_string(),
            display_name: "Assets".to_string(),
            ..FolderBuilder::default()
        },
    );

    for record in catalog_by_uuid.values().filter(|record| {
        matches!(
            record.locator.scheme(),
            ResourceScheme::Res | ResourceScheme::Package
        )
    }) {
        let Some((root_id, root_display_name, asset_path)) = folder_root_for_record(record) else {
            continue;
        };
        folders
            .entry(root_id.clone())
            .or_insert_with(|| FolderBuilder {
                parent_folder_id: None,
                locator_prefix: root_id.clone(),
                display_name: root_display_name,
                ..FolderBuilder::default()
            });

        let folder_path = asset_path
            .rsplit_once('/')
            .map(|(folder_path, _)| folder_path)
            .unwrap_or_default();
        if let Some(terminal_folder_id) = terminal_folder_id(&root_id, folder_path) {
            if let Some(folder) = folders.get_mut(&terminal_folder_id) {
                folder
                    .direct_asset_uuids
                    .push(record.asset_uuid.to_string());
                folder.recursive_asset_count += 1;
                continue;
            }
        }
        let mut parent_id = root_id;
        for segment in folder_path.split('/').filter(|segment| !segment.is_empty()) {
            let folder_id = if parent_id == "res://" {
                format!("res://{segment}")
            } else {
                format!("{parent_id}/{segment}")
            };
            folders
                .entry(folder_id.clone())
                .or_insert_with(|| FolderBuilder {
                    parent_folder_id: Some(parent_id.clone()),
                    locator_prefix: folder_id.clone(),
                    display_name: segment.to_string(),
                    ..FolderBuilder::default()
                });
            if let Some(parent) = folders.get_mut(&parent_id) {
                let _ = parent.child_folder_ids.insert(folder_id.clone());
            }
            parent_id = folder_id;
        }
        if let Some(folder) = folders.get_mut(&parent_id) {
            folder
                .direct_asset_uuids
                .push(record.asset_uuid.to_string());
            folder.recursive_asset_count += 1;
        }
    }

    let mut ids_by_depth = folders
        .keys()
        .filter(|folder_id| folder_id.as_str() != "res://")
        .cloned()
        .collect::<Vec<_>>();
    ids_by_depth.sort_by_key(|folder_id| std::cmp::Reverse(folder_id.matches('/').count()));
    for folder_id in ids_by_depth {
        let count = folders
            .get(&folder_id)
            .map(|folder| folder.recursive_asset_count)
            .unwrap_or_default();
        let parent_id = folders
            .get(&folder_id)
            .and_then(|folder| folder.parent_folder_id.clone());
        if let Some(parent_id) = parent_id {
            if let Some(parent) = folders.get_mut(&parent_id) {
                parent.recursive_asset_count += count;
            }
        }
    }

    let folder_names = folders
        .iter()
        .map(|(id, folder)| (id.clone(), folder.display_name.clone()))
        .collect::<HashMap<_, _>>();
    let asset_names = catalog_by_uuid
        .values()
        .map(|record| (record.asset_uuid.to_string(), record.display_name.clone()))
        .collect::<HashMap<_, _>>();
    for folder in folders.values_mut() {
        folder.direct_asset_uuids.sort_by(|left, right| {
            let left_key = asset_names
                .get(left)
                .map(String::as_str)
                .unwrap_or_default();
            let right_key = asset_names
                .get(right)
                .map(String::as_str)
                .unwrap_or_default();
            left_key.cmp(right_key).then(left.cmp(right))
        });
    }

    folders
        .into_iter()
        .map(|(folder_id, folder)| {
            let child_folder_ids = ordered_child_folder_ids(folder.child_folder_ids, &folder_names);
            EditorAssetFolderRecord {
                folder_id,
                parent_folder_id: folder.parent_folder_id,
                locator_prefix: folder.locator_prefix,
                display_name: folder.display_name,
                child_folder_ids,
                direct_asset_uuids: folder.direct_asset_uuids,
                recursive_asset_count: folder.recursive_asset_count,
            }
        })
        .collect()
}

fn ordered_child_folder_ids(
    child_folder_ids: HashSet<String>,
    folder_names: &HashMap<String, String>,
) -> Vec<String> {
    let mut child_folder_ids = child_folder_ids.into_iter().collect::<Vec<_>>();
    child_folder_ids.sort_by(|left, right| {
        folder_names[left]
            .cmp(&folder_names[right])
            .then(left.cmp(right))
    });
    child_folder_ids
}

fn terminal_folder_id(root_id: &str, folder_path: &str) -> Option<String> {
    if folder_path.is_empty() {
        None
    } else if root_id == "res://" {
        Some(format!("res://{folder_path}"))
    } else {
        Some(format!("{root_id}/{folder_path}"))
    }
}

fn folder_root_for_record(record: &AssetCatalogRecord) -> Option<(String, String, &str)> {
    match record.locator.scheme() {
        ResourceScheme::Res => Some((
            "res://".to_string(),
            "Assets".to_string(),
            record.locator.path(),
        )),
        ResourceScheme::Package => {
            let package_id = record.locator.package_id()?;
            let package_path = record.locator.package_path()?;
            Some((
                format!("package://{package_id}"),
                package_id.to_string(),
                package_path,
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod performance_tests {
    use std::collections::{BTreeSet, HashMap, HashSet};
    use std::hint::black_box;
    use std::time::Instant;

    use super::{ordered_child_folder_ids, terminal_folder_id};

    #[test]
    fn folder_sort_borrows_display_names() {
        let source = include_str!("folders.rs");
        let cloned_sort_key = ["asset_names.get(", ").cloned().unwrap_or_default()"].concat();
        let collected_segments = ["asset_path.split('/')", ".collect::<Vec<_>>()"].concat();

        assert!(!source.contains(&cloned_sort_key));
        assert!(!source.contains(&collected_segments));
    }

    #[test]
    fn optimization_wave_20260824j_editor04_folder_child_hash_admission_preserves_order() {
        let folder_names = HashMap::from([
            ("res://zulu".to_string(), "Zulu".to_string()),
            ("res://beta".to_string(), "Alpha".to_string()),
            ("res://alpha".to_string(), "Alpha".to_string()),
        ]);
        let mut child_folder_ids = HashSet::new();
        assert!(child_folder_ids.insert("res://zulu".to_string()));
        assert!(child_folder_ids.insert("res://beta".to_string()));
        assert!(child_folder_ids.insert("res://alpha".to_string()));
        assert!(!child_folder_ids.insert("res://beta".to_string()));

        assert_eq!(
            ordered_child_folder_ids(child_folder_ids, &folder_names),
            vec![
                "res://alpha".to_string(),
                "res://beta".to_string(),
                "res://zulu".to_string(),
            ]
        );
    }

    #[test]
    fn optimization_wave_20260824j_editor04_folder_child_hash_admission_uses_set() {
        const SOURCE: &str = include_str!("folders.rs");
        let production = SOURCE.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("child_folder_ids: HashSet<String>"));
        assert!(production.contains("child_folder_ids.insert(folder_id.clone())"));
        assert!(!production.contains("child_folder_ids.contains(&folder_id)"));
        assert!(!production.contains("child_folder_ids.push(folder_id.clone())"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_wave_20260824j_editor04_folder_child_hash_admission_evidence() {
        const CANDIDATE_COUNT: usize = 4_096;
        const FOLDER_ID_BYTES: usize = 256;
        const LEGACY_LINEAR_COMPARISONS: usize = 8_386_560;
        const SAMPLE_COUNT: usize = 21;
        let suffix = "x".repeat(FOLDER_ID_BYTES - 15);
        let candidates = (0..CANDIDATE_COUNT)
            .map(|index| format!("res://{index:08}-{suffix}"))
            .collect::<Vec<_>>();

        let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
            || legacy_child_folder_ids(black_box(&candidates)),
            || hashed_child_folder_ids(black_box(&candidates)),
        );
        assert_eq!(
            legacy_child_folder_ids(&candidates),
            hashed_child_folder_ids(&candidates)
        );

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT EDITOR04_FOLDER_CHILD_HASH_ADMISSION_BENCH_V1 candidates={CANDIDATE_COUNT} folder_id_bytes={FOLDER_ID_BYTES} samples={SAMPLE_COUNT} sample_order=alternating legacy_linear_comparisons={LEGACY_LINEAR_COMPARISONS} optimized_hash_admissions={CANDIDATE_COUNT} deterministic_admission_reduction_percent=99.9512 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    fn optimization_wave_20260824j_editor04_folder_terminal_cache_preserves_ids() {
        assert_eq!(
            terminal_folder_id("res://", "characters/hero"),
            Some("res://characters/hero".to_string())
        );
        assert_eq!(
            terminal_folder_id("package://com.zircon.demo", "characters/hero"),
            Some("package://com.zircon.demo/characters/hero".to_string())
        );
        assert_eq!(terminal_folder_id("res://", ""), None);
    }

    #[test]
    fn optimization_wave_20260824j_editor04_folder_terminal_cache_uses_existing_folder() {
        const SOURCE: &str = include_str!("folders.rs");
        let production = SOURCE.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("terminal_folder_id(&root_id, folder_path)"));
        assert!(production.contains("folders.get_mut(&terminal_folder_id)"));
        let terminal_hit = production
            .split("if let Some(folder) = folders.get_mut(&terminal_folder_id)")
            .nth(1)
            .and_then(|body| body.split("let mut parent_id = root_id;").next())
            .expect("terminal-folder fast path");
        assert!(terminal_hit.contains("folder.recursive_asset_count += 1;"));
        assert!(terminal_hit.contains("continue;"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_wave_20260824j_editor04_folder_terminal_cache_evidence() {
        const ASSET_COUNT: usize = 4_096;
        const PATH_DEPTH: usize = 8;
        const LEGACY_SEGMENT_PATH_BUILDS: usize = ASSET_COUNT * PATH_DEPTH;
        const OPTIMIZED_PATH_BUILDS: usize = ASSET_COUNT + PATH_DEPTH;
        const SAMPLE_COUNT: usize = 21;
        let folder_path = (0..PATH_DEPTH)
            .map(|index| format!("segment-{index:02}-{}", "x".repeat(52)))
            .collect::<Vec<_>>()
            .join("/");

        let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
            || legacy_folder_path_resolution(black_box(&folder_path), ASSET_COUNT),
            || cached_folder_path_resolution(black_box(&folder_path), ASSET_COUNT),
        );
        assert_eq!(
            legacy_folder_path_resolution(&folder_path, ASSET_COUNT),
            cached_folder_path_resolution(&folder_path, ASSET_COUNT)
        );

        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT EDITOR04_FOLDER_TERMINAL_CACHE_BENCH_V1 assets={ASSET_COUNT} path_depth={PATH_DEPTH} samples={SAMPLE_COUNT} sample_order=alternating legacy_segment_path_builds={LEGACY_SEGMENT_PATH_BUILDS} optimized_path_builds={OPTIMIZED_PATH_BUILDS} deterministic_path_build_reduction_percent=87.4756 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    fn legacy_child_folder_ids(candidates: &[String]) -> Vec<String> {
        let mut child_folder_ids = Vec::with_capacity(candidates.len());
        for folder_id in candidates {
            if !child_folder_ids.contains(folder_id) {
                child_folder_ids.push(folder_id.clone());
            }
        }
        child_folder_ids.sort_unstable();
        child_folder_ids
    }

    fn hashed_child_folder_ids(candidates: &[String]) -> Vec<String> {
        let mut child_folder_ids = candidates.iter().cloned().collect::<HashSet<_>>();
        let mut child_folder_ids = child_folder_ids.drain().collect::<Vec<_>>();
        child_folder_ids.sort_unstable();
        child_folder_ids
    }

    fn legacy_folder_path_resolution(folder_path: &str, asset_count: usize) -> Vec<String> {
        let mut folders = BTreeSet::new();
        for _ in 0..asset_count {
            let mut parent_id = "res://".to_string();
            for segment in folder_path.split('/') {
                let folder_id = if parent_id == "res://" {
                    format!("res://{segment}")
                } else {
                    format!("{parent_id}/{segment}")
                };
                let _ = folders.insert(folder_id.clone());
                parent_id = folder_id;
            }
        }
        folders.into_iter().collect()
    }

    fn cached_folder_path_resolution(folder_path: &str, asset_count: usize) -> Vec<String> {
        let mut folders = BTreeSet::new();
        for _ in 0..asset_count {
            let terminal_folder_id = format!("res://{folder_path}");
            if folders.contains(&terminal_folder_id) {
                continue;
            }
            let mut parent_id = "res://".to_string();
            for segment in folder_path.split('/') {
                let folder_id = if parent_id == "res://" {
                    format!("res://{segment}")
                } else {
                    format!("{parent_id}/{segment}")
                };
                let _ = folders.insert(folder_id.clone());
                parent_id = folder_id;
            }
        }
        folders.into_iter().collect()
    }

    fn benchmark_paired_samples<const SAMPLE_COUNT: usize>(
        mut legacy: impl FnMut() -> Vec<String>,
        mut optimized: impl FnMut() -> Vec<String>,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample_index in 0..SAMPLE_COUNT {
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

    fn benchmark_sample(operation: &mut impl FnMut() -> Vec<String>) -> u128 {
        let started = Instant::now();
        let result = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        black_box(result);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        sorted[(sorted.len() * percentile).div_ceil(100) - 1]
    }
}
