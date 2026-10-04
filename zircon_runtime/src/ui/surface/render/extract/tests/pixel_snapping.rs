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
