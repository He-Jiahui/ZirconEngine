use crate::ui::{layout::compute_layout_tree, surface::UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        Anchor, AxisConstraint, BoxConstraints, Position, StretchMode, UiAlignment, UiAlignment2D,
        UiCanvasSlotPlacement, UiContainerKind, UiFrame, UiGridBoxConfig, UiGridSlotPlacement,
        UiLayoutEngineBackend, UiLayoutEngineFallbackReason, UiLayoutEngineFamily,
        UiLayoutEngineSupport, UiLinearBoxConfig, UiLinearSlotSizeRule, UiLinearSlotSizing,
        UiMargin, UiScrollableBoxConfig, UiSize, UiSizeBoxConfig, UiSlot, UiSlotKind,
        UiVirtualListConfig, UiWrapBoxConfig,
    },
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode, UiVisibility},
};

mod arrangement;
mod fallback_policy;
mod grid_slots;
mod linear_slots;
mod routing_diagnostics;

fn tree_with_root(root_id: u64, container: UiContainerKind) -> UiTree {
    let mut tree = UiTree::new(UiTreeId::new(format!("taffy.layout.{root_id}")));
    tree.insert_root(node(root_id).with_container(container));
    tree
}

fn insert_child(tree: &mut UiTree, parent_id: u64, child: UiTreeNode) {
    tree.insert_child(UiNodeId::new(parent_id), child)
        .expect("insert child");
}

fn node(id: u64) -> UiTreeNode {
    UiTreeNode::new(UiNodeId::new(id), UiNodePath::new(format!("node.{id}")))
}

fn fixed_node(id: u64, width: Option<f32>, height: Option<f32>) -> UiTreeNode {
    let mut constraints = BoxConstraints::default();
    if let Some(width) = width {
        constraints.width = fixed_axis(width);
    }
    if let Some(height) = height {
        constraints.height = fixed_axis(height);
    }
    node(id).with_constraints(constraints)
}

fn priority_stretch_node(id: u64, width_priority: i32) -> UiTreeNode {
    let mut constraints = BoxConstraints::default();
    constraints.width = AxisConstraint {
        min: 0.0,
        max: -1.0,
        preferred: 0.0,
        priority: width_priority,
        weight: 1.0,
        stretch_mode: StretchMode::Stretch,
    };
    constraints.height = fixed_axis(10.0);
    node(id).with_constraints(constraints)
}

fn fixed_node_with_axis_max(id: u64, max: f32) -> UiTreeNode {
    let mut constraints = BoxConstraints::default();
    constraints.width = AxisConstraint {
        min: 0.0,
        max,
        preferred: 20.0,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    };
    constraints.height = fixed_axis(10.0);
    node(id).with_constraints(constraints)
}

fn fixed_axis(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: value,
        preferred: value,
        priority: 0,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn frame(tree: &UiTree, id: u64) -> UiFrame {
    tree.node(UiNodeId::new(id))
        .expect("node")
        .layout_cache
        .frame
}

fn template_metadata(component: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        ..UiTemplateNodeMetadata::default()
    }
}

fn metadata_with_attributes(component: &str, attributes: &str) -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: component.to_string(),
        attributes: toml::from_str(attributes).expect("metadata attributes"),
        ..UiTemplateNodeMetadata::default()
    }
}

fn selection_for_node<'a>(
    report: &'a zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
) -> &'a zircon_runtime_interface::ui::layout::UiLayoutEngineSelection {
    report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(node_id)))
        .expect("layout engine selection")
}

fn assert_taffy_native_family(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
    family: UiLayoutEngineFamily,
) {
    let selection = selection_for_node(report, node_id);
    assert_eq!(selection.request.family, family);
    assert_eq!(selection.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(selection.support, UiLayoutEngineSupport::Native);
    assert_eq!(selection.fallback_reason, None);
}

fn assert_zircon_owned_route(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
    family: UiLayoutEngineFamily,
) {
    assert_fallback_route_reason(
        report,
        node_id,
        family,
        UiLayoutEngineFallbackReason::ZirconOwnedSemantics,
    );
}

fn assert_fallback_route_reason(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    node_id: u64,
    family: UiLayoutEngineFamily,
    reason: UiLayoutEngineFallbackReason,
) {
    let selection = selection_for_node(report, node_id);
    assert_eq!(selection.request.family, family);
    assert_eq!(selection.selected_backend, UiLayoutEngineBackend::Zircon);
    assert_eq!(selection.support, UiLayoutEngineSupport::Fallback);
    assert_eq!(selection.fallback_reason, Some(reason));
}

fn assert_fallback_reason_count(
    report: &zircon_runtime_interface::ui::layout::UiLayoutEngineSelectionReport,
    reason: UiLayoutEngineFallbackReason,
    count: u64,
) {
    let reason_count = report
        .fallback_reason_counts
        .iter()
        .find(|reason_count| reason_count.reason == Some(reason))
        .expect("fallback reason count");
    assert_eq!(reason_count.count, count);
}
