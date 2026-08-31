use std::cmp::Ordering;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::projects::project_filesystem_path_key;

const DOCS_DIR: &str = "docs";
const LEARN_CATALOG_LIMIT: usize = 128;
const MARKDOWN_EXTENSION: &str = "md";
const SKIPPED_DIRECTORIES: &[&str] = &[".git", "target"];
pub const SELECTED_PROJECT_LEARN_SOURCE: &str = "Selected Project";
pub const SOURCE_ENGINE_LEARN_SOURCE: &str = "Source Engine";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnCatalogEntry {
    pub title: String,
    pub category: String,
    pub source: String,
    pub summary: String,
    pub path: PathBuf,
}

#[derive(Clone)]
struct RankedLearnCatalogEntry {
    root_rank: usize,
    entry: LearnCatalogEntry,
}

pub fn discover_learn_catalog<I>(repo_roots: I) -> Result<Vec<LearnCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    discover_learn_catalog_for_scope(None, repo_roots)
}

pub fn discover_learn_catalog_for_scope<I>(
    selected_project_root: Option<PathBuf>,
    repo_roots: I,
) -> Result<Vec<LearnCatalogEntry>, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut entries = Vec::new();
    let mut visited_roots = HashSet::new();

    if let Some(project_root) = selected_project_root {
        collect_docs_root(
            SELECTED_PROJECT_LEARN_SOURCE,
            &project_root,
            0,
            &mut visited_roots,
            &mut entries,
        )?;
    }

    for (root_rank, repo_root) in repo_roots.into_iter().enumerate() {
        collect_docs_root(
            SOURCE_ENGINE_LEARN_SOURCE,
            &repo_root,
            root_rank,
            &mut visited_roots,
            &mut entries,
        )?;
    }

    retain_top_ranked_entries(&mut entries);
    Ok(entries.into_iter().map(|ranked| ranked.entry).collect())
}

fn collect_docs_root(
    source: &str,
    repo_root: &Path,
    root_rank: usize,
    visited_roots: &mut HashSet<String>,
    entries: &mut Vec<RankedLearnCatalogEntry>,
) -> Result<(), HubError> {
    let docs_root = repo_root.join(DOCS_DIR);
    if !docs_root.is_dir() {
        return Ok(());
    }
    let key = project_filesystem_path_key(&docs_root);
    if !visited_roots.insert(key) {
        return Ok(());
    }
    collect_docs(source, &docs_root, &docs_root, root_rank, entries)
}

fn source_priority(source: &str) -> u8 {
    match source {
        SELECTED_PROJECT_LEARN_SOURCE => 0,
        SOURCE_ENGINE_LEARN_SOURCE => 1,
        _ => 2,
    }
}

fn ranked_learn_order(left: &RankedLearnCatalogEntry, right: &RankedLearnCatalogEntry) -> Ordering {
    source_priority(&left.entry.source)
        .cmp(&source_priority(&right.entry.source))
        .then_with(|| left.root_rank.cmp(&right.root_rank))
        .then_with(|| left.entry.source.cmp(&right.entry.source))
        .then_with(|| left.entry.category.cmp(&right.entry.category))
        .then_with(|| left.entry.title.cmp(&right.entry.title))
        .then_with(|| left.entry.path.cmp(&right.entry.path))
}

fn retain_top_ranked_entries(entries: &mut Vec<RankedLearnCatalogEntry>) {
    if entries.len() > LEARN_CATALOG_LIMIT {
        entries.select_nth_unstable_by(LEARN_CATALOG_LIMIT, ranked_learn_order);
        entries.truncate(LEARN_CATALOG_LIMIT);
    }
    entries.sort_by(ranked_learn_order);
}

fn collect_docs(
    source: &str,
    docs_root: &Path,
    directory: &Path,
    root_rank: usize,
    entries: &mut Vec<RankedLearnCatalogEntry>,
) -> Result<(), HubError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(&entry.file_name().to_string_lossy()) {
                continue;
            }
            collect_docs(source, docs_root, &path, root_rank, entries)?;
        } else if file_type.is_file() && is_markdown_file(&path) {
            entries.push(RankedLearnCatalogEntry {
                root_rank,
                entry: read_learn_doc(source, docs_root, &path)?,
            });
        }
    }
    Ok(())
}

fn read_learn_doc(
    source: &str,
    docs_root: &Path,
    path: &Path,
) -> Result<LearnCatalogEntry, HubError> {
    let text = fs::read_to_string(path)?;
    let title = first_heading(&text).unwrap_or_else(|| fallback_title(path));
    let summary = first_summary_line(&text).unwrap_or_default();
    Ok(LearnCatalogEntry {
        title,
        category: category_from_path(docs_root, path),
        source: source.to_string(),
        summary,
        path: path.to_path_buf(),
    })
}

fn first_heading(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn first_summary_line(text: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut seen_frontmatter_start = false;
    for line in text.lines().map(str::trim) {
        if !seen_frontmatter_start && line == "---" {
            in_frontmatter = true;
            seen_frontmatter_start = true;
            continue;
        }
        if in_frontmatter {
            if line == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("- ")
            || line.starts_with("```")
            || line.ends_with(':')
        {
            continue;
        }
        return Some(line.to_string());
    }
    None
}

fn category_from_path(docs_root: &Path, path: &Path) -> String {
    path.strip_prefix(docs_root)
        .ok()
        .and_then(|relative| relative.components().next())
        .and_then(|component| component.as_os_str().to_str())
        .map(format_category)
        .unwrap_or_else(|| "Documentation".to_string())
}

fn format_category(value: &str) -> String {
    let mut words = value.replace(['_', '-'], " ");
    if words.trim().is_empty() {
        return "Documentation".to_string();
    }
    let mut chars = words.chars();
    if let Some(first) = chars.next() {
        words = first.to_uppercase().collect::<String>() + chars.as_str();
    }
    words
}

fn fallback_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(format_category)
        .unwrap_or_else(|| "Documentation".to_string())
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(MARKDOWN_EXTENSION))
}

fn should_skip_directory(name: &str) -> bool {
    SKIPPED_DIRECTORIES
        .iter()
        .any(|skipped| skipped.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Instant, SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn discover_learn_catalog_reads_markdown_titles_and_summaries() {
        let repo_root = temp_repo_root("learn-catalog");
        let docs_root = repo_root.join(DOCS_DIR).join("zircon_hub");
        fs::create_dir_all(&docs_root).unwrap();
        fs::write(
            docs_root.join("index.md"),
            r#"---
related_code:
  - zircon_hub/src/lib.rs
---

# Zircon Hub

`zircon_hub` is the standalone desktop launcher.
"#,
        )
        .unwrap();

        let entries = discover_learn_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Zircon Hub");
        assert_eq!(entries[0].category, "Zircon hub");
        assert_eq!(entries[0].source, SOURCE_ENGINE_LEARN_SOURCE);
        assert_eq!(
            entries[0].summary,
            "`zircon_hub` is the standalone desktop launcher."
        );
    }

    #[test]
    fn discover_learn_catalog_orders_selected_project_docs_first() {
        let project_root = temp_repo_root("learn-project");
        let repo_root = temp_repo_root("learn-engine");
        fs::create_dir_all(project_root.join(DOCS_DIR).join("guide")).unwrap();
        fs::create_dir_all(repo_root.join(DOCS_DIR).join("engine")).unwrap();
        fs::write(
            project_root.join(DOCS_DIR).join("guide").join("project.md"),
            "# Project Guide\n\nProject-local onboarding.",
        )
        .unwrap();
        fs::write(
            repo_root.join(DOCS_DIR).join("engine").join("engine.md"),
            "# Engine Guide\n\nEngine onboarding.",
        )
        .unwrap();

        let entries =
            discover_learn_catalog_for_scope(Some(project_root.clone()), [repo_root.clone()])
                .unwrap();
        fs::remove_dir_all(project_root).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].title, "Project Guide");
        assert_eq!(entries[0].source, SELECTED_PROJECT_LEARN_SOURCE);
        assert_eq!(entries[1].title, "Engine Guide");
        assert_eq!(entries[1].source, SOURCE_ENGINE_LEARN_SOURCE);
    }

    #[test]
    fn discover_learn_catalog_skips_non_markdown_and_transient_dirs() {
        let repo_root = temp_repo_root("learn-skip");
        fs::create_dir_all(repo_root.join(DOCS_DIR).join("target")).unwrap();
        fs::write(
            repo_root.join(DOCS_DIR).join("target").join("cache.md"),
            "# Cache",
        )
        .unwrap();
        fs::write(repo_root.join(DOCS_DIR).join("notes.txt"), "ignored").unwrap();

        let entries = discover_learn_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert!(entries.is_empty());
    }

    #[test]
    fn discover_learn_catalog_keeps_first_source_engine_root_before_fallback_limit() {
        let preferred_root = temp_repo_root("learn-preferred-engine");
        let fallback_root = temp_repo_root("learn-fallback-engine");
        fs::create_dir_all(preferred_root.join(DOCS_DIR).join("settings")).unwrap();
        fs::write(
            preferred_root
                .join(DOCS_DIR)
                .join("settings")
                .join("source-settings-refresh.md"),
            "# Source Settings Refresh\n\nPreferred docs root.",
        )
        .unwrap();
        let fallback_docs = fallback_root.join(DOCS_DIR).join("aaa");
        fs::create_dir_all(&fallback_docs).unwrap();
        for index in 0..LEARN_CATALOG_LIMIT {
            fs::write(
                fallback_docs.join(format!("aaa-{index:03}.md")),
                format!("# Aaa {index:03}\n\nFallback docs root."),
            )
            .unwrap();
        }

        let entries =
            discover_learn_catalog([preferred_root.clone(), fallback_root.clone()]).unwrap();
        fs::remove_dir_all(preferred_root).unwrap();
        fs::remove_dir_all(fallback_root).unwrap();

        assert!(entries
            .iter()
            .any(|entry| entry.title == "Source Settings Refresh"
                && entry.source == SOURCE_ENGINE_LEARN_SOURCE));
    }

    #[test]
    fn hub06_learn_catalog_retains_sorted_prefix_above_limit() {
        let repo_root = temp_repo_root("learn-sorted-prefix");
        let docs = repo_root.join(DOCS_DIR).join("guide");
        fs::create_dir_all(&docs).unwrap();
        for index in (0..150).rev() {
            fs::write(
                docs.join(format!("guide-{index:03}.md")),
                format!("# Guide {index:03}\n\nDocumentation entry {index:03}."),
            )
            .unwrap();
        }

        let entries = discover_learn_catalog([repo_root.clone()]).unwrap();
        fs::remove_dir_all(repo_root).unwrap();

        assert_eq!(entries.len(), LEARN_CATALOG_LIMIT);
        assert_eq!(entries.first().unwrap().title, "Guide 000");
        assert_eq!(entries.last().unwrap().title, "Guide 127");
    }

    #[test]
    #[ignore = "release-only Learn catalog top-K ranking benchmark"]
    fn hub06_learn_catalog_topk_release_benchmark_evidence() {
        const INPUT_ENTRIES: usize = 100_000;
        const SAMPLE_PAIRS: usize = 21;

        fn benchmark_entries() -> Vec<RankedLearnCatalogEntry> {
            (0..INPUT_ENTRIES)
                .map(|index| {
                    let rank = index.wrapping_mul(7_919) % INPUT_ENTRIES;
                    let title = format!("Guide {rank:06}");
                    let source = if rank % 3 == 0 {
                        SELECTED_PROJECT_LEARN_SOURCE
                    } else {
                        SOURCE_ENGINE_LEARN_SOURCE
                    };
                    let category = match rank % 4 {
                        0 => "Engine",
                        1 => "Editor",
                        2 => "Runtime",
                        _ => "Tooling",
                    };
                    RankedLearnCatalogEntry {
                        root_rank: rank % 31,
                        entry: LearnCatalogEntry {
                            path: PathBuf::from("docs")
                                .join(category.to_ascii_lowercase())
                                .join(format!("guide-{rank:06}.md")),
                            title,
                            category: category.to_string(),
                            source: source.to_string(),
                            summary: format!("Documentation entry {rank:06}."),
                        },
                    }
                })
                .collect()
        }

        fn legacy_full_sort(entries: &mut Vec<RankedLearnCatalogEntry>) {
            entries.sort_by(ranked_learn_order);
            entries.truncate(LEARN_CATALOG_LIMIT);
        }

        fn measure_legacy(source: &[RankedLearnCatalogEntry]) -> u128 {
            let mut entries = source.to_vec();
            let started = Instant::now();
            legacy_full_sort(&mut entries);
            black_box(entries.as_slice());
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(source: &[RankedLearnCatalogEntry]) -> u128 {
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
            "HUB06_LEARN_CATALOG_TOPK_BENCH_V1 input_entries={INPUT_ENTRIES} \
retained_entries={LEARN_CATALOG_LIMIT} sample_pairs={SAMPLE_PAIRS} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p50_ns.saturating_mul(100) <= legacy_p50_ns.saturating_mul(65),
            "partial selection must improve Learn ranking P50 by at least 35%: \
legacy={legacy_p50_ns}ns optimized={optimized_p50_ns}ns"
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(65),
            "partial selection must improve Learn ranking P95 by at least 35%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn temp_repo_root(label: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("zircon-hub-{label}-{now}"));
        fs::create_dir_all(&root).unwrap();
        root
    }
}
