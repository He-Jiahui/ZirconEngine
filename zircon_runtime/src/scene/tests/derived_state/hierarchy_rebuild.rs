use super::*;

#[test]
fn derived_state_rebuilds_reuse_the_mutation_hierarchy_index() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let topology = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("hierarchy_topology.rs"),
    );
    let typed_api = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let active_rebuild = source
        .split("fn rebuild_active_in_hierarchy")
        .nth(1)
        .and_then(|text| text.split("fn rebuild_world_matrices").next())
        .expect("read active hierarchy rebuild body");
    let world_rebuild = source
        .split("fn rebuild_world_matrices")
        .nth(1)
        .and_then(|text| text.split("fn propagate_active_state").next())
        .expect("read world matrix rebuild body");
    let active_propagate = source
        .split("fn propagate_active_state")
        .nth(1)
        .and_then(|text| text.split("fn propagate_world_matrix").next())
        .expect("read active hierarchy propagation body");
    let world_propagate = source
        .split("fn propagate_world_matrix")
        .nth(1)
        .and_then(|text| text.split("fn hierarchy_traversal_index").next())
        .expect("read world matrix propagation body");
    assert!(
        active_rebuild.contains("let frontier = self.derived_state_dirty.take_active_frontier();")
            && active_rebuild.contains("self.ensure_hierarchy_mutation_index_current();")
            && active_rebuild
                .contains("let traversal = std::mem::take(&mut self.hierarchy_mutation_index);")
            && active_rebuild.contains("self.derived_state_frontier_roots(&frontier, &traversal)")
            && active_rebuild.contains("self.hierarchy_mutation_index = traversal;")
            && !active_rebuild.contains("self.active_in_hierarchy.clear();")
            && !active_rebuild.contains("self.root_entities()"),
        "active hierarchy rebuild must reuse the mutation hierarchy index instead of constructing a temporary world traversal"
    );
    assert!(
        world_rebuild
            .contains("let frontier = self.derived_state_dirty.take_transform_frontier();")
            && world_rebuild.contains("self.ensure_hierarchy_mutation_index_current();")
            && world_rebuild
                .contains("let traversal = std::mem::take(&mut self.hierarchy_mutation_index);")
            && world_rebuild.contains("self.derived_state_frontier_roots(&frontier, &traversal)")
            && world_rebuild.contains("self.hierarchy_mutation_index = traversal;")
            && !world_rebuild.contains("self.world_matrices.clear();")
            && !world_rebuild.contains("self.root_entities()"),
        "world matrix rebuild must reuse the mutation hierarchy index instead of constructing a temporary world traversal"
    );
    assert!(
        active_propagate.contains("traversal: &HierarchyTopology")
            && active_propagate.contains("let mut stack = vec![(entity, parent_active)];")
            && active_propagate
                .contains("while let Some((current, inherited_active)) = stack.pop()")
            && active_propagate.contains("traversal.children_of(current)")
            && active_propagate.contains(".rev()")
            && !active_propagate.contains("self.propagate_active_state(")
            && !active_propagate.contains("self.children_of(entity)"),
        "active hierarchy propagation must use an explicit DFS stack over the traversal index instead of recursion or per-node child scans"
    );
    assert!(
        world_propagate.contains("traversal: &HierarchyTopology")
            && world_propagate.contains("let mut stack = vec![(entity, parent_world)];")
            && world_propagate.contains("while let Some((current, inherited_world)) = stack.pop()")
            && world_propagate.contains("traversal.children_of(current)")
            && world_propagate.contains(".rev()")
            && !world_propagate.contains("self.propagate_world_matrix(")
            && !world_propagate.contains("self.children_of(entity)"),
        "world matrix propagation must use an explicit DFS stack over the traversal index instead of recursion or per-node child scans"
    );
    assert!(
        topology.contains("pub(super) struct HierarchyTopology")
            && topology.contains("roots: BTreeMap<usize, EntityId>")
            && topology
                .contains("children_by_parent: HashMap<EntityId, BTreeMap<usize, EntityId>>")
            && topology.contains("parent_by_entity: HashMap<EntityId, Option<EntityId>>")
            && topology.contains("self.parent_by_entity.len() == entity_count")
            && topology.contains("indexed_entities: HashSet<EntityId>")
            && topology.contains("generation: u64")
            && topology.contains("fn mark_structural_change(&mut self)")
            && topology.contains(".flat_map(|children| children.values().copied())")
            && topology.contains("pub(super) fn parent_of(&self, entity: EntityId)")
            && !topology.contains("dense_child_ranges")
            && !topology.contains("topological_entities"),
        "hierarchy topology must own one versioned stable root and ordered adjacency projection without rebuilding a world-wide dense traversal after each changed edge"
    );
    assert!(
        typed_api.contains("enum HierarchyMutationMode")
            && typed_api.contains("HierarchyMutationMode::Checked")
            && typed_api.contains("self.mark_checked_hierarchy_derived_state_dirty_at(entity);")
            && typed_api.contains("self.mark_inspection_subtree_fields_dirty(entity);"),
        "validated structural reparenting must retain the changed edge identity through inspection and derived-state invalidation"
    );
}

#[test]
fn hierarchy_validity_rebuild_uses_pre_sized_parent_snapshot() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("hierarchy_validation.rs"),
    );
    let rebuild = source
        .split("fn rebuild_hierarchy_validity")
        .nth(1)
        .and_then(|text| text.split("fn hierarchy_parent_snapshot").next())
        .expect("read hierarchy validity rebuild body");
    let snapshot = source
        .split("fn hierarchy_parent_snapshot")
        .nth(1)
        .expect("read hierarchy parent snapshot body");
    let path_buffer = rebuild
        .find("let mut path = Vec::new();")
        .expect("read reusable hierarchy path buffer");
    let start_loop = rebuild
        .find("for start in entities.iter().copied()")
        .expect("read hierarchy validation start loop");

    assert!(
        rebuild.contains("let mut parents = self.hierarchy_parent_snapshot();")
            && rebuild.contains("let mut hierarchy_updates = Vec::new();")
            && rebuild.contains(".is_current_for_entity_count(self.entities.len());")
            && rebuild.contains("let entities = self.stable_entity_ids().collect::<Vec<_>>();")
            && rebuild.contains("for entity in entities.iter().copied()")
            && rebuild.contains("parents.contains_key(parent)")
            && rebuild.contains("parents.insert(entity, current_parent);")
            && rebuild.contains("let mut completed = HashSet::with_capacity(entities.len());")
            && rebuild.contains("let mut path_positions = HashMap::with_capacity(entities.len());")
            && path_buffer < start_loop
            && rebuild.contains("path.clear();")
            && rebuild.contains("if let Some(cycle_start) = path_positions.get(&entity).copied()")
            && rebuild.contains("path[cycle_start..]")
            && rebuild.contains(".insert(repaired_entity, None)")
            && rebuild.contains("for entity in path.drain(..)")
            && rebuild.contains("path_positions.remove(&entity);")
            && rebuild.contains("completed.insert(entity);")
            && rebuild.contains(
                "self.update_hierarchy_mutation_index(entity, previous_parent, current_parent);"
            )
            && !rebuild.contains("parent_chain_is_invalid")
            && !rebuild.contains("HashSet<_> = self.entities.iter().copied().collect()")
            && !rebuild.contains(".map(|entity|"),
        "hierarchy validation must use one parent snapshot, direct-edge repair, a reusable completed path walk, and authoritative topology updates after invalid edges are removed"
    );
    assert!(
        snapshot.contains("let mut parents = HashMap::with_capacity(self.entities.len());")
            && snapshot.contains("for entity in self.stable_entity_ids()")
            && snapshot.contains("let parent = match self.get::<Hierarchy>(entity)")
            && snapshot.contains("Some(hierarchy) => hierarchy.parent")
            && snapshot.contains("None => None")
            && snapshot.contains("parents.insert(entity, parent);")
            && !snapshot.contains(".and_then(|hierarchy| hierarchy.parent)")
            && !snapshot.contains(".collect()"),
        "hierarchy parent snapshot must pre-size and push parent entries through direct lookup branches"
    );
}

#[test]
fn serialized_world_initializer_resets_node_cache_projection() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("world.rs"),
    );
    let persistent_initializer = source
        .split("pub(super) fn from_persistent_state")
        .nth(1)
        .and_then(|text| text.split("impl<'de> Deserialize<'de> for World").next())
        .expect("read serialized world initializer");

    assert!(
        persistent_initializer.contains("node_cache_rows: HashMap::new(),")
            && persistent_initializer.contains("node_cache_topology_generation: 0,"),
        "serialized worlds must reset the runtime-only node-cache projection alongside the cache"
    );
}

#[test]
fn empty_entity_spawn_registers_a_root_in_the_mutation_index() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api")
            .join("bundle_entry.rs"),
    );
    let spawn_empty = source
        .split("pub(crate) fn spawn_empty_at")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn spawn_at").next())
        .expect("read empty entity spawn body");

    assert!(
        spawn_empty.contains("self.update_hierarchy_mutation_index(entity, None, None);")
            && spawn_empty.contains("self.append_entity_to_dense_storage(entity);")
            && spawn_empty.contains("self.record_node_kind_added(NodeKind::Empty);"),
        "an empty entity must enter the mutation hierarchy index as a root during spawn"
    );
}

#[test]
fn subtree_record_collection_reuses_hierarchy_traversal_index() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let public_collection = source
        .split("pub(super) fn collect_subtree_records")
        .nth(1)
        .and_then(|text| {
            text.split("fn collect_subtree_records_with_traversal")
                .next()
        })
        .expect("read public subtree record collection body");
    let indexed_collection = source
        .split("fn collect_subtree_records_with_traversal")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn is_descendant").next())
        .expect("read recursive subtree record collection body");

    assert!(
        public_collection.contains("self.hierarchy_mutation_index")
            && public_collection.contains(".is_current_for_entity_count(self.entities.len())")
            && public_collection
                .contains("self.hierarchy_mutation_index.children_of(current).rev()")
            && public_collection.contains("let traversal = self.hierarchy_traversal_index();")
            && public_collection.contains("self.collect_subtree_records_with_traversal(")
            && indexed_collection.contains("traversal: &HierarchyTraversalIndex")
            && indexed_collection.contains("let mut stack = vec![entity];")
            && indexed_collection.contains("while let Some(current) = stack.pop()")
            && indexed_collection.contains("traversal.children_of(current).iter().rev().copied()")
            && !indexed_collection.contains("self.children_of(entity)")
            && !indexed_collection.contains("self.collect_subtree_records_with_traversal(")
            && !source.contains("fn children_of(&self, entity: EntityId) -> Vec<EntityId>")
            && !source.contains(".filter(|child| self.parent_of(*child) == Some(entity))")
            && !indexed_collection.contains(".collect()"),
        "subtree record collection must build one hierarchy traversal index and use an explicit DFS stack through indexed child slices instead of scanning all entities per node"
    );
}

#[test]
fn mobility_static_parent_preflight_uses_direct_parent_branch() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("hierarchy.rs"),
    );
    let validate_mobility_change = source
        .split("pub(super) fn validate_mobility_change")
        .nth(1)
        .and_then(|text| text.split("fn ensure_transform_mutable").next())
        .expect("read validate_mobility_change body");

    assert!(
        validate_mobility_change.contains("if let Some(parent) = self.parent_of(entity)")
            && validate_mobility_change
                .contains("if self.mobility(parent) == Some(Mobility::Dynamic)")
            && validate_mobility_change.contains("SceneError::StaticMobilityUnderDynamicParent")
            && validate_mobility_change.contains("entity,")
            && validate_mobility_change.contains("parent,")
            && !validate_mobility_change.contains(".is_some_and("),
        "mobility validation must use a direct parent branch for Static-under-Dynamic preflight"
    );
}

#[test]
fn internal_scene_system_flushes_reuse_schedule_stage_plan() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let stage_flush = source
        .split("pub(crate) fn run_internal_scene_systems_for_stage")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn flush_pending_scene_systems_for_stage")
                .next()
        })
        .expect("read internal scene-system stage flush body");
    let pending_stage_flush = source
        .split("pub(crate) fn flush_pending_scene_systems_for_stage")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn flush_pending_scene_systems")
                .next()
        })
        .expect("read pending internal scene-system stage flush body");
    let pending_flush = source
        .split("pub(crate) fn flush_pending_scene_systems(&mut self)")
        .nth(1)
        .and_then(|text| {
            text.split("pub(crate) fn set_scene_system_flush_deferred")
                .next()
        })
        .expect("read pending scene-system flush body");

    assert!(
        stage_flush.contains("let stage_plan = self.schedule.stage_plan();")
            && stage_flush.contains("stage_plan.internal_systems_for_stage(stage)")
            && !stage_flush.contains("self.schedule.systems_for_stage(stage)")
            && !stage_flush.contains(".cloned()")
            && !stage_flush.contains(".collect::<Vec<_>>()"),
        "single-stage internal scene-system flush must reuse the cached stage-plan snapshot instead of collecting cloned descriptors"
    );
    assert!(
        pending_stage_flush.contains("if !self.derived_state_dirty.has_pending()")
            && pending_stage_flush.contains("stage_plan.internal_systems_for_stage(stage)")
            && pending_stage_flush.contains("self.derived_state_dirty.should_run(system)")
            && !pending_stage_flush.contains("run_internal_scene_systems_for_stage"),
        "stage completion may flush still-dirty derived state but must not replay UpdateEvents or ApplyDeferred"
    );
    assert!(
        pending_flush.contains("let stage_plan = self.schedule.stage_plan();")
            && pending_flush.contains("for stage in stage_plan.stages().iter().copied()")
            && pending_flush.contains("stage_plan.internal_systems_for_stage(stage)")
            && !pending_flush.contains("self.schedule.systems().to_vec()")
            && !pending_flush.contains("SystemStage::ORDER")
            && !pending_flush.contains(".to_vec()"),
        "pending internal scene-system flush must walk the cached stage-plan snapshot instead of cloning the flat descriptor registry"
    );

    let runner_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("schedule_runner.rs"),
    );
    assert!(runner_source.contains("world.flush_pending_scene_systems_for_stage(stage);"));
    assert!(!runner_source.contains("world.run_internal_scene_systems_for_stage(stage);"));
}
