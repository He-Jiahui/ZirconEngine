use super::UiVirtualListPrototypePoolReport;
use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        UiContainerKind, UiScrollState, UiScrollableBoxConfig, UiSlot, UiSlotKind,
        UiVirtualListConfig,
    },
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn prototype_pool_clones_complete_subtree_for_each_physical_slot() {
    let mut surface = surface_with_prototype();
    reconcile(&mut surface, 100_000);

    let report = ensure(&mut surface);

    assert_eq!(report.slot_capacity, 3);
    assert_eq!(report.live_slot_count, 3);
    assert_eq!(report.created_slot_count, 2);
    assert_eq!(surface.tree.nodes.len(), 10);
    for root_id in report.slot_root_ids {
        let root = surface.tree.node(root_id).unwrap();
        assert_eq!(root.children.len(), 1);
        let child = surface.tree.node(root.children[0]).unwrap();
        assert_eq!(child.children.len(), 1);
        assert!(surface.tree.node(child.children[0]).is_some());
        assert!(surface
            .tree
            .layout_slots()
            .iter()
            .any(|slot| slot.parent_id == root.node_id && slot.child_id == child.node_id));
    }
}

#[test]
fn unchanged_capacity_preserves_slot_roots_without_new_nodes() {
    let mut surface = surface_with_prototype();
    reconcile(&mut surface, 100_000);
    let first = ensure(&mut surface);
    let node_count = surface.tree.nodes.len();

    let second = ensure(&mut surface);

    assert_eq!(second.slot_root_ids, first.slot_root_ids);
    assert_eq!(second.created_slot_count, 0);
    assert_eq!(second.created_node_count, 0);
    assert_eq!(surface.tree.nodes.len(), node_count);
}

#[test]
fn logical_count_does_not_change_physical_tree_size() {
    let mut small = surface_with_prototype();
    reconcile(&mut small, 100);
    ensure(&mut small);
    let mut large = surface_with_prototype();
    reconcile(&mut large, 100_000);
    ensure(&mut large);

    assert_eq!(small.tree.nodes.len(), large.tree.nodes.len());
    assert_eq!(
        small
            .virtual_list_prototype_slot_roots(owner_id())
            .unwrap()
            .len(),
        large
            .virtual_list_prototype_slot_roots(owner_id())
            .unwrap()
            .len()
    );
}

#[test]
fn shrinking_then_growing_reuses_bounded_subtrees() {
    let mut surface = surface_with_prototype();
    reconcile(&mut surface, 100_000);
    let first = ensure(&mut surface);
    let retained_root = first.slot_root_ids[0];

    reconcile(&mut surface, 1);
    let shrunk = ensure(&mut surface);
    assert_eq!(shrunk.live_slot_count, 1);
    assert_eq!(shrunk.removed_slot_count, 2);

    reconcile(&mut surface, 100_000);
    let grown = ensure(&mut surface);
    assert_eq!(grown.live_slot_count, 3);
    assert_eq!(grown.slot_root_ids[0], retained_root);
    assert_eq!(grown.created_slot_count, 2);
    assert_eq!(grown.reused_node_count, 6);
    assert_eq!(surface.tree.nodes.len(), 10);
}

fn ensure(surface: &mut UiSurface) -> UiVirtualListPrototypePoolReport {
    surface
        .ensure_virtual_list_prototype_slots(owner_id(), prototype_id(), |_, _| {})
        .unwrap()
}

fn reconcile(surface: &mut UiSurface, logical_count: usize) {
    surface
        .reconcile_virtual_list_materialization(owner_id(), logical_count, &mut Vec::new())
        .unwrap();
}

fn surface_with_prototype() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.virtual_list.prototype_pool"));
    surface.tree.insert_root(
        UiTreeNode::new(owner_id(), UiNodePath::new("root/list"))
            .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                virtualization: Some(UiVirtualListConfig {
                    item_extent: 24.0,
                    overscan: 0,
                }),
                ..UiScrollableBoxConfig::default()
            }))
            .with_scroll_state(UiScrollState {
                viewport_extent: 72.0,
                content_extent: 72.0,
                ..UiScrollState::default()
            }),
    );
    insert_template_child(&mut surface, owner_id(), prototype_id(), "row", "Row");
    insert_template_child(
        &mut surface,
        prototype_id(),
        UiNodeId::new(3),
        "row/label",
        "RowLabel",
    );
    insert_template_child(
        &mut surface,
        UiNodeId::new(3),
        UiNodeId::new(4),
        "row/icon",
        "RowIcon",
    );
    surface
}

fn insert_template_child(
    surface: &mut UiSurface,
    parent_id: UiNodeId,
    node_id: UiNodeId,
    path: &str,
    control_id: &str,
) {
    let mut metadata = UiTemplateNodeMetadata::default();
    metadata.component = "VirtualRow".to_string();
    metadata.control_id = Some(control_id.to_string());
    surface
        .tree
        .insert_child(
            parent_id,
            UiTreeNode::new(node_id, UiNodePath::new(path)).with_template_metadata(metadata),
        )
        .unwrap();
    surface
        .tree
        .push_layout_slot(UiSlot::new(parent_id, node_id, UiSlotKind::Linear));
}

const fn owner_id() -> UiNodeId {
    UiNodeId::new(1)
}

const fn prototype_id() -> UiNodeId {
    UiNodeId::new(2)
}
