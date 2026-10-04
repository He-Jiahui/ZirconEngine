use super::*;

const DERIVED_STATE_MEDIUM_NODE_COUNT: usize = 1_000;
const DERIVED_STATE_LARGE_NODE_COUNT: usize = 100_000;

fn direct_child_hierarchy_world(node_count: usize) -> World {
    assert!(node_count > 0, "a hierarchy baseline needs a root entity");

    let mut world = World::empty();
    let root = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    for _ in 1..node_count {
        let child = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        world.set_parent_checked(child, Some(root)).unwrap();
    }
    world
}

fn assert_full_derived_state_baseline(node_count: usize) {
    let mut world = direct_child_hierarchy_world(node_count);
    world.reset_ecs_frame_performance_diagnostics();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
    let node_count = node_count as u64;
    assert_eq!(diagnostics.hierarchy_parent_snapshot_entities, node_count);
    assert_eq!(diagnostics.hierarchy_validity_entities, node_count);
    assert_eq!(
        diagnostics.hierarchy_parent_chain_steps,
        node_count.saturating_sub(1)
    );
    assert_eq!(diagnostics.hierarchy_topology_rebuild_entities, 0);
    assert_eq!(diagnostics.active_propagation_entities, node_count);
    assert_eq!(diagnostics.active_propagation_written_entities, node_count);
    assert_eq!(diagnostics.world_matrix_propagation_entities, node_count);
    assert_eq!(
        diagnostics.world_matrix_propagation_written_entities,
        node_count
    );
    assert_eq!(diagnostics.node_cache_rebuilt_entities, node_count);
}

#[test]
fn derived_state_full_rebuild_baseline_is_deterministic_for_one_and_one_thousand_nodes() {
    assert_full_derived_state_baseline(1);
    assert_full_derived_state_baseline(DERIVED_STATE_MEDIUM_NODE_COUNT);
}

#[test]
fn derived_state_full_rebuild_baseline_is_deterministic_for_one_hundred_thousand_nodes() {
    assert_full_derived_state_baseline(DERIVED_STATE_LARGE_NODE_COUNT);
}

#[test]
fn derived_state_leaf_transform_change_rebuilds_only_affected_rows() {
    let mut world = direct_child_hierarchy_world(DERIVED_STATE_MEDIUM_NODE_COUNT);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let changed = world.nodes().last().unwrap().id;

    world.reset_ecs_frame_performance_diagnostics();
    world
        .update_transform(
            changed,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
    assert_eq!(diagnostics.hierarchy_parent_snapshot_entities, 0);
    assert_eq!(diagnostics.hierarchy_validity_entities, 0);
    assert_eq!(diagnostics.active_propagation_entities, 0);
    assert_eq!(diagnostics.world_matrix_propagation_entities, 1);
    assert_eq!(diagnostics.world_matrix_propagation_written_entities, 1);
    assert_eq!(diagnostics.node_cache_rebuilt_entities, 1);
    assert_eq!(
        world.world_transform(changed).unwrap().translation,
        Vec3::new(1.0, 0.0, 0.0)
    );
}

#[test]
fn derived_state_parent_transform_change_rebuilds_the_affected_subtree() {
    let mut world = direct_child_hierarchy_world(DERIVED_STATE_MEDIUM_NODE_COUNT);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let root = world.nodes().first().unwrap().id;

    world.reset_ecs_frame_performance_diagnostics();
    world
        .update_transform(root, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))
        .unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
    assert_eq!(
        diagnostics.world_matrix_propagation_entities,
        DERIVED_STATE_MEDIUM_NODE_COUNT as u64
    );
    assert_eq!(
        diagnostics.world_matrix_propagation_written_entities,
        DERIVED_STATE_MEDIUM_NODE_COUNT as u64
    );
    assert_eq!(diagnostics.node_cache_rebuilt_entities, 1);
}

#[test]
fn derived_state_leaf_active_change_rebuilds_only_the_affected_row() {
    let mut world = direct_child_hierarchy_world(DERIVED_STATE_MEDIUM_NODE_COUNT);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    let changed = world.nodes().last().unwrap().id;

    world.reset_ecs_frame_performance_diagnostics();
    assert!(world.set_active_self(changed, false).unwrap());
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
    assert_eq!(diagnostics.hierarchy_parent_snapshot_entities, 0);
    assert_eq!(diagnostics.hierarchy_validity_entities, 0);
    assert_eq!(diagnostics.active_propagation_entities, 1);
    assert_eq!(diagnostics.active_propagation_written_entities, 1);
    assert_eq!(diagnostics.world_matrix_propagation_entities, 0);
    assert_eq!(diagnostics.node_cache_rebuilt_entities, 1);
    assert_eq!(world.active_in_hierarchy(changed), Some(false));
}

fn assert_structured_reparent_avoids_global_hierarchy_work(node_count: usize) {
    assert!(
        node_count >= 4,
        "reparent scale fixture needs two roots and a subtree"
    );

    let mut world = World::empty();
    let first_parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let second_parent = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let changed = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let changed_descendant = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_parent_checked(changed, Some(first_parent))
        .unwrap();
    world
        .set_parent_checked(changed_descendant, Some(changed))
        .unwrap();
    for _ in 4..node_count {
        let unrelated = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        world
            .set_parent_checked(unrelated, Some(first_parent))
            .unwrap();
    }
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world.reset_ecs_frame_performance_diagnostics();
    assert!(world
        .set_parent_checked(changed, Some(second_parent))
        .unwrap());
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let diagnostics = world.ecs_frame_performance_diagnostics().derived_state;
    assert_eq!(diagnostics.hierarchy_parent_snapshot_entities, 0);
    assert_eq!(diagnostics.hierarchy_validity_entities, 0);
    assert_eq!(diagnostics.hierarchy_topology_rebuild_entities, 0);
    assert_eq!(diagnostics.active_propagation_entities, 2);
    assert_eq!(diagnostics.active_propagation_written_entities, 0);
    assert_eq!(diagnostics.world_matrix_propagation_entities, 2);
    assert_eq!(diagnostics.world_matrix_propagation_written_entities, 0);
    assert_eq!(diagnostics.node_cache_rebuilt_entities, 1);
    assert_eq!(
        world.find_node(changed).unwrap().parent,
        Some(second_parent)
    );
    assert_eq!(
        world.find_node(changed_descendant).unwrap().parent,
        Some(changed)
    );
}

#[test]
fn derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_thousand_nodes() {
    assert_structured_reparent_avoids_global_hierarchy_work(DERIVED_STATE_MEDIUM_NODE_COUNT);
}

#[test]
fn derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_hundred_thousand_nodes() {
    assert_structured_reparent_avoids_global_hierarchy_work(DERIVED_STATE_LARGE_NODE_COUNT);
}
