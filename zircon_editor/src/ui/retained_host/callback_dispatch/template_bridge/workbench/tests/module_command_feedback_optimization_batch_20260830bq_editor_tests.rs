use std::collections::HashMap;
use std::time::Instant;

use super::WorkbenchFeedbackModule;

const SAMPLE_PAIRS: usize = 17;
const PROBES_PER_SAMPLE: usize = 100_000;
const MODULE_IDS: [&str; 11] = [
    "WorkbenchModuleScene",
    "WorkbenchModuleEffect",
    "WorkbenchModuleAbility",
    "WorkbenchModuleTags",
    "WorkbenchModulePerception",
    "WorkbenchModuleMaterial",
    "WorkbenchModuleBehavior",
    "WorkbenchModuleRender",
    "WorkbenchModuleAssets",
    "WorkbenchModuleVfx",
    "WorkbenchModuleHud",
];

#[test]
fn selected_tab_resolution_preserves_order_and_default() {
    assert_eq!(
        WorkbenchFeedbackModule::from_selected_tab(|id| id == "WorkbenchModuleRender"),
        WorkbenchFeedbackModule::Render
    );
    assert_eq!(
        WorkbenchFeedbackModule::from_selected_tab(|_| false),
        WorkbenchFeedbackModule::Effect
    );
}

#[test]
fn selected_tab_resolution_uses_one_node_lookup_for_both_flags() {
    let source = include_str!("../module_command_feedback.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("fn control_selected_or_checked"));
    assert!(implementation.contains(".nodes"));
    assert!(implementation.contains(".get(&node_id)"));
    assert!(!implementation.contains(
        "self.control_bool(control_id, \"selected\") || self.control_bool(control_id, \"checked\")"
    ));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830bq_editor_workbench_module_lookup_p95() {
    let nodes = MODULE_IDS
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, [index == 7, index == 9]))
        .collect::<HashMap<_, _>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&nodes, false));
            optimized.push(measure(&nodes, true));
        } else {
            optimized.push(measure(&nodes, true));
            legacy.push(measure(&nodes, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "EDITOR315_WORKBENCH_MODULE_LOOKUP_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} probes_per_sample={PROBES_PER_SAMPLE} controls={} legacy_node_lookups_per_probe=2 optimized_node_lookups_per_probe=1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        MODULE_IDS.len(),
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(nodes: &HashMap<&'static str, [bool; 2]>, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut selected_count = 0usize;
    for _ in 0..PROBES_PER_SAMPLE {
        for id in MODULE_IDS {
            let selected = if optimized {
                nodes.get(id).is_some_and(|flags| flags[0] || flags[1])
            } else {
                nodes.get(id).is_some_and(|flags| flags[0])
                    || nodes.get(id).is_some_and(|flags| flags[1])
            };
            selected_count += selected as usize;
        }
    }
    std::hint::black_box(selected_count);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
