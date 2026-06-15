use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiTemplateNodeMetadata, UiTreeNode},
    widget::UiWidgetBehavior,
};

use super::UiSurface;

impl UiSurface {
    pub(crate) fn seed_popup_stack_from_tree_metadata(&mut self) {
        let open_popups = self
            .tree
            .nodes
            .iter()
            .filter_map(|(node_id, node)| tree_popup_stack_record(*node_id, node))
            .collect::<BTreeMap<_, _>>();

        self.input.popup_stack.retain(|popup| {
            let Some(owner) = popup.owner else {
                return true;
            };
            open_popups
                .get(&owner)
                .is_none_or(|(popup_id, open)| popup.popup_id != *popup_id || *open)
        });

        let open_popups = open_popups
            .into_iter()
            .filter_map(|(node_id, (popup_id, open))| open.then_some((node_id, popup_id)))
            .collect::<Vec<_>>();

        for (node_id, popup_id) in open_popups {
            self.input.open_popup(popup_id, Some(node_id), None);
        }
    }

    pub(crate) fn sync_popup_stack_for_node(&mut self, node_id: UiNodeId, open: bool) {
        let Some(node) = self.tree.nodes.get(&node_id) else {
            return;
        };
        let Some(metadata) = node.template_metadata.as_ref() else {
            return;
        };
        if !is_popup_stack_metadata(metadata) {
            return;
        }
        let popup_id = popup_stack_id_for_node(node);
        if open {
            self.input.open_popup(popup_id, Some(node_id), None);
        } else {
            self.input.close_popup(popup_id.as_str());
        }
    }
}

fn tree_popup_stack_record(
    node_id: UiNodeId,
    node: &UiTreeNode,
) -> Option<(UiNodeId, (String, bool))> {
    let metadata = node.template_metadata.as_ref()?;
    is_popup_stack_metadata(metadata).then_some((
        node_id,
        (
            popup_stack_id_for_node(node),
            tree_node_popup_open(metadata),
        ),
    ))
}

fn is_popup_stack_metadata(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Popup
        || matches!(
            metadata.component.as_str(),
            "Dialog" | "ConfirmDialog" | "Modal" | "Popover"
        )
}

fn tree_node_popup_open(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "popup_open") || bool_attribute(metadata, "open")
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn popup_stack_id_for_node(node: &UiTreeNode) -> String {
    if node.node_path.0.is_empty() {
        format!("node:{}", node.node_id.0)
    } else {
        node.node_path.0.clone()
    }
}
