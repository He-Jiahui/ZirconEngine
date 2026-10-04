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
        discover_learn_catalog_for_scope(Some(project_root.clone()), [repo_root.clone()]).unwrap();
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

    let entries = discover_learn_catalog([preferred_root.clone(), fallback_root.clone()]).unwrap();
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
