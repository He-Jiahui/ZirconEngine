use super::frame_rect;
use super::pane_projection::{
    blank_pane, find_tab_snapshot, pane_from_tab, pane_from_tab_with_template_v2_data,
};
use super::*;
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::retained_host::tab_drag::{
    floating_window_edge_group_key, floating_window_group_key,
};
use crate::ui::widgets::common::document_tab_data;
use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

pub(crate) fn collect_floating_windows(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
    module_plugins: &ModulePluginsPaneViewData,
    build_export: &BuildExportPaneViewData,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> Vec<FloatingWindowData> {
    let template_v2_data = std::collections::BTreeMap::new();
    collect_floating_windows_with_template_v2_data(
        model,
        chrome,
        geometry,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        module_plugins,
        build_export,
        &template_v2_data,
        floating_window_projection_bundle,
    )
}

pub(crate) fn collect_floating_windows_with_template_v2_data(
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
    module_plugins: &ModulePluginsPaneViewData,
    build_export: &BuildExportPaneViewData,
    template_v2_data: &std::collections::BTreeMap<
        String,
        crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot,
    >,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> Vec<FloatingWindowData> {
    let mut output = Vec::with_capacity(model.floating_windows.len());
    for window in &model.floating_windows {
        output.push(floating_window_data(
            window,
            chrome,
            geometry,
            ui_asset_panes,
            animation_panes,
            runtime_diagnostics,
            module_plugins,
            build_export,
            template_v2_data,
            floating_window_projection_bundle,
        ));
    }
    output
}

fn floating_window_data(
    window: &crate::ui::workbench::model::FloatingWindowModel,
    chrome: &EditorChromeSnapshot,
    _geometry: &WorkbenchShellGeometry,
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: Option<&RuntimeDiagnosticsSnapshot>,
    module_plugins: &ModulePluginsPaneViewData,
    build_export: &BuildExportPaneViewData,
    template_v2_data: &std::collections::BTreeMap<
        String,
        crate::core::editor_extension::EditorUiTemplatePaneDataSnapshot,
    >,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> FloatingWindowData {
    let active_tab = window.focus_target_tab();
    let active_pane = active_tab
        .map(|tab| {
            pane_from_tab_with_template_v2_data(
                &tab.instance_id.0,
                &window.window_id.0,
                &tab.title,
                &tab.icon_key,
                tab.content_kind,
                tab.empty_state.as_ref(),
                find_tab_snapshot(chrome, &tab.instance_id.0),
                chrome,
                ui_asset_panes.get(&tab.instance_id.0),
                animation_panes.get(&tab.instance_id.0),
                runtime_diagnostics,
                module_plugins,
                build_export,
                template_v2_data,
            )
        })
        .unwrap_or_else(blank_pane);
    let frame = floating_window_projection_bundle
        .outer_frame(&window.window_id)
        .unwrap_or_default();

    FloatingWindowData {
        window_id: window.window_id.0.clone().into(),
        title: window.title.clone().into(),
        frame: frame_rect(frame),
        header_nodes: Default::default(),
        header_frame: Default::default(),
        overflow_frame: Default::default(),
        tab_frames: Default::default(),
        target_group: floating_window_group_key(&window.window_id).into(),
        left_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::ui::workbench::layout::DockEdge::Left,
        )
        .into(),
        right_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::ui::workbench::layout::DockEdge::Right,
        )
        .into(),
        top_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::ui::workbench::layout::DockEdge::Top,
        )
        .into(),
        bottom_edge_target_group: floating_window_edge_group_key(
            &window.window_id,
            crate::ui::workbench::layout::DockEdge::Bottom,
        )
        .into(),
        focus_target_id: window
            .focus_target_instance()
            .map(|instance_id| instance_id.0.clone())
            .unwrap_or_default()
            .into(),
        tabs: {
            let mut tab_data = Vec::with_capacity(window.tabs.len());
            for tab in &window.tabs {
                tab_data.push(document_tab_data(tab));
            }
            model_rc(tab_data)
        },
        active_pane,
    }
}

#[cfg(test)]
mod optimization_batch_20260830bv_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const WINDOWS_PER_SAMPLE: usize = 64;
    const TABS_PER_WINDOW: usize = 16;

    #[test]
    fn floating_window_projection_reserves_window_and_tab_capacity() {
        let source = include_str!("floating_windows.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("Vec::with_capacity(model.floating_windows.len())"));
        assert!(implementation.contains("Vec::with_capacity(window.tabs.len())"));
        assert!(implementation.contains("for window in &model.floating_windows"));
        assert!(implementation.contains("for tab in &window.tabs"));
    }

    #[test]
    fn floating_window_projection_keeps_window_before_tab_order() {
        let source = include_str!("floating_windows.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let window_loop = implementation
            .find("for window in &model.floating_windows")
            .expect("window loop");
        let tab_loop = implementation
            .find("for tab in &window.tabs")
            .expect("tab loop");
        assert!(window_loop < tab_loop);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bv_editor_floating_window_projection_capacity_p95() {
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
            "EDITOR320_FLOATING_WINDOW_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} windows_per_sample={WINDOWS_PER_SAMPLE} tabs_per_window={TABS_PER_WINDOW} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut windows = if optimized {
                Vec::with_capacity(WINDOWS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for _ in 0..WINDOWS_PER_SAMPLE {
                let mut tabs = if optimized {
                    Vec::with_capacity(TABS_PER_WINDOW)
                } else {
                    Vec::new()
                };
                for tab in 0..TABS_PER_WINDOW {
                    tabs.push(tab);
                }
                windows.push(tabs.len());
            }
            checksum ^= windows.len();
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
