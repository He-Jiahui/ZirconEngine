use super::support::*;
use crate::core::commands::EditorCommandRegistry;
use crate::scene::viewport::GridMode;
use crate::tests::host::retained_callback_dispatch::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, UiSize,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::{
    measure_runtime_text_width, to_host_contract_workbench_window_nodes, TemplatePaneNodeData,
};
use crate::ui::workbench::autolayout::{
    workbench_layout_tier_for_logical_width, WorkbenchChromeMetrics, WorkbenchLayoutTier,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{StatusTaskProgressSnapshot, StatusTaskProgressTone};
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorTypographyTokens,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::tree::UiVisibility;

#[test]
fn componentized_workbench_status_bar_syncs_chrome_and_task_progress() {
    let _guard = env_lock().lock().unwrap();
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.status_line = "Saving project".to_string();
    chrome.console_output = "Saving project".into();
    chrome.scene_viewport_settings.grid_mode = GridMode::VisibleAndSnap;
    chrome.scene_viewport_settings.translate_step = 2.5;
    chrome.status_task_progress = Some(
        StatusTaskProgressSnapshot::new("desktop_export:7", "Export desktop_windows")
            .with_detail("cargo-build - Running generated SourceTemplate Cargo build")
            .with_percent(72)
            .with_tone(StatusTaskProgressTone::Info),
    );

    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge.sync_from_chrome(&chrome).unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchStatusReady", "text").as_deref(),
        Some("Saving project")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusErrors", "text").as_deref(),
        Some("No Errors")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusWarnings", "text").as_deref(),
        Some("0 Warnings")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusMessages", "text").as_deref(),
        Some("1 Message")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusGrid", "text").as_deref(),
        Some("Grid: 2.5 m")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusSnap", "text").as_deref(),
        Some("Snap: On")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusZoom", "text").as_deref(),
        Some("100%")
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchStatusTaskProgress"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusTaskProgress", "text").as_deref(),
        Some("Export desktop_windows 72%")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchStatusTaskBar", "variant").as_deref(),
        Some("linear")
    );
    assert_eq!(
        control_float(&bridge, "WorkbenchStatusTaskBar", "value"),
        Some(72.0)
    );

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let task = template_contract_node(&nodes, "WorkbenchStatusTaskProgress");
    assert_eq!(task.text.as_str(), "Export desktop_windows 72%");
    assert!((task.value_percent - 0.72).abs() < 0.001);
    assert_eq!(
        task.value_text.as_str(),
        "cargo-build - Running generated SourceTemplate Cargo build"
    );
}

#[test]
fn status_task_progress_uses_semantic_tone_without_local_rgb_overrides() {
    let _guard = env_lock().lock().unwrap();
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.status_task_progress = Some(
        StatusTaskProgressSnapshot::new("shader_compile:3", "Compile shaders")
            .with_percent(48)
            .with_tone(StatusTaskProgressTone::Warning),
    );
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    bridge.sync_from_chrome(&chrome).unwrap();

    for control_id in [
        "WorkbenchStatusTaskProgress",
        "WorkbenchStatusTaskLabel",
        "WorkbenchStatusTaskBar",
    ] {
        assert_eq!(
            control_string(&bridge, control_id, "text_tone").as_deref(),
            Some("warning"),
            "{control_id} should preserve semantic task tone"
        );
    }
    for control_id in ["WorkbenchStatusTaskProgress", "WorkbenchStatusTaskBar"] {
        assert_eq!(
            control_string(&bridge, control_id, "track_fill_color"),
            None,
            "{control_id} should resolve its track from the shared palette"
        );
        assert_eq!(
            control_string(&bridge, control_id, "value_color"),
            None,
            "{control_id} should resolve its fill from the semantic tone"
        );
    }
}

#[test]
fn status_signals_project_semantic_variants_without_local_color_or_spacing_overrides() {
    let _guard = env_lock().lock().unwrap();
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    for (control_id, expected_tone) in [
        ("WorkbenchStatusReady", "primary"),
        ("WorkbenchStatusErrors", "muted"),
        ("WorkbenchStatusWarnings", "muted"),
        ("WorkbenchStatusMessages", "muted"),
    ] {
        assert_eq!(
            control_string(&bridge, control_id, "component_variant").as_deref(),
            Some("semantic_status_signal"),
            "{control_id} should use the centralized signal palette"
        );
        assert_eq!(
            control_string(&bridge, control_id, "text_tone").as_deref(),
            Some(expected_tone),
            "{control_id} should preserve its semantic text role"
        );
        for local_override in [
            "icon_fill",
            "icon_size",
            "layout_gap",
            "layout_offset_x",
            "layout_offset_y",
            "text_color",
        ] {
            assert_eq!(
                control_string(&bridge, control_id, local_override),
                None,
                "{control_id} should not project a local `{local_override}` override"
            );
        }
    }
}

#[test]
fn componentized_workbench_status_bar_collapses_task_slot_when_idle() {
    let _guard = env_lock().lock().unwrap();
    let chrome = default_preview_fixture().build_chrome();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    bridge.sync_from_chrome(&chrome).unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchStatusMessages", "text").as_deref(),
        Some("0 Messages")
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchStatusTaskProgress"),
        Some(UiVisibility::Collapsed)
    );
    let ready = bridge
        .control_frame("WorkbenchStatusReady")
        .expect("ready status should stay visible");
    let errors = bridge
        .control_frame("WorkbenchStatusErrors")
        .expect("error status should stay visible");
    let warnings = bridge
        .control_frame("WorkbenchStatusWarnings")
        .expect("warning status should stay visible");
    let messages = bridge
        .control_frame("WorkbenchStatusMessages")
        .expect("message status should stay visible");
    assert!(
        ready.width > 72.0,
        "wide status bars should give the primary summary the remaining width: {ready:?}"
    );
    assert_compact_status_item_frame(errors, 92.0);
    assert_compact_status_item_frame(warnings, 96.0);
    assert_compact_status_item_frame(messages, 100.0);

    assert!(bridge
        .control_frame("WorkbenchStatusTaskProgress")
        .is_none());
    assert_compact_status_item_frame(
        bridge
            .control_frame("WorkbenchStatusGrid")
            .expect("grid status should stay visible"),
        80.0,
    );
    assert_compact_status_item_frame(
        bridge
            .control_frame("WorkbenchStatusSnap")
            .expect("snap status should stay visible"),
        74.0,
    );
    assert_compact_status_item_frame(
        bridge
            .control_frame("WorkbenchStatusSnapToggle")
            .expect("snap toggle should stay visible"),
        28.0,
    );
    assert_compact_status_item_frame(
        bridge
            .control_frame("WorkbenchStatusWorld")
            .expect("world status should stay visible"),
        28.0,
    );
    assert_compact_status_item_frame(
        bridge
            .control_frame("WorkbenchStatusTarget")
            .expect("target status should stay visible"),
        28.0,
    );
    assert_compact_status_item_frame(
        bridge
            .control_frame("WorkbenchStatusZoom")
            .expect("zoom status should stay visible"),
        56.0,
    );

    let nodes = to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    assert!(template_contract_node_optional(&nodes, "WorkbenchStatusTaskProgress").is_none());
}

#[test]
fn componentized_workbench_status_bar_adapts_primary_runtime_text_without_shrinking_fixed_controls()
{
    let _guard = env_lock().lock().unwrap();
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.status_line = "Blend space opened".to_string();
    chrome.console_output = "Blend space opened".into();
    let required_primary_width = status_signal_required_width(&chrome.status_line);

    for surface_width in status_contract_widths() {
        let bridge = status_bridge(surface_width, &chrome);

        let status_bar = bridge.frames().status_bar;
        let ready = status_frame(&bridge, "WorkbenchStatusReady");
        let before_task = expected_status_controls_before_task(surface_width);
        let after_task = expected_status_controls_after_task(surface_width);

        for &(control_id, expected_width) in before_task.iter().chain(&after_task) {
            assert_compact_status_item_frame(status_frame(&bridge, control_id), expected_width);
        }

        assert!(
            ready.width >= required_primary_width,
            "{surface_width}px idle status bar should preserve the full primary Runtime Text: ready={ready:?}, required={required_primary_width}"
        );

        let mut previous_right = ready.x + ready.width;
        for &(control_id, _) in before_task.iter().chain(&after_task) {
            let frame = status_frame(&bridge, control_id);
            assert!(
                frame.x + 0.01 >= previous_right,
                "{control_id} should not overlap the preceding adaptive status slot at {surface_width}px: previous_right={previous_right}, frame={frame:?}"
            );
            previous_right = frame.x + frame.width;
        }
        assert!(
            (previous_right - (status_bar.x + status_bar.width)).abs() <= 0.01,
            "right status controls should remain pinned to the status-bar edge at {surface_width}px: status_bar={status_bar:?}, right={previous_right}"
        );
    }
}

#[test]
fn componentized_workbench_status_bar_prioritizes_primary_text_and_active_task_by_tier() {
    let _guard = env_lock().lock().unwrap();
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.status_line = "Blend space opened".to_string();
    chrome.console_output = "Blend space opened".into();
    chrome.status_task_progress = Some(
        StatusTaskProgressSnapshot::new("desktop_export:7", "Export desktop_windows")
            .with_detail("cargo-build")
            .with_percent(72)
            .with_tone(StatusTaskProgressTone::Info),
    );
    let required_primary_width = status_signal_required_width(&chrome.status_line);

    for surface_width in status_contract_widths() {
        let bridge = status_bridge(surface_width, &chrome);
        let status_bar = bridge.frames().status_bar;
        let ready = status_frame(&bridge, "WorkbenchStatusReady");
        let task = status_frame(&bridge, "WorkbenchStatusTaskProgress");
        let task_label = status_frame(&bridge, "WorkbenchStatusTaskLabel");
        let task_bar = status_frame(&bridge, "WorkbenchStatusTaskBar");
        let before_task = expected_status_controls_before_task(surface_width);
        let after_task = expected_status_controls_after_task(surface_width);

        assert!(
            ready.width >= required_primary_width,
            "{surface_width}px active-task status bar should preserve the full primary Runtime Text: ready={ready:?}, required={required_primary_width}"
        );
        assert!(
            (160.0..=224.0).contains(&task.width),
            "task composite should remain within its authored compression range at {surface_width}px: {task:?}"
        );
        if (surface_width - 641.0).abs() < f32::EPSILON {
            assert!(
                task.width < 224.0,
                "regular-tier lower bound should compress the task composite before primary text: {task:?}"
            );
        }
        assert!(
            (100.0..=132.0).contains(&task_label.width),
            "task label should stay within its authored compression range at {surface_width}px: {task_label:?}"
        );
        assert!(
            (52.0..=84.0).contains(&task_bar.width),
            "task progress should stay within its authored compression range at {surface_width}px: {task_bar:?}"
        );
        assert!(task_label.x + 0.01 >= task.x);
        assert!(task_bar.x + 0.01 >= task_label.x + task_label.width + 8.0);
        assert!(task_bar.x + task_bar.width <= task.x + task.width + 0.01);

        let mut previous_right = ready.x + ready.width;
        for &(control_id, expected_width) in &before_task {
            let frame = status_frame(&bridge, control_id);
            assert_compact_status_item_frame(frame, expected_width);
            assert!(frame.x + 0.01 >= previous_right);
            previous_right = frame.x + frame.width;
        }
        assert!(task.x + 0.01 >= previous_right);
        previous_right = task.x + task.width;

        for &(control_id, expected_width) in &after_task {
            let frame = status_frame(&bridge, control_id);
            assert_compact_status_item_frame(frame, expected_width);
            assert!(frame.x + 0.01 >= previous_right);
            previous_right = frame.x + frame.width;
        }
        assert!(
            (previous_right - (status_bar.x + status_bar.width)).abs() <= 0.01,
            "active-task controls should remain pinned to the status-bar edge at {surface_width}px: status_bar={status_bar:?}, right={previous_right}"
        );
    }
}

fn status_bridge(
    surface_width: f32,
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
) -> BuiltinWorkbenchWindowTemplateSurfaceBridge {
    let shell_size = UiSize::new(surface_width, 620.0);
    let model = WorkbenchViewModel::build(&EditorCommandRegistry::default_workbench(), chrome);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            shell_size,
            &model,
            &WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    bridge.sync_from_chrome(chrome).unwrap();
    bridge
}

fn status_contract_widths() -> [f32; 8] {
    [420.0, 480.0, 481.0, 640.0, 641.0, 900.0, 1259.0, 1260.0]
}

fn expected_status_controls_before_task(surface_width: f32) -> Vec<(&'static str, f32)> {
    let mut controls = Vec::new();
    if matches!(
        workbench_layout_tier_for_logical_width(surface_width),
        WorkbenchLayoutTier::Regular | WorkbenchLayoutTier::Wide
    ) {
        controls.extend([
            ("WorkbenchStatusErrors", 92.0),
            ("WorkbenchStatusWarnings", 96.0),
            ("WorkbenchStatusMessages", 100.0),
        ]);
    }
    controls
}

fn expected_status_controls_after_task(surface_width: f32) -> Vec<(&'static str, f32)> {
    let mut controls = Vec::new();
    if workbench_layout_tier_for_logical_width(surface_width) == WorkbenchLayoutTier::Wide {
        controls.extend([
            ("WorkbenchStatusGrid", 80.0),
            ("WorkbenchStatusSnap", 74.0),
            ("WorkbenchStatusSnapToggle", 28.0),
            ("WorkbenchStatusWorld", 28.0),
            ("WorkbenchStatusTarget", 28.0),
            ("WorkbenchStatusZoom", 56.0),
        ]);
    }
    controls
}

fn status_signal_required_width(text: &str) -> f32 {
    let density = EditorDensityTokens::workbench_dense();
    let controls = EditorControlTokens::workbench_dense();
    let clip_guard = density.gap_medium - controls.border_width * 2.0;
    measure_runtime_text_width(text, EditorTypographyTokens::WORKBENCH_BODY_SIZE)
        + density.gap_medium * 3.0
        + clip_guard
}

fn status_frame(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge, control_id: &str) -> UiFrame {
    bridge.control_frame(control_id).unwrap_or_else(|| {
        panic!(
            "{control_id} should stay visible in the status bar; authored status region={:?}",
            bridge.frames().status_bar
        )
    })
}

fn assert_compact_status_item_frame(frame: UiFrame, expected_width: f32) {
    assert!((frame.width - expected_width).abs() < 0.001);
    assert!((frame.height - 46.0).abs() < 0.001 || (frame.height - 30.0).abs() < 0.001);
}

fn template_contract_node<'a>(
    nodes: &'a ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> &'a TemplatePaneNodeData {
    (0..nodes.row_count())
        .filter_map(|row| nodes.get(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to the host contract"))
}

fn template_contract_node_optional<'a>(
    nodes: &'a ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> Option<&'a TemplatePaneNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.get(row))
        .find(|node| node.control_id.as_str() == control_id)
}
