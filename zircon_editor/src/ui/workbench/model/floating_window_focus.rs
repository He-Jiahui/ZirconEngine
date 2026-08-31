use crate::ui::workbench::view::ViewInstanceId;

use super::document_tab_model::DocumentTabModel;
use super::floating_window_model::FloatingWindowModel;

impl FloatingWindowModel {
    pub(crate) fn focus_target_tab(&self) -> Option<&DocumentTabModel> {
        let focused_view = self.focused_view.as_ref();
        let mut active_tab = None;
        for tab in &self.tabs {
            if focused_view == Some(&tab.instance_id) {
                return Some(tab);
            }
            if active_tab.is_none() && tab.active {
                active_tab = Some(tab);
            }
        }
        active_tab.or_else(|| self.tabs.first())
    }

    pub(crate) fn focus_target_instance(&self) -> Option<&ViewInstanceId> {
        self.focus_target_tab().map(|tab| &tab.instance_id)
    }
}

#[cfg(test)]
mod optimization_batch_20260830cl_editor_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::ui::workbench::autolayout::ShellFrame;
    use crate::ui::workbench::layout::{MainPageId, WorkspaceTarget};
    use crate::ui::workbench::snapshot::ViewContentKind;
    use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const TABS_PER_SAMPLE: usize = 1_024;

    #[test]
    fn focus_target_preserves_focused_active_and_first_priorities() {
        let focused = window(Some("focused"), &[("active", true), ("focused", false)]);
        assert_eq!(focus_target_id(&focused), Some("focused"));

        let active = window(Some("missing"), &[("first", false), ("active", true)]);
        assert_eq!(focus_target_id(&active), Some("active"));

        let first = window(None, &[("first", false), ("second", false)]);
        assert_eq!(focus_target_id(&first), Some("first"));
    }

    #[test]
    fn focus_target_scans_tabs_once() {
        let source = include_str!("floating_window_focus.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("floating window focus implementation");

        assert!(implementation.contains("let mut active_tab = None"));
        assert!(implementation.contains("for tab in &self.tabs"));
        assert!(implementation.contains("active_tab.is_none() && tab.active"));
        assert!(implementation.contains("active_tab.or_else(|| self.tabs.first())"));
        assert!(!implementation.contains("or_else(|| self.tabs.iter().find(|tab| tab.active))"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cl_editor_floating_focus_single_scan_p95() {
        let tabs = (0..TABS_PER_SAMPLE)
            .map(|index| index + 1 == TABS_PER_SAMPLE)
            .collect::<Vec<_>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&tabs, false));
                optimized.push(measure(&tabs, true));
            } else {
                optimized.push(measure(&tabs, true));
                legacy.push(measure(&tabs, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!("EDITOR334_FLOATING_FOCUS_SINGLE_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} tabs_per_sample={TABS_PER_SAMPLE} focused_target=missing active_target=last legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn window(focused: Option<&str>, tabs: &[(&str, bool)]) -> FloatingWindowModel {
        let window_id = MainPageId::new("window:test");
        FloatingWindowModel {
            window_id: window_id.clone(),
            title: "Test".to_string(),
            requested_frame: ShellFrame::default(),
            focused_view: focused.map(ViewInstanceId::new),
            tabs: tabs
                .iter()
                .enumerate()
                .map(|(index, (instance_id, active))| DocumentTabModel {
                    workspace: WorkspaceTarget::FloatingWindow(window_id.clone()),
                    workspace_path: vec![index],
                    instance_id: ViewInstanceId::new(*instance_id),
                    descriptor_id: ViewDescriptorId::new("editor.placeholder"),
                    title: (*instance_id).to_string(),
                    icon_key: "placeholder".to_string(),
                    content_kind: ViewContentKind::Placeholder,
                    active: *active,
                    closeable: true,
                    empty_state: None,
                })
                .collect(),
        }
    }

    fn focus_target_id(window: &FloatingWindowModel) -> Option<&str> {
        window.focus_target_instance().map(|id| id.0.as_str())
    }

    fn measure(tabs: &[bool], use_single_scan: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        let missing = black_box(usize::MAX);
        for _ in 0..1_024 {
            let selected = if use_single_scan {
                let mut active = None;
                for (index, is_active) in black_box(tabs).iter().copied().enumerate() {
                    if index == missing {
                        active = Some(index);
                        break;
                    }
                    if active.is_none() && is_active {
                        active = Some(index);
                    }
                }
                active.or(Some(0))
            } else {
                black_box(tabs)
                    .iter()
                    .enumerate()
                    .position(|(index, _)| index == missing)
                    .or_else(|| tabs.iter().position(|active| *active))
                    .or(Some(0))
            };
            checksum ^= selected.unwrap_or_default();
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
