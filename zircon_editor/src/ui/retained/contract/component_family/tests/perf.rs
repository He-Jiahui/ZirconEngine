use std::hint::black_box;
use std::time::Instant;

use super::{workbench_control_family, TemplateComponentFamily};

const CHECKS_PER_SAMPLE: usize = 262_144;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_gc_editor415_workbench_suffix_classification_preserves_precedence() {
    for control_id in [
        "WorkbenchMiniAction",
        "ExternalIconButton",
        "WorkbenchCheckboxField",
        "WorkbenchRadioField",
        "WorkbenchToggleField",
        "WorkbenchDrawerTabAssets",
        "ExternalSegmentedControl",
        "WorkbenchInputStepsSlider",
        "WorkbenchInputDropdown",
        "WorkbenchFieldName",
        "WorkbenchListRow",
        "WorkbenchSceneVirtualItem42",
        "WorkbenchTableRow",
        "ExternalButton",
        "UnknownControl",
    ] {
        assert_eq!(
            workbench_control_family(control_id),
            legacy_workbench_control_family(control_id),
            "{control_id}"
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_gc_editor415_workbench_suffix_classification_benchmark() {
    const INPUT: &str = "WorkbenchInputStepsSlider";
    for _ in 0..4 {
        black_box(measure_checks(INPUT, false));
        black_box(measure_checks(INPUT, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_checks(INPUT, false));
            optimized_samples.push(measure_checks(INPUT, true));
        } else {
            optimized_samples.push(measure_checks(INPUT, true));
            legacy_samples.push(measure_checks(INPUT, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR415_WORKBENCH_SUFFIX_CLASSIFICATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_workbench_prefix_scans_per_check=12 optimized_workbench_prefix_scans_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=35",
        INPUT.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 65 / 100);
}

fn measure_checks(input: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        let family = if optimized {
            workbench_control_family(black_box(input))
        } else {
            legacy_workbench_control_family(black_box(input))
        };
        black_box(family);
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_workbench_control_family(control_id: &str) -> Option<TemplateComponentFamily> {
    if control_id.starts_with("WorkbenchMini")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchRail")
        || control_id.contains("IconButton")
    {
        Some(TemplateComponentFamily::IconButton)
    } else if control_id.starts_with("WorkbenchCheckbox") {
        Some(TemplateComponentFamily::Checkbox)
    } else if control_id.starts_with("WorkbenchRadio") {
        Some(TemplateComponentFamily::Radio)
    } else if control_id.starts_with("WorkbenchToggle") {
        Some(TemplateComponentFamily::Toggle)
    } else if control_id.starts_with("WorkbenchDrawerTab")
        || control_id.starts_with("WorkbenchLabsTab")
    {
        Some(TemplateComponentFamily::Tab)
    } else if control_id.contains("Segmented") {
        Some(TemplateComponentFamily::SegmentedControl)
    } else if control_id.starts_with("WorkbenchInputSlider")
        || control_id.starts_with("WorkbenchInputRangeSlider")
        || control_id.starts_with("WorkbenchInputStepsSlider")
        || control_id.starts_with("WorkbenchSlider")
    {
        Some(TemplateComponentFamily::Slider)
    } else if control_id == "WorkbenchInputDropdown" || control_id.starts_with("WorkbenchDropdown")
    {
        Some(TemplateComponentFamily::Dropdown)
    } else if control_id.starts_with("WorkbenchInput") || control_id.starts_with("WorkbenchField") {
        Some(TemplateComponentFamily::TextInput)
    } else if control_id.starts_with("WorkbenchList") {
        Some(TemplateComponentFamily::ListRow)
    } else if control_id.starts_with("WorkbenchSceneVirtualItem")
        || (control_id.starts_with("WorkbenchScene") && control_id.ends_with("Item"))
        || control_id.starts_with("WorkbenchEffectAsset")
        || control_id.starts_with("WorkbenchEffectHierarchy")
    {
        Some(TemplateComponentFamily::TreeRow)
    } else if control_id.starts_with("WorkbenchTable")
        || control_id.starts_with("WorkbenchEffectModifier")
    {
        Some(TemplateComponentFamily::TableRow)
    } else if control_id.ends_with("Button") || control_id.contains("Button") {
        Some(TemplateComponentFamily::Button)
    } else {
        None
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
