use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    select_workbench_icon_button_style, WorkbenchIconButtonContext, WorkbenchIconButtonStyle,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) type IconButtonContext =
    WorkbenchIconButtonContext;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_context(
    node: &TemplatePaneNodeData,
) -> IconButtonContext {
    let control_id = node.control_id.as_str();
    if control_id.starts_with("WorkbenchRail") {
        IconButtonContext::Rail
    } else if is_tab_close_button(control_id) {
        IconButtonContext::Toolbar
    } else if control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchRun")
        || control_id.starts_with("WorkbenchLayout")
        || control_id.starts_with("WorkbenchTheme")
    {
        IconButtonContext::Toolbar
    } else {
        IconButtonContext::Panel
    }
}

fn is_tab_close_button(control_id: &str) -> bool {
    if control_id.ends_with("TabClose") {
        return true;
    }
    match control_id.as_bytes().first() {
        Some(b'D') => {
            control_id.starts_with("DockTabClose") || control_id.starts_with("DocumentTabClose")
        }
        Some(b'P') => control_id.starts_with("PageTabClose"),
        _ => false,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_style(
    node: &TemplatePaneNodeData,
    context: IconButtonContext,
) -> WorkbenchIconButtonStyle {
    select_workbench_icon_button_style(node, context)
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::is_tab_close_button;

    const CHECKS_PER_SAMPLE: usize = 1_048_576;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_ge_editor417_tab_close_suffix_preserves_complete_old_set() {
        for control_id in [
            "DockTabClose",
            "PageTabClose",
            "DocumentTabClose",
            "CustomTabClose",
        ] {
            assert!(is_tab_close_button(control_id), "{control_id}");
        }
        for control_id in [
            "DockTabCloseExtra",
            "PageTabCloseButton",
            "DocumentTabCloseExtra",
        ] {
            assert!(is_tab_close_button(control_id), "{control_id}");
        }
        for control_id in ["", "CustomTabCloseButton", "TabCloser"] {
            assert!(!is_tab_close_button(control_id), "{control_id}");
        }
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_ge_editor417_tab_close_suffix_benchmark() {
        const INPUT: &str = "WorkbenchCustomTabClose";
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
            "EDITOR417_TAB_CLOSE_SUFFIX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_checks_per_lookup=4 optimized_checks_per_lookup=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50",
            INPUT.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 50 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let matched = if optimized {
                is_tab_close_button(black_box(input))
            } else {
                legacy_is_tab_close_button(black_box(input))
            };
            black_box(matched);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_is_tab_close_button(control_id: &str) -> bool {
        control_id.starts_with("DockTabClose")
            || control_id.starts_with("PageTabClose")
            || control_id.starts_with("DocumentTabClose")
            || control_id.ends_with("TabClose")
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
}
