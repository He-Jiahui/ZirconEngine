use std::hint::black_box;
use std::time::Instant;

use crate::learn::SELECTED_PROJECT_LEARN_SOURCE;

use super::*;

const PERF_SAMPLE_PAIRS: usize = 21;
const PERF_ITERATIONS_PER_SAMPLE: usize = 1_000;
const PERF_MATURITY_INPUTS: [&str; 4] = ["stable", "STABLE", "preview", "稳定"];
const PERF_CATEGORY_INPUTS: [&str; 6] = [
    "guide",
    "REFERENCE",
    "editor-workflow",
    "Documentation",
    "local",
    "指南",
];

#[test]
fn hub03_catalog_scope_key_stays_stable_across_localized_display_copy() {
    assert_eq!(catalog_scope_key(PROJECT_ASSET_SOURCE), "project");
    assert_eq!(catalog_scope_key(SELECTED_PROJECT_LEARN_SOURCE), "project");
    assert_eq!(catalog_scope_key(ENGINE_PLUGIN_SCOPE), "engine");
    assert_eq!(catalog_scope_key(SOURCE_ENGINE_LEARN_SOURCE), "engine");
    assert_eq!(catalog_scope_key("项目"), "project");
    assert_eq!(catalog_scope_key("源码引擎"), "engine");
}

#[test]
fn hub03_catalog_scope_key_maps_engine_asset_roots_to_engine_filter_key() {
    assert_eq!(catalog_scope_key("Editor"), "engine");
    assert_eq!(catalog_scope_key("Runtime"), "engine");
}

#[test]
fn hub03_catalog_category_key_stays_stable_across_localized_learn_copy() {
    assert_eq!(catalog_category_key("guide"), "guide");
    assert_eq!(catalog_category_key("Reference"), "reference");
    assert_eq!(catalog_category_key("指南"), "guide");
    assert_eq!(catalog_category_key("参考"), "reference");
}

#[test]
fn hub03_catalog_scope_display_uses_current_language() {
    assert_eq!(
        localized_catalog_scope(SELECTED_PROJECT_ASSET_SOURCE, HubLanguage::Chinese),
        "已选项目"
    );
    assert_eq!(
        localized_catalog_scope(SOURCE_ENGINE_LEARN_SOURCE, HubLanguage::Chinese),
        "源码引擎"
    );
    assert_eq!(
        localized_catalog_scope(PROJECT_PLUGIN_SCOPE, HubLanguage::English),
        "Project"
    );
}

#[test]
fn hub03_asset_detail_punctuation_is_localized_before_react_renders_it() {
    assert_eq!(
        asset_detail("Material", "E:\\Assets\\Hero", HubLanguage::English),
        "Material - E:\\Assets\\Hero"
    );
    assert_eq!(
        asset_detail("材质", "E:\\Assets\\Hero", HubLanguage::Chinese),
        "材质：E:\\Assets\\Hero"
    );
}

#[test]
fn hub03_plugin_maturity_tone_does_not_parse_only_english_display_copy() {
    assert_eq!(plugin_maturity_tone("stable"), "success");
    assert_eq!(plugin_maturity_tone("STABLE"), "success");
    assert_eq!(plugin_maturity_tone("稳定"), "success");
    assert_eq!(plugin_maturity_tone("preview"), "warning");
}

#[test]
#[ignore = "release performance evidence"]
fn hub03_catalog_classifiers_release_benchmark_evidence() {
    let mut legacy_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    black_box(run_legacy_classifiers());
    black_box(run_optimized_classifiers());

    for sample in 0..PERF_SAMPLE_PAIRS {
        let (legacy, optimized) = if sample % 2 == 0 {
            (
                measure_classifiers(run_legacy_classifiers),
                measure_classifiers(run_optimized_classifiers),
            )
        } else {
            let optimized = measure_classifiers(run_optimized_classifiers);
            let legacy = measure_classifiers(run_legacy_classifiers);
            (legacy, optimized)
        };
        legacy_ns.push(legacy);
        optimized_ns.push(optimized);
    }

    let legacy_p50 = percentile(&legacy_ns, 50);
    let legacy_p95 = percentile(&legacy_ns, 95);
    let optimized_p50 = percentile(&optimized_ns, 50);
    let optimized_p95 = percentile(&optimized_ns, 95);
    println!(
        "PERF_RESULT hub03_catalog_classifiers legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} classifier_calls_per_sample=10000 iterations_per_sample={PERF_ITERATIONS_PER_SAMPLE} samples={PERF_SAMPLE_PAIRS} legacy_heap_allocations_per_call=1 optimized_heap_allocations_per_call=0 legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_ns),
        raw(&optimized_ns),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "optimized P95 {optimized_p95}ns must be at most 80% of legacy P95 {legacy_p95}ns"
    );
}

fn run_legacy_classifiers() -> usize {
    let mut checksum = 0;
    for _ in 0..PERF_ITERATIONS_PER_SAMPLE {
        for value in PERF_MATURITY_INPUTS {
            checksum ^= black_box(legacy_plugin_maturity_tone(value).len());
        }
        for value in PERF_CATEGORY_INPUTS {
            checksum ^= black_box(legacy_catalog_category_key(value).len());
        }
    }
    checksum
}

fn run_optimized_classifiers() -> usize {
    let mut checksum = 0;
    for _ in 0..PERF_ITERATIONS_PER_SAMPLE {
        for value in PERF_MATURITY_INPUTS {
            checksum ^= black_box(plugin_maturity_tone(value).len());
        }
        for value in PERF_CATEGORY_INPUTS {
            checksum ^= black_box(catalog_category_key(value).len());
        }
    }
    checksum
}

fn legacy_plugin_maturity_tone(maturity: &str) -> &'static str {
    let normalized = black_box(maturity.trim().to_ascii_lowercase());
    if normalized.contains("stable") || maturity.contains("稳定") {
        "success"
    } else {
        "warning"
    }
}

fn legacy_catalog_category_key(category: &str) -> &'static str {
    let normalized = black_box(category.trim().to_ascii_lowercase());
    if normalized.contains("guide") || category.contains("指南") {
        "guide"
    } else if normalized.contains("reference") || category.contains("参考") {
        "reference"
    } else if normalized.contains("workflow") || category.contains("工作流") {
        "workflow"
    } else if normalized.contains("documentation") || category.contains("文档") {
        "documentation"
    } else {
        "local"
    }
}

fn measure_classifiers(classify: impl FnOnce() -> usize) -> u64 {
    let started = Instant::now();
    black_box(classify());
    started.elapsed().as_nanos() as u64
}

fn percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    ordered[rank]
}

fn raw(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
