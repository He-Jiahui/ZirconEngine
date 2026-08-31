use std::collections::BTreeSet;

use super::*;
use crate::ui::surface::{arranged_node_indices, build_arranged_tree};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiStateFlags, UiTreeId},
    navigation::{
        UiDirectionalNavigation, UiDirectionalNavigationTarget, UiNavigationContract,
        UiNavigationGroup,
    },
    surface::UiHitTestEntry,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::UiWidgetContract,
};

#[test]
fn published_hit_geometry_is_the_directional_navigation_authority() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.projected.geometry"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(id(1), focus_node(2, UiFrame::new(0.0, 0.0, 20.0, 20.0)))
        .unwrap();
    tree.insert_child(id(1), focus_node(3, UiFrame::new(100.0, 0.0, 20.0, 20.0)))
        .unwrap();
    tree.insert_child(id(1), focus_node(4, UiFrame::new(0.0, 100.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let published_grid = UiHitTestGrid {
        entries: vec![
            hit_entry(2, UiFrame::new(0.0, 0.0, 20.0, 20.0), 1),
            hit_entry(3, UiFrame::new(0.0, 100.0, 20.0, 20.0), 2),
            hit_entry(4, UiFrame::new(100.0, 0.0, 20.0, 20.0), 3),
        ]
        .into(),
        ..UiHitTestGrid::default()
    };
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &published_grid,
    );
    let build_generation = index.build_generation();

    assert_eq!(
        index
            .next_navigation_target(Some(id(2)), UiNavigationEventKind::Right)
            .unwrap(),
        Some(id(4)),
        "directional navigation must use final published geometry, not tree placeholders",
    );
    assert_eq!(index.build_generation(), build_generation);
}

#[test]
fn prebuilt_tab_scopes_preserve_base_and_modal_wrap_order() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.modal.tab"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(
        id(1),
        focus_node(2, UiFrame::new(0.0, 0.0, 20.0, 20.0)).with_navigation_contract(tab_contract(2)),
    )
    .unwrap();
    tree.insert_child(
        id(1),
        focus_node(3, UiFrame::new(30.0, 0.0, 20.0, 20.0))
            .with_navigation_contract(tab_contract(1)),
    )
    .unwrap();
    tree.insert_child(
        id(1),
        focus_node(5, UiFrame::new(0.0, 60.0, 20.0, 20.0))
            .with_navigation_contract(modal_tab_contract(1)),
    )
    .unwrap();
    tree.insert_child(
        id(5),
        focus_node(6, UiFrame::new(30.0, 60.0, 20.0, 20.0))
            .with_navigation_contract(modal_tab_contract(2)),
    )
    .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );
    let build_generation = index.build_generation();

    assert_eq!(
        index
            .next_navigation_target(Some(id(2)), UiNavigationEventKind::Next)
            .unwrap(),
        Some(id(3)),
    );
    assert_eq!(
        index
            .next_navigation_target(Some(id(5)), UiNavigationEventKind::Next)
            .unwrap(),
        Some(id(6)),
    );
    assert_eq!(
        index
            .next_navigation_target(Some(id(6)), UiNavigationEventKind::Next)
            .unwrap(),
        Some(id(5)),
    );
    assert_eq!(index.build_generation(), build_generation);
}

#[test]
fn input_semantics_gate_skips_pointer_only_changes_but_keeps_focus_changes() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.input-semantic-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(id(1), focus_node(2, UiFrame::new(0.0, 0.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );

    let changed = BTreeSet::from([id(2)]);
    tree.node_mut(id(2)).unwrap().input_policy = UiInputPolicy::Ignore;
    assert!(!needs_semantics_rebuild(&index, &tree, &changed));

    tree.node_mut(id(2)).unwrap().state_flags.focusable = false;
    assert!(needs_semantics_rebuild(&index, &tree, &changed));
}

#[test]
fn retained_semantics_gate_skips_text_only_changes_but_keeps_local_navigation_changes() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.retained-semantic-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(id(1), focus_node(2, UiFrame::new(0.0, 0.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );
    let changed = BTreeSet::from([id(2)]);

    assert!(!needs_semantics_rebuild(&index, &tree, &changed));

    tree.node_mut(id(2)).unwrap().navigation.tab_index = Some(UiTabIndex::new(7));
    assert!(needs_semantics_rebuild(&index, &tree, &changed));

    tree.node_mut(id(2)).unwrap().navigation.tab_index = None;
    tree.node_mut(id(2)).unwrap().navigation.directional = Some(UiDirectionalNavigation {
        right: UiDirectionalNavigationTarget::Blocked,
        ..UiDirectionalNavigation::default()
    });
    assert!(needs_semantics_rebuild(&index, &tree, &changed));
}

#[test]
fn retained_semantics_gate_detects_navigation_group_owner_changes() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.retained-group-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0))
            .with_navigation_contract(UiNavigationContract {
                group: Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("toolbar"),
                    order: 1,
                    ..UiNavigationGroup::default()
                }),
                ..UiNavigationContract::default()
            }),
    );
    tree.insert_child(id(1), focus_node(2, UiFrame::new(0.0, 0.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );
    let changed = BTreeSet::from([id(1)]);

    assert!(!needs_semantics_rebuild(&index, &tree, &changed));
    tree.node_mut(id(1))
        .unwrap()
        .navigation
        .group
        .as_mut()
        .unwrap()
        .order = 2;
    assert!(needs_semantics_rebuild(&index, &tree, &changed));
}

#[test]
fn retained_semantics_gate_detects_mui_modal_activation_changes() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.retained-modal-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(
        id(1),
        UiTreeNode::new(id(2), UiNodePath::new("root/modal"))
            .with_frame(UiFrame::new(0.0, 0.0, 120.0, 80.0))
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Modal".to_string(),
                attributes: [("open".to_string(), toml::Value::Boolean(true))]
                    .into_iter()
                    .collect(),
                widget: UiWidgetContract {
                    open_property: Some("open".to_string()),
                    ..UiWidgetContract::default()
                },
                ..UiTemplateNodeMetadata::default()
            }),
    )
    .unwrap();
    tree.insert_child(id(2), focus_node(3, UiFrame::new(8.0, 8.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );
    let changed = BTreeSet::from([id(2)]);

    assert!(!needs_semantics_rebuild(&index, &tree, &changed));
    tree.node_mut(id(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("open".to_string(), toml::Value::Boolean(false));
    assert!(needs_semantics_rebuild(&index, &tree, &changed));
}

#[test]
fn retained_semantics_gate_only_rebuilds_for_removed_navigation_participants() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.retained-removal-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(
        id(1),
        UiTreeNode::new(id(2), UiNodePath::new("root/label"))
            .with_frame(UiFrame::new(0.0, 0.0, 20.0, 20.0)),
    )
    .unwrap();
    tree.insert_child(id(1), focus_node(3, UiFrame::new(30.0, 0.0, 20.0, 20.0)))
        .unwrap();
    tree.insert_child(
        id(1),
        UiTreeNode::new(id(4), UiNodePath::new("root/container"))
            .with_frame(UiFrame::new(60.0, 0.0, 40.0, 40.0)),
    )
    .unwrap();
    tree.insert_child(id(4), focus_node(5, UiFrame::new(64.0, 4.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );

    assert!(!needs_semantics_rebuild_with_removed(
        &index,
        &tree,
        &BTreeSet::from([id(2)]),
    ));
    assert!(needs_semantics_rebuild_with_removed(
        &index,
        &tree,
        &BTreeSet::from([id(3)]),
    ));
    assert!(needs_semantics_rebuild_with_removed(
        &index,
        &tree,
        &BTreeSet::from([id(4)]),
    ));
}

#[test]
fn retained_semantics_gate_detects_new_navigation_subtrees_from_the_changed_root() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.retained-insert-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );

    tree.insert_child(
        id(1),
        UiTreeNode::new(id(2), UiNodePath::new("root/new-container"))
            .with_frame(UiFrame::new(0.0, 0.0, 40.0, 40.0)),
    )
    .unwrap();
    tree.insert_child(id(2), focus_node(3, UiFrame::new(4.0, 4.0, 20.0, 20.0)))
        .unwrap();

    assert!(needs_semantics_rebuild(
        &index,
        &tree,
        &BTreeSet::from([id(2)]),
    ));
}

#[test]
fn geometry_patch_skips_non_candidates_and_updates_focus_candidate_frames() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.geometry-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(
        id(1),
        UiTreeNode::new(id(2), UiNodePath::new("root/non-candidate"))
            .with_frame(UiFrame::new(0.0, 0.0, 20.0, 20.0)),
    )
    .unwrap();
    tree.insert_child(id(1), focus_node(3, UiFrame::new(30.0, 0.0, 20.0, 20.0)))
        .unwrap();
    tree.insert_child(id(1), focus_node(4, UiFrame::new(45.0, 0.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
    );
    assert_eq!(
        index
            .next_navigation_target(Some(id(2)), UiNavigationEventKind::Right)
            .unwrap(),
        Some(id(3)),
    );

    tree.node_mut(id(2)).unwrap().layout_cache.frame = UiFrame::new(5.0, 0.0, 20.0, 20.0);
    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    assert_eq!(
        index.patch_changed_geometry(
            &tree,
            &arranged_tree,
            &arranged_node_indices,
            &UiProjectedHitTestIndex::default(),
            &UiHitTestGrid::default(),
            &BTreeSet::from([id(2)]),
            &BTreeSet::new(),
        ),
        Ok(0),
    );

    let next_frame = UiFrame::new(60.0, 0.0, 20.0, 20.0);
    tree.node_mut(id(3)).unwrap().layout_cache.frame = next_frame;
    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    assert_eq!(
        index.patch_changed_geometry(
            &tree,
            &arranged_tree,
            &arranged_node_indices,
            &UiProjectedHitTestIndex::default(),
            &UiHitTestGrid::default(),
            &BTreeSet::from([id(3)]),
            &BTreeSet::new(),
        ),
        Ok(1),
    );
    assert_eq!(index.nodes.get(&id(3)).unwrap().frame, next_frame);
    assert_eq!(
        index
            .next_navigation_target(Some(id(2)), UiNavigationEventKind::Right)
            .unwrap(),
        Some(id(4)),
        "directional queries must observe the patched candidate frame",
    );

    tree.node_mut(id(3)).unwrap().z_index = 1;
    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    assert!(
        index
            .patch_changed_geometry(
                &tree,
                &arranged_tree,
                &arranged_node_indices,
                &UiProjectedHitTestIndex::default(),
                &UiHitTestGrid::default(),
                &BTreeSet::from([id(3)]),
                &BTreeSet::new(),
            )
            .is_err()
    );
}

#[test]
fn projected_geometry_patch_updates_candidate_frames_but_rebuilds_for_order_changes() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.projected-geometry-gate"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(
        id(1),
        UiTreeNode::new(id(2), UiNodePath::new("root/non-candidate"))
            .with_frame(UiFrame::new(0.0, 0.0, 20.0, 20.0)),
    )
    .unwrap();
    tree.insert_child(id(1), focus_node(3, UiFrame::new(30.0, 0.0, 20.0, 20.0)))
        .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut published_grid = UiHitTestGrid {
        entries: vec![
            hit_entry(2, UiFrame::new(0.0, 0.0, 20.0, 20.0), 1),
            hit_entry(3, UiFrame::new(30.0, 0.0, 20.0, 20.0), 2),
        ]
        .into(),
        ..UiHitTestGrid::default()
    };
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &published_grid,
    );

    published_grid.entries[0].frame = UiFrame::new(5.0, 0.0, 20.0, 20.0);
    assert_eq!(
        index.patch_projected_geometry(
            &tree,
            &arranged_tree,
            &arranged_node_indices,
            &UiProjectedHitTestIndex::default(),
            &published_grid,
        ),
        Ok(0),
    );

    published_grid.entries[1].frame = UiFrame::new(60.0, 0.0, 20.0, 20.0);
    assert_eq!(
        index.patch_projected_geometry(
            &tree,
            &arranged_tree,
            &arranged_node_indices,
            &UiProjectedHitTestIndex::default(),
            &published_grid,
        ),
        Ok(1),
    );
    assert_eq!(
        index.nodes.get(&id(3)).unwrap().frame,
        UiFrame::new(60.0, 0.0, 20.0, 20.0),
    );

    published_grid.entries[1].paint_order = 7;
    assert!(
        index
            .patch_projected_geometry(
                &tree,
                &arranged_tree,
                &arranged_node_indices,
                &UiProjectedHitTestIndex::default(),
                &published_grid,
            )
            .is_err()
    );
}

#[test]
fn externally_referenced_modal_root_patches_frames_but_rebuilds_for_order_changes() {
    let mut tree = UiTree::new(UiTreeId::new("navigation.external-modal-root-geometry"));
    tree.insert_root(
        UiTreeNode::new(id(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 160.0)),
    );
    tree.insert_child(
        id(1),
        UiTreeNode::new(id(2), UiNodePath::new("root/modal-root"))
            .with_frame(UiFrame::new(10.0, 10.0, 100.0, 80.0)),
    )
    .unwrap();
    tree.insert_child(
        id(1),
        focus_node(3, UiFrame::new(20.0, 20.0, 20.0, 20.0)).with_navigation_contract(
            UiNavigationContract {
                group: Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("external-modal-root"),
                    root: Some(id(2)),
                    modal: true,
                    ..UiNavigationGroup::default()
                }),
                ..UiNavigationContract::default()
            },
        ),
    )
    .unwrap();

    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    let mut published_grid = UiHitTestGrid {
        entries: vec![
            hit_entry(2, UiFrame::new(10.0, 10.0, 100.0, 80.0), 1),
            hit_entry(3, UiFrame::new(20.0, 20.0, 20.0, 20.0), 2),
        ]
        .into(),
        ..UiHitTestGrid::default()
    };
    let mut index = UiSurfaceNavigationIndex::default();
    index.rebuild(
        &tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &published_grid,
    );

    let local_frame = UiFrame::new(12.0, 10.0, 100.0, 80.0);
    tree.node_mut(id(2)).unwrap().layout_cache.frame = local_frame;
    let arranged_tree = build_arranged_tree(&tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    assert_eq!(
        index.patch_changed_geometry(
            &tree,
            &arranged_tree,
            &arranged_node_indices,
            &UiProjectedHitTestIndex::default(),
            &published_grid,
            &BTreeSet::from([id(2)]),
            &BTreeSet::new(),
        ),
        Ok(1),
    );

    let projected_frame = UiFrame::new(40.0, 30.0, 100.0, 80.0);
    published_grid.entries[0].frame = projected_frame;
    assert_eq!(
        index.patch_projected_geometry(
            &tree,
            &arranged_tree,
            &arranged_node_indices,
            &UiProjectedHitTestIndex::default(),
            &published_grid,
        ),
        Ok(1),
    );
    assert_eq!(index.nodes.get(&id(2)).unwrap().frame, projected_frame);

    published_grid.entries[0].paint_order = 9;
    assert!(
        index
            .patch_projected_geometry(
                &tree,
                &arranged_tree,
                &arranged_node_indices,
                &UiProjectedHitTestIndex::default(),
                &published_grid,
            )
            .is_err()
    );
}

fn focus_node(node_id: u64, frame: UiFrame) -> UiTreeNode {
    UiTreeNode::new(id(node_id), UiNodePath::new(format!("root/{node_id}")))
        .with_frame(frame)
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            focusable: true,
            ..UiStateFlags::default()
        })
}

fn hit_entry(node_id: u64, frame: UiFrame, paint_order: u64) -> UiHitTestEntry {
    UiHitTestEntry {
        node_id: id(node_id),
        frame,
        clip_frame: frame,
        z_index: 0,
        paint_order,
        control_id: None,
        route_node_index: node_id as u32,
    }
}

fn tab_contract(order: i32) -> UiNavigationContract {
    UiNavigationContract {
        tab_index: Some(UiTabIndex::new(order)),
        ..UiNavigationContract::default()
    }
}

fn modal_tab_contract(order: i32) -> UiNavigationContract {
    UiNavigationContract {
        tab_index: Some(UiTabIndex::new(order)),
        group: Some(UiNavigationGroup {
            group_id: UiNavigationGroupId::new("dialog"),
            root: Some(id(5)),
            modal: true,
            wrap: true,
            ..UiNavigationGroup::default()
        }),
        ..UiNavigationContract::default()
    }
}

fn needs_semantics_rebuild(
    index: &UiSurfaceNavigationIndex,
    tree: &UiTree,
    changed_node_ids: &BTreeSet<UiNodeId>,
) -> bool {
    let arranged_tree = build_arranged_tree(tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    index.needs_semantics_rebuild(
        tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
        changed_node_ids,
        &BTreeSet::new(),
    )
}

fn needs_semantics_rebuild_with_removed(
    index: &UiSurfaceNavigationIndex,
    tree: &UiTree,
    removed_node_ids: &BTreeSet<UiNodeId>,
) -> bool {
    let arranged_tree = build_arranged_tree(tree);
    let arranged_node_indices = arranged_node_indices(&arranged_tree);
    index.needs_semantics_rebuild(
        tree,
        &arranged_tree,
        &arranged_node_indices,
        &UiProjectedHitTestIndex::default(),
        &UiHitTestGrid::default(),
        &BTreeSet::new(),
        removed_node_ids,
    )
}

const fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}
