use super::*;
use crate::ui::surface::arranged_node_indices;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    surface::UiArrangedNode,
    tree::{UiInputPolicy, UiTree, UiVisibility},
    widget::UiWidgetContract,
};

#[test]
fn surface_anchor_resolves_from_the_arranged_root() {
    let root_node_id = UiNodeId::new(1);
    let popup_node_id = UiNodeId::new(7);
    let surface_frame = UiFrame::new(12.0, 24.0, 960.0, 540.0);
    let arranged_tree = UiArrangedTree {
        tree_id: UiTreeId::new("surface.anchor.extract"),
        roots: vec![root_node_id].into(),
        nodes: vec![
            arranged_test_node(root_node_id, None, surface_frame),
            arranged_test_node(
                popup_node_id,
                Some(root_node_id),
                UiFrame::new(0.0, 0.0, 560.0, 220.0),
            ),
        ]
        .into(),
        draw_order: vec![root_node_id, popup_node_id].into(),
        canvas_layers: Vec::new().into(),
    };
    let indices = arranged_node_indices(&arranged_tree);

    assert_eq!(
        arranged_surface_root_frame(&arranged_tree, &indices, popup_node_id),
        Some(surface_frame)
    );
}

#[test]
fn pointer_anchor_resolves_from_transient_surface_state() {
    let popup_node_id = UiNodeId::new(7);
    let metadata = UiTemplateNodeMetadata {
        widget: UiWidgetContract {
            popup_anchor: UiPopupAnchor::Pointer {
                owner_property: "context_target".to_string(),
            },
            ..UiWidgetContract::default()
        },
        ..UiTemplateNodeMetadata::default()
    };
    let points = BTreeMap::from([(popup_node_id, UiPoint::new(42.0, 64.0))]);

    assert_eq!(
        resolve_popup_anchor_frame(
            &UiTree::new(UiTreeId::new("pointer.anchor.extract")),
            &UiArrangedTree::default(),
            &BTreeMap::new(),
            &UiArrangedVisibilityIndex::default(),
            popup_node_id,
            Some(&metadata),
            UiFrame::new(0.0, 0.0, 120.0, 72.0),
            None,
            Some(&points),
        ),
        Some(UiFrame::new(42.0, 64.0, 0.0, 0.0))
    );
}

fn arranged_test_node(
    node_id: UiNodeId,
    parent: Option<UiNodeId>,
    frame: UiFrame,
) -> UiArrangedNode {
    UiArrangedNode {
        node_id,
        node_path: UiNodePath::new(format!("node/{}", node_id.0)),
        parent,
        children: Vec::new(),
        frame,
        clip_frame: frame,
        z_index: 0,
        paint_order: node_id.0,
        visibility: UiVisibility::Visible,
        input_policy: UiInputPolicy::Receive,
        pointer_events: Default::default(),
        enabled: true,
        clickable: false,
        hoverable: false,
        focusable: false,
        clip_to_bounds: false,
        control_id: None,
        slot: None,
    }
}
