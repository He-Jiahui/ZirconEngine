use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::projects::project_filesystem_path_key;

const ASSET_CATALOG_LIMIT: usize = 256;
const PROJECT_ASSET_DIRS: &[&str] = &["Assets", "assets"];
pub const SELECTED_PROJECT_ASSET_SOURCE: &str = "Selected Project";
pub const PROJECT_ASSET_SOURCE: &str = "Project";
const ENGINE_ASSET_ROOTS: &[(&str, &[&str])] = &[
    ("Editor", &["zircon_editor", "assets"]),
    ("Runtime", &["zircon_runtime", "assets"]),
];
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetCatalogEntry {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub size_bytes: u64,
    pub path: PathBuf,
}

#[derive(Clone)]
struct RankedAssetCatalogEntry {
    root_rank: usize,
    entry: AssetCatalogEntry,
}

pub fn discover_asset_catalog<P, R>(
    project_roots: P,
    repo_roots: R,
) -> Result<Vec<AssetCatalogEntry>, HubError>
where
    P: IntoIterator<Item = PathBuf>,
    R: IntoIterator<Item = PathBuf>,
{
    discover_asset_catalog_for_scope(None, project_roots, repo_roots)
}

pub fn discover_asset_catalog_for_scope<P, R>(
    selected_project_root: Option<PathBuf>,
    project_roots: P,
    repo_roots: R,
) -> Result<Vec<AssetCatalogEntry>, HubError>
where
    P: IntoIterator<Item = PathBuf>,
    R: IntoIterator<Item = PathBuf>,
{
    let mut entries = Vec::new();
    let mut visited_roots = HashSet::new();

    if let Some(project_root) = selected_project_root {
        collect_project_asset_roots(
            SELECTED_PROJECT_ASSET_SOURCE,
            &project_root,
            0,
            &mut visited_roots,
            &mut entries,
        )?;
    }

    let mut project_root_rank = 0;
    for project_root in project_roots {
        collect_project_asset_roots(
            PROJECT_ASSET_SOURCE,
            &project_root,
            project_root_rank,
            &mut visited_roots,
            &mut entries,
        )?;
        project_root_rank += 1;
    }

    for (root_rank, repo_root) in repo_roots.into_iter().enumerate() {
        for (label, segments) in ENGINE_ASSET_ROOTS {
            let root = segments
                .iter()
                .fold(repo_root.clone(), |path, segment| path.join(segment));
            collect_asset_root(label, &root, root_rank, &mut visited_roots, &mut entries)?;
        }
    }

    retain_top_ranked_entries(&mut entries);
    Ok(entries.into_iter().map(|ranked| ranked.entry).collect())
}

fn collect_project_asset_roots(
    source: &str,
    project_root: &Path,
    root_rank: usize,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<RankedAssetCatalogEntry>,
) -> Result<(), HubError> {
    for asset_dir in PROJECT_ASSET_DIRS {
        let root = project_root.join(asset_dir);
        collect_asset_root(source, &root, root_rank, visited_roots, entries)?;
    }
    Ok(())
}

fn source_priority(source: &str) -> u8 {
    match source {
        SELECTED_PROJECT_ASSET_SOURCE => 0,
        PROJECT_ASSET_SOURCE => 1,
        _ => 2,
    }
}

fn ranked_asset_order(left: &RankedAssetCatalogEntry, right: &RankedAssetCatalogEntry) -> Ordering {
    source_priority(&left.entry.source)
        .cmp(&source_priority(&right.entry.source))
        .then_with(|| left.root_rank.cmp(&right.root_rank))
        .then_with(|| left.entry.source.cmp(&right.entry.source))
        .then_with(|| left.entry.kind.cmp(&right.entry.kind))
        .then_with(|| left.entry.name.cmp(&right.entry.name))
        .then_with(|| left.entry.path.cmp(&right.entry.path))
}

fn retain_top_ranked_entries(entries: &mut Vec<RankedAssetCatalogEntry>) {
    if entries.len() > ASSET_CATALOG_LIMIT {
        entries.select_nth_unstable_by(ASSET_CATALOG_LIMIT, ranked_asset_order);
        entries.truncate(ASSET_CATALOG_LIMIT);
    }
    entries.sort_by(ranked_asset_order);
}

fn collect_asset_root(
    source: &str,
    root: &Path,
    root_rank: usize,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<RankedAssetCatalogEntry>,
) -> Result<(), HubError> {
    if !root.is_dir() {
        return Ok(());
    }
    let root_key = project_filesystem_path_key(root);
    if !visited_roots.insert(root_key) {
        return Ok(());
    }
    collect_asset_files(source, root, root_rank, entries)
}

fn collect_asset_files(
    source: &str,
    directory: &Path,
    root_rank: usize,
    entries: &mut Vec<RankedAssetCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_asset_files(source, &path, root_rank, entries)?;
        } else if file_type.is_file() {
            let metadata = entry.metadata()?;
            entries.push(RankedAssetCatalogEntry {
                root_rank,
                entry: AssetCatalogEntry {
                    name: path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("asset")
                        .to_string(),
                    kind: asset_kind(&path),
                    source: source.to_string(),
                    size_bytes: metadata.len(),
                    path,
                },
            });
        }
    }
    Ok(())
}

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

fn asset_kind(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if file_name.ends_with(".ui.toml") || file_name.ends_with(".v2.ui.toml") {
        return "ui".to_string();
    }
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" | "jpg" | "jpeg" | "webp" | "svg" => "image",
        "glb" | "gltf" | "obj" | "fbx" => "model",
        "wav" | "ogg" | "mp3" | "flac" => "audio",
        "wgsl" | "glsl" | "hlsl" => "shader",
        "toml" | "json" | "ron" => "data",
        "zircon" | "scene" => "scene",
        "" => "file",
        other => other,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn discover_asset_catalog_reads_project_and_engine_assets() {
        let project_root = temp_dir("asset-project");
        let repo_root = temp_dir("asset-repo");
        fs::create_dir_all(project_root.join("Assets").join("textures")).unwrap();
        fs::write(
            project_root
                .join("Assets")
                .join("textures")
                .join("diffuse.png"),
            "image",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join("zircon_editor").join("assets").join("icons")).unwrap();
        fs::write(
            repo_root
                .join("zircon_editor")
                .join("assets")
                .join("icons")
                .join("add.svg"),
            "svg",
        )
        .unwrap();

        let entries = discover_asset_catalog([project_root.clone()], [repo_root.clone()]).unwrap();
        fs::remove_dir_all(project_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert!(entries
            .iter()
            .any(|entry| entry.source == "Project" && entry.kind == "image"));
        assert!(entries
            .iter()
            .any(|entry| entry.source == "Editor" && entry.name == "add.svg"));
    }

    #[test]
    fn discover_asset_catalog_skips_transient_directories() {
        let project_root = temp_dir("asset-skip");
        fs::create_dir_all(project_root.join("Assets").join("target")).unwrap();
        fs::write(
            project_root.join("Assets").join("target").join("cache.png"),
            "cache",
        )
        .unwrap();

        let entries =
            discover_asset_catalog([project_root.clone()], Vec::<PathBuf>::new()).unwrap();
        fs::remove_dir_all(project_root).unwrap();

        assert!(entries.is_empty());
    }

    #[test]
    fn discover_asset_catalog_labels_selected_project_assets() {
        let selected_project_root = temp_dir("asset-selected");
        let other_project_root = temp_dir("asset-other");
        fs::create_dir_all(selected_project_root.join("Assets")).unwrap();
        fs::write(
            selected_project_root.join("Assets").join("hero.glb"),
            "model",
        )
        .unwrap();
        fs::create_dir_all(other_project_root.join("assets")).unwrap();
        fs::write(
            other_project_root.join("assets").join("ambient.ogg"),
            "audio",
        )
        .unwrap();

        let entries = discover_asset_catalog_for_scope(
            Some(selected_project_root.clone()),
            [selected_project_root.clone(), other_project_root.clone()],
            Vec::<PathBuf>::new(),
        )
        .unwrap();
        fs::remove_dir_all(selected_project_root).unwrap();
        fs::remove_dir_all(other_project_root).unwrap();

        assert!(entries.iter().any(|entry| {
            entry.name == "hero.glb" && entry.source == SELECTED_PROJECT_ASSET_SOURCE
        }));
        assert!(entries
            .iter()
            .any(|entry| entry.name == "ambient.ogg" && entry.source == PROJECT_ASSET_SOURCE));
        assert!(!entries
            .iter()
            .any(|entry| entry.name == "hero.glb" && entry.source == PROJECT_ASSET_SOURCE));
    }

    #[test]
    fn discover_asset_catalog_orders_selected_project_assets_first() {
        let selected_project_root = temp_dir("asset-selected-first");
        let other_project_root = temp_dir("asset-other-first");
        let repo_root = temp_dir("asset-repo-first");
        fs::create_dir_all(selected_project_root.join("Assets")).unwrap();
        fs::write(
            selected_project_root.join("Assets").join("hero.glb"),
            "model",
        )
        .unwrap();
        fs::create_dir_all(other_project_root.join("assets")).unwrap();
        fs::write(
            other_project_root.join("assets").join("ambient.ogg"),
            "audio",
        )
        .unwrap();
        fs::create_dir_all(repo_root.join("zircon_runtime").join("assets")).unwrap();
        fs::write(
            repo_root
                .join("zircon_runtime")
                .join("assets")
                .join("runtime.svg"),
            "svg",
        )
        .unwrap();

        let entries = discover_asset_catalog_for_scope(
            Some(selected_project_root.clone()),
            [selected_project_root.clone(), other_project_root.clone()],
            [repo_root.clone()],
        )
        .unwrap();
        fs::remove_dir_all(selected_project_root).unwrap();
        fs::remove_dir_all(other_project_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries[0].source, SELECTED_PROJECT_ASSET_SOURCE);
        assert_eq!(entries[1].source, PROJECT_ASSET_SOURCE);
        assert!(entries[2..]
            .iter()
            .all(|entry| entry.source != SELECTED_PROJECT_ASSET_SOURCE));
    }

    #[test]
    fn discover_asset_catalog_keeps_first_source_engine_root_before_fallback_limit() {
        let preferred_root = temp_dir("asset-preferred-engine");
        let fallback_root = temp_dir("asset-fallback-engine");
        fs::create_dir_all(
            preferred_root
                .join("zircon_editor")
                .join("assets")
                .join("icons"),
        )
        .unwrap();
        fs::write(
            preferred_root
                .join("zircon_editor")
                .join("assets")
                .join("icons")
                .join("source-settings-tool.svg"),
            "svg",
        )
        .unwrap();
        let fallback_assets = fallback_root
            .join("zircon_editor")
            .join("assets")
            .join("icons");
        fs::create_dir_all(&fallback_assets).unwrap();
        for index in 0..ASSET_CATALOG_LIMIT {
            fs::write(fallback_assets.join(format!("aaa-{index:03}.svg")), "svg").unwrap();
        }

        let entries = discover_asset_catalog(
            Vec::<PathBuf>::new(),
            [preferred_root.clone(), fallback_root.clone()],
        )
        .unwrap();
        fs::remove_dir_all(preferred_root).unwrap();
        fs::remove_dir_all(fallback_root).unwrap();

        assert!(entries
            .iter()
            .any(|entry| entry.name == "source-settings-tool.svg" && entry.source == "Editor"));
    }

    #[test]
    fn hub06_asset_catalog_retains_sorted_prefix_above_limit() {
        let project_root = temp_dir("asset-sorted-prefix");
        let assets = project_root.join("Assets");
        fs::create_dir_all(&assets).unwrap();
        for index in (0..300).rev() {
            fs::write(assets.join(format!("asset-{index:03}.png")), "image").unwrap();
        }

        let entries =
            discover_asset_catalog([project_root.clone()], Vec::<PathBuf>::new()).unwrap();
        fs::remove_dir_all(project_root).unwrap();

        assert_eq!(entries.len(), ASSET_CATALOG_LIMIT);
        assert_eq!(entries.first().unwrap().name, "asset-000.png");
        assert_eq!(entries.last().unwrap().name, "asset-255.png");
    }

    #[test]
    #[ignore = "release-only asset catalog top-K ranking benchmark"]
    fn hub06_asset_catalog_topk_release_benchmark_evidence() {
        const INPUT_ENTRIES: usize = 100_000;
        const SAMPLE_PAIRS: usize = 21;

        fn benchmark_entries() -> Vec<RankedAssetCatalogEntry> {
            (0..INPUT_ENTRIES)
                .map(|index| {
                    let rank = index.wrapping_mul(7_919) % INPUT_ENTRIES;
                    let name = format!("asset-{rank:06}.png");
                    let source = match rank % 4 {
                        0 => SELECTED_PROJECT_ASSET_SOURCE,
                        1 => PROJECT_ASSET_SOURCE,
                        2 => "Editor",
                        _ => "Runtime",
                    };
                    let kind = match rank % 3 {
                        0 => "image",
                        1 => "model",
                        _ => "audio",
                    };
                    RankedAssetCatalogEntry {
                        root_rank: rank % 31,
                        entry: AssetCatalogEntry {
                            path: PathBuf::from("Assets").join(&name),
                            name,
                            kind: kind.to_string(),
                            source: source.to_string(),
                            size_bytes: rank as u64,
                        },
                    }
                })
                .collect()
        }

        fn legacy_full_sort(entries: &mut Vec<RankedAssetCatalogEntry>) {
            entries.sort_by(ranked_asset_order);
            entries.truncate(ASSET_CATALOG_LIMIT);
        }

        fn measure_legacy(source: &[RankedAssetCatalogEntry]) -> u128 {
            let mut entries = source.to_vec();
            let started = Instant::now();
            legacy_full_sort(&mut entries);
            black_box(entries.as_slice());
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(source: &[RankedAssetCatalogEntry]) -> u128 {
            let mut entries = source.to_vec();
            let started = Instant::now();
            retain_top_ranked_entries(&mut entries);
            black_box(entries.as_slice());
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let source = benchmark_entries();
        let mut legacy = source.clone();
        legacy_full_sort(&mut legacy);
        let mut optimized = source.clone();
        retain_top_ranked_entries(&mut optimized);
        assert_eq!(
            legacy
                .iter()
                .map(|ranked| &ranked.entry)
                .collect::<Vec<_>>(),
            optimized
                .iter()
                .map(|ranked| &ranked.entry)
                .collect::<Vec<_>>()
        );

        for _ in 0..4 {
            black_box(measure_legacy(&source));
            black_box(measure_optimized(&source));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&source));
                optimized_samples.push(measure_optimized(&source));
            } else {
                optimized_samples.push(measure_optimized(&source));
                legacy_samples.push(measure_legacy(&source));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);

        println!(
            "HUB06_ASSET_CATALOG_TOPK_BENCH_V1 input_entries={INPUT_ENTRIES} \
retained_entries={ASSET_CATALOG_LIMIT} sample_pairs={SAMPLE_PAIRS} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p50_ns.saturating_mul(100) <= legacy_p50_ns.saturating_mul(65),
            "partial selection must improve asset ranking P50 by at least 35%: \
legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns"
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(65),
            "partial selection must improve asset ranking P95 by at least 35%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn temp_dir(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("zircon-hub-{label}-{now}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
