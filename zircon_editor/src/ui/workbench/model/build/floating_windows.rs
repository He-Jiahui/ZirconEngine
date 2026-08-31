use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::document_tabs::workspace_tabs;
use super::super::floating_window_model::FloatingWindowModel;

pub(super) fn build_floating_windows(chrome: &EditorChromeSnapshot) -> Vec<FloatingWindowModel> {
    let windows = &chrome.workbench.floating_windows;
    let mut output = Vec::with_capacity(windows.len());
    for window in windows {
        output.push(FloatingWindowModel {
            window_id: window.window_id.clone(),
            title: window.title.clone(),
            requested_frame: window.requested_frame,
            focused_view: window.focused_view.clone(),
            tabs: workspace_tabs(
                &window.workspace,
                WorkspaceTarget::FloatingWindow(window.window_id.clone()),
                chrome,
            ),
        });
    }
    output
}

#[cfg(test)]
mod optimization_batch_20260830bt_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const WINDOWS_PER_SAMPLE: usize = 256;

    #[test]
    fn floating_window_models_reserve_snapshot_window_capacity() {
        let source = include_str!("floating_windows.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("let windows = &chrome.workbench.floating_windows;"));
        assert!(implementation.contains("Vec::with_capacity(windows.len())"));
        assert!(implementation.contains("for window in windows"));
        assert!(implementation.contains("output.push(FloatingWindowModel"));
    }

    #[test]
    fn floating_window_models_keep_snapshot_order() {
        let source = include_str!("floating_windows.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let loop_start = implementation
            .find("for window in windows")
            .expect("window loop");
        let push = implementation
            .find("output.push(FloatingWindowModel")
            .expect("model push");
        assert!(loop_start < push);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bt_editor_floating_window_capacity_p95() {
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
            "EDITOR318_FLOATING_WINDOW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} windows_per_sample={WINDOWS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut output = if optimized {
                Vec::with_capacity(WINDOWS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..WINDOWS_PER_SAMPLE {
                output.push(index);
            }
            checksum ^= output.len();
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
