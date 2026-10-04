use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiPoint,
    tree::{UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
    widget::{UiPopupAnchor, UiWidgetBehavior},
};

use super::{popup_stack_id_matches, tree_popup_stack_record, UiPopupDependencyImpact, UiSurface};

fn open_popup_node(node_id: UiNodeId) -> UiTreeNode {
    let mut metadata = UiTemplateNodeMetadata {
        component: "Popover".to_string(),
        ..UiTemplateNodeMetadata::default()
    };
    metadata
        .attributes
        .insert("open".to_string(), toml::Value::Boolean(true));
    UiTreeNode::new(node_id, UiNodePath::new("root/popup")).with_template_metadata(metadata)
}

#[test]
fn disabled_popup_owner_is_not_seeded_into_runtime_stack() {
    let mut node = open_popup_node(UiNodeId::new(7));
    node.state_flags.enabled = false;

    assert_eq!(
        tree_popup_stack_record(node.node_id, &node),
        Some((node.node_id, ("root/popup".to_string(), false)))
    );
}

#[test]
fn collapsed_popup_owner_is_not_seeded_into_runtime_stack() {
    let node = open_popup_node(UiNodeId::new(7)).with_visibility(UiVisibility::Collapsed);

    assert_eq!(
        tree_popup_stack_record(node.node_id, &node),
        Some((node.node_id, ("root/popup".to_string(), false)))
    );
}

#[test]
fn popup_stack_id_match_preserves_path_and_canonical_node_id_semantics() {
    let path_node = UiTreeNode::new(UiNodeId::new(7), UiNodePath::new("root/popup"));
    assert!(popup_stack_id_matches(&path_node, "root/popup"));
    assert!(!popup_stack_id_matches(&path_node, "node:7"));

    let id_node = UiTreeNode::new(UiNodeId::new(7), UiNodePath::new(""));
    assert!(popup_stack_id_matches(&id_node, "node:7"));
    assert!(!popup_stack_id_matches(&id_node, "node:07"));
    assert!(!popup_stack_id_matches(&id_node, "node:+7"));
    assert!(!popup_stack_id_matches(&id_node, "node:invalid"));
}

#[test]
fn popup_dependency_impact_preserves_independent_domain_semantics() {
    let mut surface = UiSurface::new(UiTreeId::new("popup.dependency.impact"));
    surface.tree.insert_root(UiTreeNode::new(
        UiNodeId::new(1),
        UiNodePath::new("root/normal"),
    ));

    let mut closed_popup = open_popup_node(UiNodeId::new(2));
    closed_popup
        .template_metadata
        .as_mut()
        .expect("popup metadata")
        .attributes
        .insert("open".to_string(), toml::Value::Boolean(false));
    surface.tree.insert_root(closed_popup);

    let mut control_popup = open_popup_node(UiNodeId::new(3));
    control_popup
        .template_metadata
        .as_mut()
        .expect("popup metadata")
        .widget
        .popup_anchor = UiPopupAnchor::Control {
        control_id: "trigger".to_string(),
    };
    surface.tree.insert_root(control_popup);

    let impact_for =
        |node_id| surface.popup_dependency_impact(&BTreeSet::from([UiNodeId::new(node_id)]));
    assert_eq!(
        impact_for(99),
        UiPopupDependencyImpact {
            render_extract: true,
            stack_reconciliation: false,
        }
    );
    assert_eq!(
        impact_for(2),
        UiPopupDependencyImpact {
            render_extract: false,
            stack_reconciliation: true,
        }
    );
    assert_eq!(
        impact_for(3),
        UiPopupDependencyImpact {
            render_extract: true,
            stack_reconciliation: true,
        }
    );
    assert_eq!(impact_for(1), UiPopupDependencyImpact::default());
}

#[test]
fn dynamic_control_anchor_retargets_open_popup_without_layout_invalidation() {
    let mut surface = UiSurface::new(UiTreeId::new("popup.dynamic.control.anchor"));
    surface.tree.insert_root(control_node(1, "first"));
    surface.tree.insert_root(control_node(2, "second"));

    let mut popup = open_popup_node(UiNodeId::new(3));
    let popup_metadata = popup.template_metadata.as_mut().expect("popup metadata");
    popup_metadata.widget.behavior = UiWidgetBehavior::Popup;
    popup_metadata.widget.popup_anchor = UiPopupAnchor::Control {
        control_id: "first".to_string(),
    };
    surface.tree.insert_root(popup);
    surface.seed_popup_stack_from_tree_metadata();
    surface.clear_dirty_flags();

    assert!(surface
        .set_popup_control_anchor(UiNodeId::new(3), "second")
        .unwrap());
    let popup = surface.tree.node(UiNodeId::new(3)).unwrap();
    assert_eq!(
        popup
            .template_metadata
            .as_ref()
            .unwrap()
            .widget
            .popup_anchor
            .control_id(),
        Some("second")
    );
    assert_eq!(surface.input.popup_stack.len(), 1);
    assert_eq!(surface.input.popup_stack[0].owner, Some(UiNodeId::new(2)));
    assert!(popup.dirty.render);
    assert!(popup.dirty.input);
    assert!(!popup.dirty.layout);
    assert!(!popup.dirty.hit_test);
    assert!(!surface
        .set_popup_control_anchor(UiNodeId::new(3), "second")
        .unwrap());
}

#[test]
fn pointer_anchor_captures_transient_point_and_restores_target_owner() {
    let mut surface = UiSurface::new(UiTreeId::new("popup.pointer.anchor"));
    surface.tree.insert_root(control_node(1, "target"));

    let mut popup = open_popup_node(UiNodeId::new(2));
    let popup_metadata = popup.template_metadata.as_mut().expect("popup metadata");
    popup_metadata.widget.behavior = UiWidgetBehavior::Popup;
    popup_metadata.widget.popup_anchor = UiPopupAnchor::Pointer {
        owner_property: "context_target".to_string(),
    };
    popup_metadata.attributes.insert(
        "context_target".to_string(),
        toml::Value::String("target".to_string()),
    );
    surface.tree.insert_root(popup);
    surface.clear_dirty_flags();

    let point = UiPoint::new(48.0, 72.0);
    assert!(surface
        .set_popup_pointer_anchor(UiNodeId::new(2), point)
        .unwrap());
    assert_eq!(
        surface.input.popup_anchor_point(UiNodeId::new(2)),
        Some(point)
    );
    assert_eq!(surface.input.popup_stack.len(), 1);
    assert_eq!(surface.input.popup_stack[0].owner, Some(UiNodeId::new(1)));
    assert_eq!(surface.input.popup_stack[0].anchor, Some(point));
    let popup = surface.tree.node(UiNodeId::new(2)).unwrap();
    assert!(popup.dirty.render);
    assert!(popup.dirty.input);
    assert!(!popup.dirty.layout);

    assert_eq!(
        surface.sync_popup_stack_for_node(UiNodeId::new(2), false),
        None
    );
    assert!(surface.input.popup_stack.is_empty());
    assert_eq!(surface.input.popup_anchor_point(UiNodeId::new(2)), None);
}

fn control_node(node_id: u64, control_id: &str) -> UiTreeNode {
    UiTreeNode::new(
        UiNodeId::new(node_id),
        UiNodePath::new(format!("root/{control_id}")),
    )
    .with_template_metadata(UiTemplateNodeMetadata {
        control_id: Some(control_id.to_string()),
        ..UiTemplateNodeMetadata::default()
    })
}
