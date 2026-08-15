use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, StretchMode, UiSize},
    tree::{UiDirtyFlags, UiTreeNode},
};

#[test]
fn surface_merges_reentrant_dirty_marks_into_one_committed_node() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    let node_id = UiNodeId::new(2);

    surface
        .mark_node_dirty(
            node_id,
            UiDirtyFlags {
                render: true,
                ..Default::default()
            },
        )
        .unwrap();
    surface
        .mark_node_dirty(
            node_id,
            UiDirtyFlags {
                hit_test: true,
                input: true,
                ..Default::default()
            },
        )
        .unwrap();

    assert_eq!(surface.pending_invalidation_changed_node_count(), 1);
    surface.rebuild_dirty(root_size()).unwrap();

    let commit = surface
        .last_invalidation_commit()
        .expect("dirty rebuild should publish invalidation");
    assert_eq!(commit.generation, baseline.generation + 1);
    assert_eq!(commit.changed_nodes.len(), 1);
    assert!(commit.changed_nodes[0].dirty.hit_test);
    assert!(commit.changed_nodes[0].dirty.render);
    assert!(commit.changed_nodes[0].dirty.input);
}

#[test]
fn unchanged_property_and_stable_rebuild_do_not_advance_generation() {
    let mut surface = test_surface();
    let baseline = surface.invalidation_generations();
    let request =
        || UiPropertyMutationRequest::new(UiNodeId::new(2), "pressed", UiValue::Bool(true));

    assert_eq!(
        surface.mutate_property(request()).unwrap().status,
        UiPropertyMutationStatus::Accepted
    );
    surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation + 1
    );

    assert_eq!(
        surface.mutate_property(request()).unwrap().status,
        UiPropertyMutationStatus::Unchanged
    );
    surface.rebuild_dirty(root_size()).unwrap();
    assert_eq!(
        surface.invalidation_generations().generation,
        baseline.generation + 1
    );
    assert_eq!(surface.pending_invalidation_changed_node_count(), 0);
}

fn test_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.invalidation"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: fixed_constraint(120.0),
                height: fixed_constraint(60.0),
            },
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button")).with_constraints(
                BoxConstraints {
                    width: fixed_constraint(80.0),
                    height: fixed_constraint(24.0),
                },
            ),
        )
        .unwrap();
    surface.compute_layout(root_size()).unwrap();
    surface.clear_dirty_flags();
    surface
}

fn fixed_constraint(value: f32) -> AxisConstraint {
    AxisConstraint {
        min: value,
        max: value,
        preferred: value,
        stretch_mode: StretchMode::Fixed,
        ..Default::default()
    }
}

fn root_size() -> UiSize {
    UiSize {
        width: 120.0,
        height: 60.0,
    }
}
