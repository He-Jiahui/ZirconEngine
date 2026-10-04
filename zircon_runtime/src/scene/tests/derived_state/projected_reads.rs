//! 调用方可在内部系统刷新前读取 World；读取投影须立即反映父链变化，
//! 而已刷新的派生值仍以通用 ComponentStorage 为唯一存储所有者。

use std::sync::Arc;

use super::*;
use crate::core::framework::render::RenderComponentValue;
use crate::scene::components::{ActiveInHierarchy, WorldMatrix};

#[test]
fn derived_world_matrix_uses_component_storage_without_a_fixed_owner() {
    let mut world = World::new();
    let entity = world.active_camera();
    let projected = world
        .world_matrix(entity)
        .expect("active camera must have a derived world matrix");

    assert!(world.contains_component::<WorldMatrix>(entity));
    assert_eq!(
        world.get::<WorldMatrix>(entity).map(|matrix| matrix.0),
        Some(projected)
    );
    assert!(world.contains_component::<ActiveInHierarchy>(entity));
}

#[test]
fn world_clone_rebuilds_derived_component_storage() {
    let world = World::new();
    let entity = world.active_camera();
    let expected_matrix = world
        .world_matrix(entity)
        .expect("active camera must have a derived world matrix");

    let mut cloned = world.clone();
    cloned.flush_pending_scene_systems();

    assert!(cloned.contains_component::<WorldMatrix>(entity));
    assert!(cloned.contains_component::<ActiveInHierarchy>(entity));
    assert_eq!(
        cloned.get::<WorldMatrix>(entity).map(|matrix| matrix.0),
        Some(expected_matrix)
    );
    assert_eq!(
        cloned.active_in_hierarchy(entity),
        world.active_in_hierarchy(entity)
    );
}

#[test]
fn dirty_projected_reads_match_flushed_reparent_values() {
    let mut world = pending_reparented_world();
    let child = world
        .nodes()
        .last()
        .expect("reparented fixture must contain its child")
        .id;
    assert!(world.has_pending_scene_systems());

    let projected_matrix = world
        .world_matrix(child)
        .expect("dirty child must have a projected world matrix");
    let projected_transform = world
        .world_transform(child)
        .expect("dirty child must have a projected world transform");
    let projected_active = world
        .active_in_hierarchy(child)
        .expect("dirty child must have a projected active value");

    assert_eq!(projected_transform.translation, Vec3::new(12.0, 0.0, 0.0));
    assert!(!projected_active);
    assert!(world.has_pending_scene_systems());

    world.flush_pending_scene_systems();
    assert_eq!(world.world_matrix(child), Some(projected_matrix));
    assert_eq!(world.world_transform(child), Some(projected_transform));
    assert_eq!(world.active_in_hierarchy(child), Some(projected_active));
}

#[test]
fn equivalent_parent_reparent_preserves_derived_ticks_and_render_fields() {
    let mut world = World::empty();
    let first_parent = world
        .spawn_node(NodeKind::Cube)
        .expect("first parent should spawn");
    let second_parent = world
        .spawn_node(NodeKind::Cube)
        .expect("second parent should spawn");
    let child = world
        .spawn_node(NodeKind::Mesh)
        .expect("renderable child should spawn");
    let parent_transform = Transform::from_translation(Vec3::new(3.0, 0.0, 0.0));

    world
        .update_transform(first_parent, parent_transform)
        .expect("first parent transform should update");
    world
        .update_transform(second_parent, parent_transform)
        .expect("second parent transform should update");
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .expect("child transform should update");
    world
        .set_parent_checked(child, Some(first_parent))
        .expect("initial parent should attach");
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let expected_world_matrix = *world
        .get::<WorldMatrix>(child)
        .expect("child world matrix should be materialized");
    let expected_active = *world
        .get::<ActiveInHierarchy>(child)
        .expect("child active state should be materialized");
    let world_matrix_ticks = world
        .component_change_ticks::<WorldMatrix>(child)
        .expect("child world matrix should have change ticks");
    let active_ticks = world
        .component_change_ticks::<ActiveInHierarchy>(child)
        .expect("child active state should have change ticks");
    let before = world
        .render_component_change_artifact()
        .expect("renderable child should have an initial render projection");
    let before_child = before
        .upserts()
        .iter()
        .find(|snapshot| snapshot.entity() == child)
        .expect("initial render projection should include the child");
    assert_eq!(
        before_child.world_matrix(),
        &RenderComponentValue::Present(expected_world_matrix.0)
    );
    assert_eq!(
        before_child.active_in_hierarchy(),
        &RenderComponentValue::Present(expected_active.0)
    );
    assert!(expected_active.0);

    assert_eq!(
        world.world_matrix(first_parent),
        world.world_matrix(second_parent)
    );
    assert_eq!(world.active_in_hierarchy(first_parent), Some(true));
    assert_eq!(world.active_in_hierarchy(second_parent), Some(true));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(5.0, 0.0, 0.0)
    );
    assert!(world
        .set_parent_checked(child, Some(second_parent))
        .expect("equivalent parent reparent should succeed"));
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert_eq!(
        world.get::<WorldMatrix>(child),
        Some(&expected_world_matrix)
    );
    assert_eq!(
        world.get::<ActiveInHierarchy>(child),
        Some(&expected_active)
    );
    assert_eq!(world.active_in_hierarchy(child), Some(true));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(5.0, 0.0, 0.0)
    );
    assert_eq!(
        world.component_change_ticks::<WorldMatrix>(child),
        Some(world_matrix_ticks)
    );
    assert_eq!(
        world.component_change_ticks::<ActiveInHierarchy>(child),
        Some(active_ticks)
    );

    let after = world
        .render_component_change_artifact()
        .expect("render component projection should remain available");
    assert!(Arc::ptr_eq(&before, &after));
    assert_eq!(after.journal_generation(), before.journal_generation());
    let after_child = after
        .upserts()
        .iter()
        .find(|snapshot| snapshot.entity() == child)
        .expect("unchanged render artifact should retain the child fields");
    assert_eq!(
        after_child.world_matrix(),
        &RenderComponentValue::Present(expected_world_matrix.0)
    );
    assert_eq!(
        after_child.active_in_hierarchy(),
        &RenderComponentValue::Present(expected_active.0)
    );
}

#[test]
fn dirty_projected_reads_stream_a_deep_chain_before_and_after_flush() {
    const CHAIN_DEPTH: usize = 128;

    let mut world = World::empty();
    let mut records = Vec::with_capacity(CHAIN_DEPTH);
    for index in 0..CHAIN_DEPTH {
        let id = 100_000 + index as u64;
        let mut record = detached_node_record(id, NodeKind::Empty);
        record.parent = index.checked_sub(1).map(|parent| 100_000 + parent as u64);
        record.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
        record.active = index != 0;
        records.push(record);
    }
    let leaf = records.last().expect("deep fixture must have a leaf").id;
    world
        .insert_node_records(&records)
        .expect("deep fixture records should publish");
    assert!(world.has_pending_scene_systems());

    let projected_transform = world
        .world_transform(leaf)
        .expect("deep dirty leaf must have a projected transform");
    let projected_active = world
        .active_in_hierarchy(leaf)
        .expect("deep dirty leaf must have a projected active value");
    assert_eq!(
        projected_transform.translation,
        Vec3::new(CHAIN_DEPTH as f32, 0.0, 0.0)
    );
    assert!(!projected_active);
    assert!(world.has_pending_scene_systems());

    world.flush_pending_scene_systems();
    assert_eq!(world.world_transform(leaf), Some(projected_transform));
    assert_eq!(world.active_in_hierarchy(leaf), Some(projected_active));
}

#[test]
fn dirty_projected_reads_stop_at_self_and_missing_parent_edges() {
    for self_parent in [true, false] {
        let mut world = World::empty();
        let target = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        let other = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        world.flush_pending_scene_systems();

        world
            .update_transform(
                target,
                Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)),
            )
            .expect("target transform should update");
        world
            .set_active_self(other, false)
            .expect("other node should become inactive");
        let parent = if self_parent { target } else { u64::MAX };
        assert!(world.corrupt_hierarchy_parent_for_tests(target, Some(parent)));

        assert_eq!(
            world
                .world_transform(target)
                .map(|transform| transform.translation),
            Some(Vec3::new(4.0, 0.0, 0.0))
        );
        assert_eq!(world.active_in_hierarchy(target), Some(true));
    }
}

#[test]
fn dirty_projected_reads_fail_closed_on_multi_node_parent_cycles() {
    let mut world = World::empty();
    let first = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let second = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let unrelated = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.flush_pending_scene_systems();

    world
        .update_transform(first, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))
        .expect("cycle member transform should update");
    world
        .set_active_self(unrelated, false)
        .expect("unrelated node should become inactive");
    assert!(world.corrupt_hierarchy_parent_for_tests(first, Some(second)));
    assert!(world.corrupt_hierarchy_parent_for_tests(second, Some(first)));

    assert_eq!(world.world_matrix(first), None);
    assert_eq!(world.active_in_hierarchy(first), Some(false));
}

#[test]
fn derived_state_projected_value_reads_use_direct_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let active_read = source
        .split("pub(super) fn project_active_in_hierarchy_for_read")
        .nth(1)
        .and_then(|text| text.split("#[cfg(test)]").next())
        .expect("read project_active_in_hierarchy_for_read body");
    let world_transform = source
        .split("pub(super) fn project_world_transform")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn project_node_for_read").next())
        .expect("read project_world_transform body");
    let world_owner = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("world.rs"),
    );
    let fixed_owner = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("fixed_components.rs"),
    );

    assert!(
        active_read.contains("let Some(active) = self.get::<ActiveInHierarchy>(entity) else")
            && active_read.contains("return Some(active.0);")
            && active_read.contains("if !self.contains_entity(entity)")
            && active_read.contains("Some(self.active_self_chain_value(entity))")
            && !active_read.contains(".map(|active| active.0)")
            && !active_read.contains(".then(||"),
        "projected active reads must branch directly for cached and dirty paths"
    );
    assert!(
        world_transform.contains("let Some(world) = self.get::<WorldMatrix>(entity) else")
            && world_transform.contains("return Some(matrix_to_transform(world.0));")
            && world_transform.contains(
                "let Some(world_matrix) = self.project_world_matrix_for_read(entity) else"
            )
            && world_transform.contains("Some(matrix_to_transform(world_matrix))")
            && !world_transform.contains(".map(|world| matrix_to_transform(world.0))")
            && !world_transform.contains(".map(matrix_to_transform)"),
        "projected world-transform reads must branch directly for cached and dirty paths"
    );
    let fixed_validation = fixed_owner
        .split("pub(super) fn validate_fixed_component")
        .nth(1)
        .expect("read fixed component validation dispatch");

    assert!(
        !source.contains("self.world_matrices")
            && !world_owner.contains("world_matrices:")
            && !world_owner.contains("active_in_hierarchy:")
            && !fixed_owner.contains("world_matrices")
            && !fixed_owner.contains("active_in_hierarchy")
            && !fixed_validation.contains("WorldMatrix")
            && !fixed_validation.contains("ActiveInHierarchy"),
        "derived components must have ComponentStorage as their only body owner instead of restoring fixed-component maps or dispatch branches"
    );
}

#[test]
fn derived_state_default_component_reads_use_direct_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let project_node = source
        .split("pub(super) fn project_node_for_read")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn project_world_matrix_for_read")
                .next()
        })
        .expect("read project_node_for_read body");
    let propagate_world = source
        .split("fn propagate_world_matrix")
        .nth(1)
        .and_then(|text| text.split("fn hierarchy_traversal_index").next())
        .expect("read propagate_world_matrix body");
    let local_value = source
        .split("fn local_transform_value")
        .nth(1)
        .and_then(|text| text.split("fn active_self_value").next())
        .expect("read local_transform_value body");
    let active_value = source
        .split("fn active_self_value")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn refresh_node_cache").next())
        .expect("read active_self_value body");
    let refresh = source
        .split("pub(super) fn refresh_node_cache")
        .nth(1)
        .and_then(|text| text.split("fn prepare_render_extract").next())
        .expect("read refresh_node_cache body");
    let targeted = [
        project_node,
        propagate_world,
        local_value,
        active_value,
        refresh,
    ]
    .join("\n");

    assert!(
        local_value.contains("let Some(local) = self.get::<LocalTransform>(entity) else")
            && local_value.contains("return Transform::default();")
            && local_value.contains("local.transform"),
        "local transform defaults must use a direct lookup branch"
    );
    assert!(
        active_value.contains("let Some(active) = self.get::<ActiveSelf>(entity) else")
            && active_value.contains("return true;")
            && active_value.contains("active.0"),
        "active-self defaults must use a direct lookup branch"
    );
    assert!(
        project_node.contains("let Some(name) = self.get::<Name>(entity) else")
            && project_node.contains("let Some(kind) = self.node_kind(entity) else")
            && project_node.contains("name: name.0.clone()")
            && project_node.contains("transform: self.local_transform_value(entity)"),
        "projected node reads must branch directly for name/kind and reuse local transform helper"
    );
    assert!(
        propagate_world.contains("self.local_transform_value(")
            && refresh.contains("self.project_node_for_read(entity)"),
        "world-matrix propagation must reuse the local helper and node-cache refresh must reuse the node projector"
    );
    assert!(
        !targeted.contains(".unwrap_or_default()")
            && !targeted.contains(".map(|name| name.0.clone())")
            && !targeted.contains(".copied().unwrap_or_default()"),
        "derived-state default component reads must not keep the old adapter chains"
    );
}

#[test]
fn node_records_projection_uses_pre_sized_push_snapshot() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("query.rs"),
    );
    let node_records = source
        .split("pub fn node_records(&self) -> Vec<SceneNode>")
        .nth(1)
        .and_then(|text| text.split("pub fn find_node").next())
        .expect("read node_records body");

    assert!(
        node_records.contains("let mut nodes = Vec::with_capacity(self.entities.len());")
            && node_records.contains("for entity in self.stable_entity_ids()")
            && node_records.contains("self.project_node_for_read(entity)")
            && node_records.contains("nodes.push(node);")
            && node_records.contains("nodes.sort_by_key(|node| node.id);")
            && !node_records.contains(".filter_map(")
            && !node_records.contains(".collect::<Vec<_>>()"),
        "node_records must build a pre-sized projected-node snapshot and retain final id sorting instead of relying on iterator collect growth"
    );
}

#[test]
fn world_query_scalar_accessors_use_direct_lookup_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("query.rs"),
    );
    let parent_of = source
        .split("pub fn parent_of")
        .nth(1)
        .and_then(|text| text.split("pub fn active_camera").next())
        .expect("read parent_of body");
    let active_self = source
        .split("pub fn active_self")
        .nth(1)
        .and_then(|text| text.split("pub fn set_active_self").next())
        .expect("read active_self body");
    let render_layer_mask = source
        .split("pub fn render_layer_mask")
        .nth(1)
        .and_then(|text| text.split("pub fn set_render_layer_mask").next())
        .expect("read render_layer_mask body");

    assert!(
        parent_of.contains("let Some(hierarchy) = self.get::<Hierarchy>(entity) else")
            && parent_of.contains("return None;")
            && parent_of.contains("hierarchy.parent")
            && !parent_of.contains(".and_then(|hierarchy| hierarchy.parent)"),
        "parent_of must branch directly on hierarchy presence"
    );
    assert!(
        active_self.contains("let Some(active) = self.get::<ActiveSelf>(entity) else")
            && active_self.contains("return None;")
            && active_self.contains("Some(active.0)")
            && !active_self.contains(".map(|active| active.0)"),
        "active_self must branch directly on generic active component presence"
    );
    assert!(
        render_layer_mask.contains("let Some(mask) = self.get::<RenderLayerMask>(entity) else")
            && render_layer_mask.contains("return None;")
            && render_layer_mask.contains("Some(mask.0)")
            && !render_layer_mask.contains(".map(|mask| mask.0)"),
        "render_layer_mask must branch directly on generic render-layer component presence"
    );
}

#[test]
fn retained_node_cache_refresh_updates_one_row_and_preserves_slice_pointer() {
    let mut world = World::empty();
    let first = world
        .spawn_node(NodeKind::Empty)
        .expect("first cache fixture node should spawn");
    let second = world
        .spawn_node(NodeKind::Empty)
        .expect("second cache fixture node should spawn");
    world.flush_pending_scene_systems();

    let before = world.nodes();
    assert_eq!(before.len(), 2);
    let retained_buffer = before.as_ptr();
    let unchanged_name = before
        .iter()
        .find(|node| node.id == first)
        .expect("first node should be cached")
        .name
        .clone();

    world.reset_ecs_frame_performance_diagnostics();
    assert!(world
        .rename_node(second, "Renamed Cache Row")
        .expect("second node should rename"));
    world.flush_pending_scene_systems();

    let after = world.nodes();
    assert_eq!(after.as_ptr(), retained_buffer);
    assert_eq!(
        after
            .iter()
            .find(|node| node.id == first)
            .expect("first node should remain cached")
            .name,
        unchanged_name
    );
    assert_eq!(
        after
            .iter()
            .find(|node| node.id == second)
            .expect("second node should remain cached")
            .name,
        "Renamed Cache Row"
    );
    assert_eq!(
        world
            .ecs_frame_performance_diagnostics()
            .derived_state
            .node_cache_rebuilt_entities,
        1
    );
}
