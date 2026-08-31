use super::super::super::data::TemplatePaneNodeData;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use super::super::style_selector::WorkbenchChromeKind as ShellPanelKind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shell_panel_kind(
    node: &TemplatePaneNodeData,
) -> Option<ShellPanelKind> {
    match node.control_id.as_str() {
        "WorkbenchWindowRoot" => Some(ShellPanelKind::WindowRoot),
        "WorkbenchWindowTopToolbar" | "WorkbenchWindowTopToolbarRegion" => {
            Some(ShellPanelKind::TopToolbar)
        }
        "WorkbenchMainBand" | "WorkbenchWindowMainBandRegion" => Some(ShellPanelKind::MainBand),
        "WorkbenchWindowActivityRail" | "WorkbenchMainBandActivityRail" => {
            Some(ShellPanelKind::ActivityRail)
        }
        "WorkbenchSceneTreePanel" | "WorkbenchMainBandSceneTreePanel" => {
            Some(ShellPanelKind::ScenePanel)
        }
        "WorkbenchViewportPanel" | "WorkbenchMainBandViewportPanel" => {
            Some(ShellPanelKind::ViewportPanel)
        }
        "WorkbenchInspectorPanel" | "WorkbenchMainBandInspectorPanel" => {
            Some(ShellPanelKind::InspectorPanel)
        }
        "WorkbenchComponentDrawer" | "WorkbenchWindowComponentDrawerRegion" => {
            Some(ShellPanelKind::ComponentDrawer)
        }
        "WorkbenchComponentDrawerBody" | "WorkbenchComponentDrawerConsoleBody" => {
            Some(ShellPanelKind::DrawerBody)
        }
        "WorkbenchComponentInputs"
        | "WorkbenchComponentSelection"
        | "WorkbenchComponentFeedback"
        | "WorkbenchComponentList" => Some(ShellPanelKind::DrawerColumn),
        "WorkbenchWindowStatusBar" | "WorkbenchWindowStatusBarRegion" => {
            Some(ShellPanelKind::StatusBar)
        }
        "WorkbenchSceneTabs" | "WorkbenchInspectorTabs" | "WorkbenchComponentDrawerTabs" => {
            Some(ShellPanelKind::TabsBand)
        }
        "WorkbenchInspectorTransform" | "WorkbenchInspectorMesh" => {
            Some(ShellPanelKind::InspectorSection)
        }
        _ if is_workbench_content_panel_id(node.control_id.as_str()) => {
            Some(ShellPanelKind::ContentPanel)
        }
        _ => None,
    }
}

fn is_workbench_content_panel_id(control_id: &str) -> bool {
    control_id.starts_with("Workbench")
        && matches!(
            content_panel_suffix(control_id),
            Some("LeftPanel" | "CenterPanel" | "RightPanel")
        )
}

fn content_panel_suffix(control_id: &str) -> Option<&'static str> {
    let stem = control_id.strip_suffix("Panel")?;
    if stem.ends_with("Left") {
        Some("LeftPanel")
    } else if stem.ends_with("Center") {
        Some("CenterPanel")
    } else if stem.ends_with("Right") {
        Some("RightPanel")
    } else {
        None
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::content_panel_suffix;

    const CHECKS_PER_SAMPLE: usize = 1_048_576;
    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn optimization_batch_gd_editor416_content_panel_suffix_preserves_boundaries() {
        assert_eq!(
            content_panel_suffix("WorkbenchSceneLeftPanel"),
            Some("LeftPanel")
        );
        assert_eq!(
            content_panel_suffix("WorkbenchCenterPanel"),
            Some("CenterPanel")
        );
        assert_eq!(
            content_panel_suffix("WorkbenchInspectorRightPanel"),
            Some("RightPanel")
        );
        assert_eq!(content_panel_suffix("WorkbenchRightPane"), None);
        assert_eq!(content_panel_suffix("WorkbenchPanel"), None);
        assert_eq!(content_panel_suffix("WorkbenchRightPanelExtra"), None);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_gd_editor416_content_panel_suffix_benchmark() {
        const INPUT: &str = "WorkbenchInspectorRightPanel";
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
            "EDITOR416_CONTENT_PANEL_SUFFIX_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_full_suffix_checks_per_lookup=3 optimized_shared_panel_checks_per_lookup=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
            INPUT.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(optimized_p95 <= legacy_p95 * 75 / 100);
    }

    fn measure_checks(input: &str, optimized: bool) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            let suffix = if optimized {
                content_panel_suffix(black_box(input))
            } else {
                legacy_content_panel_suffix(black_box(input))
            };
            black_box(suffix);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_content_panel_suffix(control_id: &str) -> Option<&'static str> {
        ["LeftPanel", "CenterPanel", "RightPanel"]
            .into_iter()
            .find(|suffix| control_id.ends_with(suffix))
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
