use super::support::*;
use crate::scene::viewport::GridMode;
use crate::tests::host::retained_callback_dispatch::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, UiSize,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::{to_host_contract_workbench_window_nodes, TemplatePaneNodeData};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::snapshot::{StatusTaskProgressSnapshot, StatusTaskProgressTone};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::tree::UiVisibility;

#[test]
fn componentized_workbench_status_bar_syncs_chrome_and_task_progress() {
    let _guard = env_lock().lock().unwrap();
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.status_line = "Saving project".to_string();
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
    assert_compact_status_item_frame(ready, 72.0);
    assert_compact_status_item_frame(errors, 92.0);
    assert_compact_status_item_frame(warnings, 96.0);
    assert_compact_status_item_frame(messages, 100.0);

    assert!(bridge.control_frame("WorkbenchStatusFill").is_none());
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
    assert!(template_contract_node_optional(&nodes, "WorkbenchStatusFill").is_none());
    assert!(template_contract_node_optional(&nodes, "WorkbenchStatusTaskProgress").is_none());
}

fn assert_compact_status_item_frame(frame: UiFrame, expected_width: f32) {
    assert!((frame.width - expected_width).abs() < 0.001);
    assert!((frame.height - 46.0).abs() < 0.001 || (frame.height - 30.0).abs() < 0.001);
}

fn template_contract_node(
    nodes: &ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> TemplatePaneNodeData {
    template_contract_node_optional(nodes, control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to the host contract"))
}

fn template_contract_node_optional(
    nodes: &ModelRc<TemplatePaneNodeData>,
    control_id: &str,
) -> Option<TemplatePaneNodeData> {
    (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
}
