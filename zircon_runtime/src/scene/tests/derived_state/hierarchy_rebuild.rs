use super::*;

#[test]
fn derived_state_rebuilds_use_single_hierarchy_traversal_index() {
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
    let traversal_index = source
        .split("fn hierarchy_traversal_index")
        .nth(1)
        .and_then(|text| text.split("fn children_of").next())
        .expect("read hierarchy traversal index builder");

    assert!(
        active_rebuild.contains("let traversal = self.hierarchy_traversal_index();")
            && active_rebuild.contains("for root in traversal.roots()")
            && !active_rebuild.contains("self.root_entities()"),
        "active hierarchy rebuild must build one traversal index instead of collecting roots separately"
    );
    assert!(
        world_rebuild.contains("let traversal = self.hierarchy_traversal_index();")
            && world_rebuild.contains("for root in traversal.roots()")
            && !world_rebuild.contains("self.root_entities()"),
        "world matrix rebuild must build one traversal index instead of collecting roots separately"
    );
    assert!(
        active_propagate.contains("traversal: &HierarchyTraversalIndex")
            && active_propagate.contains("traversal.children_of(entity)")
            && !active_propagate.contains("self.children_of(entity)"),
        "active hierarchy propagation must reuse the traversal index instead of scanning children at each node"
    );
    assert!(
        world_propagate.contains("traversal: &HierarchyTraversalIndex")
            && world_propagate.contains("traversal.children_of(entity)")
            && !world_propagate.contains("self.children_of(entity)"),
        "world matrix propagation must reuse the traversal index instead of scanning children at each node"
    );
    assert!(
        traversal_index.contains("HierarchyTraversalIndex::with_entity_capacity(self.entities.len())")
            && traversal_index.contains("for entity in self.entities.iter().copied()")
            && traversal_index.contains("if let Some(parent) = self.parent_of(entity)")
            && traversal_index.contains("index.push_child(parent, entity);")
            && traversal_index.contains("index.push_root(entity);"),
        "hierarchy traversal index must preserve world entity order while building roots and child lists once"
    );
    assert!(
        source.contains("struct HierarchyTraversalIndex")
            && source.contains("roots: Vec<EntityId>")
            && source.contains("children_by_parent: HashMap<EntityId, Vec<EntityId>>")
            && source.contains("fn with_entity_capacity(entity_count: usize) -> Self")
            && source.contains("roots: Vec::with_capacity(entity_count)")
            && source.contains("children_by_parent: HashMap::with_capacity(entity_count)")
            && source.contains("fn roots(&self) -> &[EntityId]")
            && source.contains("fn children_of(&self, parent: EntityId) -> &[EntityId]"),
        "hierarchy traversal index must own pre-sized root and child storage with slice accessors"
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

    assert!(
        rebuild.contains("let parents = self.hierarchy_parent_snapshot();")
            && rebuild.contains("for entity_index in 0..self.entities.len()")
            && rebuild.contains("let entity = self.entities[entity_index];")
            && rebuild.contains("parents.contains_key(parent)")
            && rebuild.contains("parent_chain_is_invalid(*parent, entity, &parents)")
            && !rebuild.contains("HashSet<_> = self.entities.iter().copied().collect()")
            && !rebuild.contains(".collect::<Vec<_>>()")
            && !rebuild.contains(".map(|entity|"),
        "hierarchy validity rebuild must use one pre-sized parent snapshot and an index walk instead of collect-built temporary snapshots"
    );
    assert!(
        snapshot.contains("let mut parents = HashMap::with_capacity(self.entities.len());")
            && snapshot.contains("for entity in self.entities.iter().copied()")
            && snapshot.contains("let parent = match self.hierarchy.get(&entity)")
            && snapshot.contains("Some(hierarchy) => hierarchy.parent")
            && snapshot.contains("None => None")
            && snapshot.contains("parents.insert(entity, parent);")
            && !snapshot.contains(".and_then(|hierarchy| hierarchy.parent)")
            && !snapshot.contains(".collect()"),
        "hierarchy parent snapshot must pre-size and push parent entries through direct lookup branches"
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
    let recursive_collection = source
        .split("fn collect_subtree_records_with_traversal")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn is_descendant").next())
        .expect("read recursive subtree record collection body");

    assert!(
        public_collection.contains("let traversal = self.hierarchy_traversal_index();")
            && public_collection.contains("self.collect_subtree_records_with_traversal(")
            && recursive_collection.contains("traversal: &HierarchyTraversalIndex")
            && recursive_collection.contains("traversal.children_of(entity)")
            && recursive_collection.contains(
                "self.collect_subtree_records_with_traversal(*child, records, traversal)"
            )
            && !recursive_collection.contains("self.children_of(entity)")
            && !source.contains("fn children_of(&self, entity: EntityId) -> Vec<EntityId>")
            && !source.contains(".filter(|child| self.parent_of(*child) == Some(entity))")
            && !source.contains(".collect()"),
        "subtree record collection must build one hierarchy traversal index and recurse through indexed child slices instead of scanning all entities per node"
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
            && validate_mobility_change
                .contains("\"cannot make node {entity} Static under Dynamic parent\"")
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
            text.split("pub(crate) fn flush_pending_scene_systems")
                .next()
        })
        .expect("read internal scene-system stage flush body");
    let pending_flush = source
        .split("pub(crate) fn flush_pending_scene_systems")
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
        pending_flush.contains("let stage_plan = self.schedule.stage_plan();")
            && pending_flush.contains("for stage in stage_plan.stages().iter().copied()")
            && pending_flush.contains("stage_plan.internal_systems_for_stage(stage)")
            && !pending_flush.contains("self.schedule.systems().to_vec()")
            && !pending_flush.contains("SystemStage::ORDER")
            && !pending_flush.contains(".to_vec()"),
        "pending internal scene-system flush must walk the cached stage-plan snapshot instead of cloning the flat descriptor registry"
    );
}
