use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{UiDebugTimelineFrameHandle, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot},
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use crate::ui::surface::UiDebugTimelineStore;

#[test]
fn ui_debug_timeline_store_retains_latest_frames_and_reports_dropped_count() {
    let surface = diagnostic_surface();
    let options = timeline_options();
    let mut store = UiDebugTimelineStore::new(2);

    let first = store.capture_snapshot(
        timeline_snapshot(&surface, 10, "Frame 10", UiNodeId::new(2)),
        options.clone(),
    );
    let second = store.capture_snapshot(
        timeline_snapshot(&surface, 11, "Frame 11", UiNodeId::new(3)),
        options.clone(),
    );
    let third = store.capture_snapshot(
        timeline_snapshot(&surface, 12, "Frame 12", UiNodeId::new(2)),
        options.clone(),
    );

    assert_eq!(first, UiDebugTimelineFrameHandle(1));
    assert_eq!(second, UiDebugTimelineFrameHandle(2));
    assert_eq!(third, UiDebugTimelineFrameHandle(3));

    let timeline = store.snapshot();
    assert_eq!(timeline.retention.capacity, 2);
    assert_eq!(timeline.retention.len, 2);
    assert_eq!(timeline.retention.first_frame, Some(second));
    assert_eq!(timeline.retention.latest_frame, Some(third));
    assert_eq!(timeline.retention.selected_frame, Some(third));
    assert_eq!(timeline.retention.dropped_frame_count, 1);
    assert_eq!(
        timeline
            .summaries
            .iter()
            .map(|summary| summary.frame_index)
            .collect::<Vec<_>>(),
        vec![11, 12]
    );
    assert_eq!(
        timeline
            .summaries
            .iter()
            .map(|summary| summary.source_label.as_str())
            .collect::<Vec<_>>(),
        vec!["Frame 11", "Frame 12"]
    );
    assert_eq!(timeline.summaries[1].capture_options, options);
    assert_eq!(timeline.summaries[1].selected_node, Some(UiNodeId::new(2)));
    assert_eq!(timeline.frames.len(), 2);
}

#[test]
fn ui_debug_timeline_store_selects_retained_frame_and_rejects_evicted_handle() {
    let surface = diagnostic_surface();
    let options = timeline_options();
    let mut store = UiDebugTimelineStore::new(2);

    let first = store.capture_snapshot(
        timeline_snapshot(&surface, 1, "Frame 1", UiNodeId::new(2)),
        options.clone(),
    );
    let second = store.capture_snapshot(
        timeline_snapshot(&surface, 2, "Frame 2", UiNodeId::new(3)),
        options.clone(),
    );

    assert!(store.select_frame(first));
    assert_eq!(store.snapshot().retention.selected_frame, Some(first));

    let third = store.capture_snapshot(
        timeline_snapshot(&surface, 3, "Frame 3", UiNodeId::new(2)),
        options,
    );

    assert_eq!(second, UiDebugTimelineFrameHandle(2));
    assert_eq!(third, UiDebugTimelineFrameHandle(3));
    assert!(!store.select_frame(first));
    let timeline = store.snapshot();
    assert_eq!(timeline.retention.first_frame, Some(second));
    assert_eq!(timeline.retention.latest_frame, Some(third));
    assert_eq!(timeline.retention.selected_frame, Some(third));
}

#[test]
fn ui_debug_timeline_selection_does_not_mutate_surface_snapshot_source() {
    let surface = diagnostic_surface();
    let before = surface.debug_snapshot();
    let options = timeline_options();
    let mut store = UiDebugTimelineStore::new(3);

    let first = store.capture_snapshot(
        timeline_snapshot(&surface, 21, "History 21", UiNodeId::new(2)),
        options.clone(),
    );
    let second = store.capture_snapshot(
        timeline_snapshot(&surface, 22, "History 22", UiNodeId::new(3)),
        options,
    );

    assert!(store.select_frame(first));
    assert!(store.select_frame(second));

    let after = surface.debug_snapshot();
    assert_eq!(after.tree_id, before.tree_id);
    assert_eq!(after.nodes.len(), before.nodes.len());
    assert_eq!(after.render.command_count, before.render.command_count);
    assert_eq!(after.capture.selected_node, before.capture.selected_node);
}

fn timeline_options() -> UiSurfaceDebugOptions {
    UiSurfaceDebugOptions {
        include_hit_cells: false,
        include_overdraw_cells: false,
        include_overlay_primitives: false,
        ..UiSurfaceDebugOptions::default()
    }
}

fn timeline_snapshot(
    surface: &UiSurface,
    frame_index: u64,
    label: &str,
    selected_node: UiNodeId,
) -> UiSurfaceDebugSnapshot {
    let mut snapshot = surface.debug_snapshot_for_selection(selected_node, &timeline_options());
    snapshot.capture.frame_index = Some(frame_index);
    snapshot.capture.captured_at_millis = Some(frame_index * 100);
    snapshot.capture.surface_name = Some(label.to_string());
    snapshot
}

fn diagnostic_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.timeline"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 80.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                2,
                "root/back",
                "timeline.back",
                UiFrame::new(0.0, 0.0, 80.0, 40.0),
                0,
            ),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            button_node(
                3,
                "root/front",
                "timeline.front",
                UiFrame::new(40.0, 0.0, 80.0, 40.0),
                10,
            ),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn button_node(
    node_id: u64,
    node_path: &str,
    control_id: &str,
    frame: UiFrame,
    z_index: i32,
) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(node_id), UiNodePath::new(node_path))
        .with_frame(frame)
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            attributes: toml::from_str(
                r##"
text = "Timeline"
opacity = 1.0

[background]
color = "#224466"
"##,
            )
            .unwrap(),
            ..Default::default()
        })
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
