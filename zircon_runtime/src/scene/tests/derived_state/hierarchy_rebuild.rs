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
    let mutation_index = source
        .split("impl HierarchyMutationIndex")
        .nth(1)
        .and_then(|text| text.split("const fn node_kind_ordinal_index").next())
        .expect("read hierarchy mutation index implementation");

    assert!(
        active_rebuild.contains("self.ensure_hierarchy_mutation_index_current();")
            && active_rebuild.contains("let traversal = std::mem::take(&mut self.hierarchy_mutation_index);")
            && active_rebuild.contains("for root in traversal.roots()")
            && active_rebuild.contains("self.hierarchy_mutation_index = traversal;")
            && !active_rebuild.contains("self.active_in_hierarchy.clear();")
            && !active_rebuild.contains("self.root_entities()"),
        "active hierarchy rebuild must reuse the mutation hierarchy index instead of constructing a temporary world traversal"
    );
    assert!(
        world_rebuild.contains("self.ensure_hierarchy_mutation_index_current();")
            && world_rebuild.contains("let traversal = std::mem::take(&mut self.hierarchy_mutation_index);")
            && world_rebuild.contains("for root in traversal.roots()")
            && world_rebuild.contains("self.hierarchy_mutation_index = traversal;")
            && !world_rebuild.contains("self.world_matrices.clear();")
            && !world_rebuild.contains("self.root_entities()"),
        "world matrix rebuild must reuse the mutation hierarchy index instead of constructing a temporary world traversal"
    );
    assert!(
        active_propagate.contains("traversal: &HierarchyMutationIndex")
            && active_propagate.contains("let mut stack = vec![(entity, parent_active)];")
            && active_propagate.contains("while let Some((current, inherited_active)) = stack.pop()")
            && active_propagate.contains("traversal.children_of(current)")
            && active_propagate.contains(".rev()")
            && !active_propagate.contains("self.propagate_active_state(")
            && !active_propagate.contains("self.children_of(entity)"),
        "active hierarchy propagation must use an explicit DFS stack over the traversal index instead of recursion or per-node child scans"
    );
    assert!(
        world_propagate.contains("traversal: &HierarchyMutationIndex")
            && world_propagate.contains("let mut stack = vec![(entity, parent_world)];")
            && world_propagate.contains("while let Some((current, inherited_world)) = stack.pop()")
            && world_propagate.contains("traversal.children_of(current)")
            && world_propagate.contains(".rev()")
            && !world_propagate.contains("self.propagate_world_matrix(")
            && !world_propagate.contains("self.children_of(entity)"),
        "world matrix propagation must use an explicit DFS stack over the traversal index instead of recursion or per-node child scans"
    );
    assert!(
        mutation_index.contains("roots: BTreeMap<usize, EntityId>")
            && mutation_index.contains("children_by_parent: HashMap<EntityId, BTreeMap<usize, EntityId>>")
            && mutation_index.contains("fn is_current_for_entity_count(&self, entity_count: usize) -> bool")
            && mutation_index.contains("self.indexed_entities.len() == entity_count")
            && mutation_index.contains("fn roots(&self) -> impl DoubleEndedIterator<Item = EntityId> + '_")
            && mutation_index.contains("self.roots.values().copied()")
            && mutation_index.contains("self.roots.remove(&stable_order);")
            && mutation_index.contains("self.roots.insert(stable_order, entity);"),
        "hierarchy mutation index must own stable root and child order for every derived-state traversal"
    );
}

#[test]
fn hierarchy_validity_rebuild_uses_pre_sized_parent_snapshot() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let rebuild = source
        .split("fn rebuild_hierarchy_validity")
        .nth(1)
        .and_then(|text| text.split("fn hierarchy_parent_snapshot").next())
        .expect("read hierarchy validity rebuild body");
    let snapshot = source
        .split("fn hierarchy_parent_snapshot")
        .nth(1)
        .and_then(|text| text.split("fn rebuild_active_in_hierarchy").next())
        .expect("read hierarchy parent snapshot body");
    let parent_chain = source
        .split("fn parent_chain_is_invalid(")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn matrix_to_transform").next())
        .expect("read parent-chain validation body");

    assert!(
        rebuild.contains("let parents = self.hierarchy_parent_snapshot();")
            && rebuild.contains("let mut seen = HashSet::new();")
            && rebuild.contains("let mut hierarchy_updates = Vec::new();")
            && rebuild.contains(".is_current_for_entity_count(self.entities.len());")
            && rebuild.contains("let entities = self.stable_entity_ids().collect::<Vec<_>>();")
            && rebuild.contains("for entity in entities {")
            && rebuild.contains("parents.contains_key(parent)")
            && rebuild.contains("parent_chain_is_invalid(*parent, entity, &parents, &mut seen)")
            && rebuild.contains("self.update_hierarchy_mutation_index(entity, previous_parent, current_parent);")
            && !rebuild.contains("HashSet<_> = self.entities.iter().copied().collect()")
            && !rebuild.contains(".map(|entity|"),
        "hierarchy validity rebuild must use one parent snapshot, stable entity walk, and update the authoritative hierarchy index after invalid edges are removed"
    );
    assert!(
        parent_chain.contains("seen.clear();")
            && parent_chain.contains("seen.insert(entity);")
            && !parent_chain.contains("HashSet::from([entity])"),
        "hierarchy validation must reuse one visited-set allocation across all parent-chain checks"
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
        public_collection.contains("let traversal = self.hierarchy_traversal_index();")
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
