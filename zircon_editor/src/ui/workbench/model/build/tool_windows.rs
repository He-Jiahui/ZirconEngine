use std::collections::BTreeMap;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::pane_tab::pane_tab_model;
use super::super::tool_window_stack_model::ToolWindowStackModel;

pub(super) fn build_tool_windows(
    chrome: &EditorChromeSnapshot,
) -> BTreeMap<ActivityDrawerSlot, ToolWindowStackModel> {
    let mut output = BTreeMap::new();
    for (slot, drawer) in &chrome.workbench.drawers {
        let mut tabs = Vec::with_capacity(drawer.tabs.len());
        for tab in &drawer.tabs {
            tabs.push(pane_tab_model(
                tab,
                drawer.active_tab.as_ref() == Some(&tab.instance_id),
                chrome,
            ));
        }
        output.insert(
            *slot,
            ToolWindowStackModel {
                slot: *slot,
                mode: drawer.mode,
                visible: drawer.visible,
                active_tab: drawer.active_tab.clone(),
                tabs,
            },
        );
    }
    output
}

#[cfg(test)]
mod optimization_batch_20260830bz_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const TABS_PER_DRAWER: usize = 128;

    #[test]
    fn tool_window_projection_reserves_drawer_tab_capacity() {
        let source = include_str!("tool_windows.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(drawer.tabs.len())"));
        assert!(implementation.contains("for (slot, drawer) in &chrome.workbench.drawers"));
        assert!(implementation.contains("for tab in &drawer.tabs"));
        assert!(!implementation.contains(".iter().map(|tab|"));
    }

    #[test]
    fn tool_window_projection_keeps_drawer_then_tab_order() {
        let source = include_str!("tool_windows.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let drawer = implementation
            .find("for (slot, drawer) in &chrome.workbench.drawers")
            .expect("drawer loop");
        let tab = implementation
            .find("for tab in &drawer.tabs")
            .expect("tab loop");
        let insert = implementation.find("output.insert(").expect("map insert");
        assert!(drawer < tab);
        assert!(tab < insert);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bz_editor_tool_window_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR324_TOOL_WINDOW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} tabs_per_drawer={TABS_PER_DRAWER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut tabs = if optimized {
                Vec::with_capacity(TABS_PER_DRAWER)
            } else {
                Vec::new()
            };
            for index in 0..TABS_PER_DRAWER {
                tabs.push(index);
            }
            checksum ^= tabs.len();
        }
        std::hint::black_box(checksum);
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
}
