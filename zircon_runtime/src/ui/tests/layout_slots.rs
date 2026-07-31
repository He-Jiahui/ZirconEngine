use crate::ui::surface::{hit_test_surface_frame, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, DesiredSize, Pivot, Position, StretchMode,
        UiAlignment, UiAlignment2D, UiAxis, UiCanvasSlotPlacement, UiContainerKind, UiFrame,
        UiGridBoxConfig, UiGridSlotPlacement, UiLayoutEngineBackend, UiLayoutEngineFamily,
        UiLayoutEngineSupport, UiLinearBoxConfig, UiMargin, UiMasonryBoxConfig, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiSlot, UiSlotKind,
        UiVirtualListConfig, UiVirtualListWindow,
    },
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

mod flow_grid_masonry;
mod linear_free;
mod overlay_scroll;

#[test]
fn layout_measure_reuses_profile_flag_and_sorts_desired_payload_once() {
    let source = include_str!("../layout/pass/measure.rs");

    assert!(source.contains("measure_node_with_profile("));
    assert!(!source.contains("node.template_metadata.clone()"));
    assert!(!source.contains(".find(|(child_id, _)| child_id == ordered_child_id)"));
    assert!(source.contains("(order, index, child_id, desired)"));
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn pointer_node(
    id: u64,
    path: impl Into<String>,
    control_id: impl Into<String>,
    constraints: BoxConstraints,
    z_index: i32,
) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(path))
        .with_constraints(constraints)
        .with_z_index(z_index)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state())
        .with_template_metadata(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.into()),
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
        ..Default::default()
    }
}

fn render_frame_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
    node_id: UiNodeId,
) -> Option<UiFrame> {
    frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .map(|command| command.frame)
}

fn render_z_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
    node_id: UiNodeId,
) -> Option<i32> {
    frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == node_id)
        .map(|command| command.z_index)
}

fn hit_frame_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
    node_id: UiNodeId,
) -> Option<UiFrame> {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .map(|entry| entry.frame)
}

fn hit_z_for(
    frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame,
    node_id: UiNodeId,
) -> Option<i32> {
    frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == node_id)
        .map(|entry| entry.z_index)
}
