use crate::ui::template::UiTemplateInstance;
use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiScrollState,
    template::UiTemplateNode,
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
};

use super::build_error::UiTemplateBuildError;
use super::child_segment::child_segment;
use super::container_inference::infer_container;
use super::interaction::infer_interaction;
use super::layout_contract::infer_layout_contract;

#[derive(Default)]
pub struct UiTemplateTreeBuilder {
    next_node_id: u64,
}

impl UiTemplateTreeBuilder {
    pub fn build_tree(
        tree_id: UiTreeId,
        instance: &UiTemplateInstance,
    ) -> Result<UiTree, UiTemplateBuildError> {
        let mut builder = Self { next_node_id: 1 };
        let mut tree = UiTree::new(tree_id);
        builder.insert_node(&mut tree, None, &instance.root, "root")?;
        Ok(tree)
    }

    fn insert_node(
        &mut self,
        tree: &mut UiTree,
        parent_id: Option<UiNodeId>,
        node: &UiTemplateNode,
        path: &str,
    ) -> Result<UiNodeId, UiTemplateBuildError> {
        let node_id = UiNodeId::new(self.next_node_id);
        self.next_node_id += 1;

        let parent_container = parent_id
            .and_then(|parent_id| tree.node(parent_id))
            .map(|parent| parent.container);
        let (state_flags, input_policy) = infer_interaction(node);
        let layout = infer_layout_contract(node, path, parent_container)?;
        let container = layout
            .container
            .unwrap_or_else(|| infer_container(node.component.as_deref().unwrap_or_default()));
        let mut tree_node = UiTreeNode::new(node_id, UiNodePath::new(path.to_string()))
            .with_state_flags(state_flags)
            .with_constraints(layout.constraints)
            .with_anchor(layout.anchor)
            .with_pivot(layout.pivot)
            .with_position(layout.position)
            .with_input_policy(layout.input_policy.unwrap_or(input_policy))
            .with_layout_boundary(layout.layout_boundary)
            .with_layout_stretch_axes(layout.stretch_width, layout.stretch_height)
            .with_z_index(layout.z_index)
            .with_container(container)
            .with_clip_to_bounds(layout.clip_to_bounds || container.clips_to_bounds())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: node.component.clone().unwrap_or_default(),
                control_id: node.control_id.clone(),
                classes: node.classes.clone(),
                attributes: node.attributes.clone(),
                slot_attributes: node.slot_attributes.clone(),
                style_overrides: node.style_overrides.clone(),
                style_tokens: node.style_tokens.clone(),
                bindings: node.bindings.clone(),
            });
        if container.is_scrollable() {
            tree_node = tree_node.with_scroll_state(UiScrollState::default());
        }

        if let Some(parent_id) = parent_id {
            tree.insert_child(parent_id, tree_node)?;
        } else {
            tree.insert_root(tree_node);
        }

        for (index, child) in node.children.iter().enumerate() {
            let child_path = format!("{path}/{}", child_segment(child, index));
            self.insert_node(tree, Some(node_id), child, &child_path)?;
        }

        Ok(node_id)
    }
}
