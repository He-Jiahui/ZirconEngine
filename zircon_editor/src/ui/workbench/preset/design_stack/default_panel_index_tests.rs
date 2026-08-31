use std::hint::black_box;
use std::time::Instant;

use super::{panel_by_default_index, EditorUiDesignStack};
use crate::ui::workbench::preset::panel_preset::FyroxPanelPreset;

const SAMPLE_PAIRS: usize = 31;
const LOOKUPS_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260829ap_editor261_default_panel_index_preserves_reordered_presets() {
    let mut stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();

    assert_eq!(
        stack.panel("editor.asset_metadata").unwrap().title,
        "Asset Metadata"
    );
    let last = stack.panels.len() - 1;
    stack.panels.swap(0, last);
    assert_eq!(
        stack.panel("editor.asset_metadata").unwrap().title,
        "Asset Metadata"
    );
    assert_eq!(stack.panel("editor.scene").unwrap().title, "Scene");
}

#[test]
fn optimization_batch_20260829ap_editor261_panel_lookup_uses_default_index_before_fallback() {
    let source = include_str!("../design_stack.rs");
    let lookup = source
        .split("fn panel_by_default_index")
        .nth(1)
        .expect("default panel lookup")
        .split("fn default_jetbrains_shell_preset")
        .next()
        .expect("default panel lookup body");

    assert!(lookup.contains("default_panel_index(view_id)"));
    assert!(lookup.contains("panels.get(expected_index)"));
    assert!(lookup.contains("or_else(|| panels.iter().find"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ap_editor261_indexed_default_panel_lookups_bench() {
    let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
    let view_id = "editor.asset_metadata";
    assert_eq!(
        panel_by_default_index(&stack.panels, view_id),
        legacy_panel(&stack.panels, view_id)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&stack.panels, view_id, false));
            optimized_samples.push(measure(&stack.panels, view_id, true));
        } else {
            optimized_samples.push(measure(&stack.panels, view_id, true));
            legacy_samples.push(measure(&stack.panels, view_id, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR261_INDEXED_DEFAULT_PANEL_LOOKUPS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
lookups_per_sample={LOOKUPS_PER_SAMPLE} panels=20 legacy_worst_case_comparisons_per_lookup=20 \
optimized_expected_slot_checks_per_lookup=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_panel<'a>(panels: &'a [FyroxPanelPreset], view_id: &str) -> Option<&'a FyroxPanelPreset> {
    panels.iter().find(|panel| panel.view_id == view_id)
}

fn measure(panels: &[FyroxPanelPreset], view_id: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let panel = if optimized {
            panel_by_default_index(black_box(panels), black_box(view_id))
        } else {
            legacy_panel(black_box(panels), black_box(view_id))
        }
        .expect("benchmark panel");
        checksum = checksum.wrapping_add(panel.title.len());
    }
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
