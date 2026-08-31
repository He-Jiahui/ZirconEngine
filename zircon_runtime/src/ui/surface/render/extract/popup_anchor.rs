use std::collections::BTreeMap;

use crate::ui::surface::{UiArrangedVisibilityIndex, UiSurfaceControlIndex, arranged_node_indexed};
use zircon_runtime_interface::ui::surface::UiArrangedTree;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
    tree::{UiTemplateNodeMetadata, UiTree},
    widget::{UiPopupAnchor, UiWidgetBehavior},
};

pub(super) fn resolve_popup_anchor_frame(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    arranged_visibility: &UiArrangedVisibilityIndex,
    popup_node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    owner_frame: UiFrame,
    control_index: Option<&UiSurfaceControlIndex>,
    popup_anchor_points: Option<&BTreeMap<UiNodeId, UiPoint>>,
) -> Option<UiFrame> {
    let Some(metadata) = metadata else {
        return Some(owner_frame);
    };
    let control_id = match &metadata.widget.popup_anchor {
        UiPopupAnchor::None => return Some(owner_frame),
        UiPopupAnchor::Surface => {
            return arranged_surface_root_frame(arranged_tree, node_indices, popup_node_id);
        }
        UiPopupAnchor::Pointer { .. } => {
            let point = popup_anchor_points?.get(&popup_node_id)?;
            return Some(UiFrame::new(point.x, point.y, 0.0, 0.0));
        }
        UiPopupAnchor::Control { control_id } => control_id,
    };
    let trigger_node_id = match control_index {
        Some(control_index) => control_index.unique_node_id_for_surface(tree, control_id),
        None => unique_control_node_id(tree, control_id),
    }?;
    if !arranged_visibility.is_render_visible(trigger_node_id) {
        return None;
    }
    let trigger = tree.nodes.get(&trigger_node_id)?;
    if !trigger.state_flags.enabled {
        return None;
    }
    let trigger_frame = arranged_node_indexed(arranged_tree, node_indices, trigger_node_id)
        .ok()?
        .frame;
    (trigger_frame.x.is_finite()
        && trigger_frame.y.is_finite()
        && trigger_frame.width.is_finite()
        && trigger_frame.height.is_finite()
        && trigger_frame.width > 0.0
        && trigger_frame.height > 0.0)
        .then_some(trigger_frame)
}

fn arranged_surface_root_frame(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Option<UiFrame> {
    let mut current = node_id;
    for _ in 0..=arranged_tree.nodes.len() {
        let node = arranged_node_indexed(arranged_tree, node_indices, current).ok()?;
        let Some(parent) = node.parent else {
            let frame = node.frame;
            return (frame.x.is_finite()
                && frame.y.is_finite()
                && frame.width.is_finite()
                && frame.height.is_finite()
                && frame.width > 0.0
                && frame.height > 0.0)
                .then_some(frame);
        };
        current = parent;
    }
    None
}

fn unique_control_node_id(tree: &UiTree, control_id: &str) -> Option<UiNodeId> {
    let mut matches = tree.nodes.iter().filter_map(|(node_id, node)| {
        (node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some(control_id))
        .then_some(*node_id)
    });
    let node_id = matches.next()?;
    matches.next().is_none().then_some(node_id)
}

pub(super) fn popup_runtime_anchor_is_open(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    let Some(metadata) = metadata else {
        return false;
    };
    matches!(
        &metadata.widget.popup_anchor,
        UiPopupAnchor::Control { .. } | UiPopupAnchor::Surface | UiPopupAnchor::Pointer { .. }
    ) && (metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Popup
        || matches!(
            metadata.component.as_str(),
            "Dialog" | "ConfirmDialog" | "Modal" | "Popover"
        ))
        && ["popup_open", "open"]
            .iter()
            .any(|key| metadata.attributes.get(*key).and_then(toml::Value::as_bool) == Some(true))
}

#[cfg(test)]
mod tests {
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
}
