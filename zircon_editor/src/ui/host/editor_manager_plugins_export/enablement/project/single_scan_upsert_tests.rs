use std::hint::black_box;
use std::time::Instant;

use super::replace_or_push_selection_at;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ProjectPluginSelection};

const SAMPLE_PAIRS: usize = 31;
const UPDATES_PER_SAMPLE: usize = 200;
const SELECTION_COUNT: usize = 4_096;

#[test]
fn optimization_batch_20260829ar_editor263_indexed_upsert_preserves_first_duplicate_and_append() {
    let mut selections = vec![selection(0), selection(7), selection(7)];
    let mut replacement = selection(7);
    replacement.enabled = false;

    replace_or_push_selection_at(&mut selections, Some(1), replacement);
    assert!(!selections[1].enabled);
    assert!(selections[2].enabled);

    replace_or_push_selection_at(&mut selections, None, selection(11));
    assert_eq!(selections.last().unwrap().id, "plugin.0011");
}

#[test]
fn optimization_batch_20260829ar_editor263_project_plugin_update_reuses_selection_position() {
    let source = include_str!("../project.rs");
    let implementation = source
        .split("fn set_project_plugin_enabled_unpublished")
        .nth(1)
        .expect("project plugin enablement")
        .split("fn replace_or_push_selection_at")
        .next()
        .expect("project plugin enablement body");

    assert!(implementation.contains(".position(|selection| selection.id == plugin_id)"));
    assert!(implementation.contains("replace_or_push_selection_at("));
    assert!(!implementation.contains("manifest.plugins.set_enabled("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ar_editor263_single_scan_project_plugin_upsert_bench() {
    let selections = selections();

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&selections, false));
            optimized_samples.push(measure(&selections, true));
        } else {
            optimized_samples.push(measure(&selections, true));
            legacy_samples.push(measure(&selections, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR263_SINGLE_SCAN_PROJECT_PLUGIN_UPSERT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
updates_per_sample={UPDATES_PER_SAMPLE} selections={SELECTION_COUNT} \
legacy_linear_scans_per_update=2 optimized_linear_scans_per_update=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn selections() -> Vec<ProjectPluginSelection> {
    (0..SELECTION_COUNT).map(selection).collect()
}

fn selection(index: usize) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: format!("plugin.{index:04}"),
        enabled: true,
        required: false,
        target_modes: Vec::new(),
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    }
}

fn legacy_replace(
    selections: &mut [ProjectPluginSelection],
    plugin_id: &str,
    replacement: ProjectPluginSelection,
) {
    if let Some(existing) = selections
        .iter_mut()
        .find(|selection| selection.id == plugin_id)
    {
        *existing = replacement;
    }
}

fn measure(base: &[ProjectPluginSelection], optimized: bool) -> u128 {
    let mut selections = base.to_vec();
    let plugin_id = black_box("plugin.4095");
    let started = Instant::now();
    let mut checksum = 0usize;
    for update in 0..UPDATES_PER_SAMPLE {
        let existing_index = selections
            .iter()
            .position(|selection| selection.id == plugin_id)
            .expect("benchmark plugin selection");
        let mut replacement = selections[existing_index].clone();
        replacement.enabled = update % 2 == 0;
        if optimized {
            replace_or_push_selection_at(&mut selections, Some(existing_index), replacement);
        } else {
            legacy_replace(&mut selections, plugin_id, replacement);
        }
        checksum = checksum.wrapping_add(existing_index);
    }
    black_box(&selections);
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
