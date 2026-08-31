use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiPixelSnappingPolicy, surface::UiRenderCommand, tree::UiTree,
};

pub(super) fn apply_resolved_pixel_snapping_policies(
    tree: &UiTree,
    commands: &mut [UiRenderCommand],
) -> usize {
    let mut resolved = BTreeMap::new();
    let mut unresolved_path = Vec::new();
    let mut visited_node_count = 0;

    for command in commands.iter_mut() {
        command.style.pixel_snapping = resolve_command_policy(
            tree,
            command.node_id,
            &mut resolved,
            &mut unresolved_path,
            &mut visited_node_count,
        );
    }

    crate::profile_counter!(
        "runtime",
        "ui.render_extract.pixel_snapping_node_visit_count",
        visited_node_count
    );
    visited_node_count
}

fn resolve_command_policy(
    tree: &UiTree,
    node_id: UiNodeId,
    resolved: &mut BTreeMap<UiNodeId, UiPixelSnappingPolicy>,
    unresolved_path: &mut Vec<(UiNodeId, UiPixelSnappingPolicy)>,
    visited_node_count: &mut usize,
) -> UiPixelSnappingPolicy {
    if let Some(policy) = resolved.get(&node_id).copied() {
        return policy;
    }

    unresolved_path.clear();
    let mut current = Some(node_id);
    let mut parent_policy = UiPixelSnappingPolicy::Inherit;
    let mut hop_count = 0;
    while let Some(current_id) = current {
        if let Some(policy) = resolved.get(&current_id).copied() {
            parent_policy = policy;
            break;
        }
        if hop_count > tree.nodes.len() {
            unresolved_path.clear();
            return UiPixelSnappingPolicy::Inherit;
        }
        hop_count += 1;

        let Some(node) = tree.node(current_id) else {
            break;
        };
        *visited_node_count += 1;
        let authored = node
            .template_metadata
            .as_ref()
            .map(|metadata| metadata.pixel_snapping)
            .unwrap_or_default();
        unresolved_path.push((current_id, authored));
        current = node.parent;
    }

    while let Some((current_id, authored)) = unresolved_path.pop() {
        parent_policy = authored.inherit_from(parent_policy);
        resolved.insert(current_id, parent_policy);
    }
    resolved.get(&node_id).copied().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::apply_resolved_pixel_snapping_policies;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::UiPixelSnappingPolicy,
        surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
        tree::{UiTemplateNodeMetadata, UiTree, UiTreeNode},
    };

    #[test]
    fn single_command_does_not_visit_unrelated_siblings() {
        let root_id = UiNodeId::new(1);
        let target_id = UiNodeId::new(10_001);
        let mut tree = UiTree::new(UiTreeId::new("render.extract.pixel-snapping.local"));
        tree.insert_root(node(root_id, UiPixelSnappingPolicy::Disabled));
        for value in 2..=10_001 {
            tree.insert_child(
                root_id,
                node(UiNodeId::new(value), UiPixelSnappingPolicy::Inherit),
            )
            .expect("insert pixel-snapping sibling");
        }
        let mut commands = vec![command(target_id)];

        let visited_node_count = apply_resolved_pixel_snapping_policies(&tree, &mut commands);

        assert_eq!(visited_node_count, 2);
        assert_eq!(
            commands[0].style.pixel_snapping,
            UiPixelSnappingPolicy::Disabled
        );
    }

    #[test]
    fn nearest_explicit_policy_wins_along_the_command_ancestor_path() {
        let root_id = UiNodeId::new(1);
        let parent_id = UiNodeId::new(2);
        let target_id = UiNodeId::new(3);
        let mut tree = UiTree::new(UiTreeId::new("render.extract.pixel-snapping.inherit"));
        tree.insert_root(node(root_id, UiPixelSnappingPolicy::Disabled));
        tree.insert_child(root_id, node(parent_id, UiPixelSnappingPolicy::SnapToPixel))
            .expect("insert explicit pixel-snapping parent");
        tree.insert_child(parent_id, node(target_id, UiPixelSnappingPolicy::Inherit))
            .expect("insert inherited pixel-snapping target");
        let mut commands = vec![command(target_id)];

        let visited_node_count = apply_resolved_pixel_snapping_policies(&tree, &mut commands);

        assert_eq!(visited_node_count, 3);
        assert_eq!(
            commands[0].style.pixel_snapping,
            UiPixelSnappingPolicy::SnapToPixel
        );
    }

    fn node(node_id: UiNodeId, policy: UiPixelSnappingPolicy) -> UiTreeNode {
        UiTreeNode::new(
            node_id,
            UiNodePath::new(format!("pixel-snapping/{}", node_id.0)),
        )
        .with_template_metadata(UiTemplateNodeMetadata {
            pixel_snapping: policy,
            ..UiTemplateNodeMetadata::default()
        })
    }

    fn command(node_id: UiNodeId) -> UiRenderCommand {
        UiRenderCommand {
            node_id,
            kind: UiRenderCommandKind::default(),
            frame: Default::default(),
            clip_frame: None,
            z_index: 0,
            style: UiResolvedStyle::default(),
            text_layout: None,
            text: None,
            image: None,
            opacity: 1.0,
        }
    }
}
