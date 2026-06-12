use std::path::{Path, PathBuf};

use crate::core::framework::render::{
    RenderExtractContext, RenderWorldSnapshotHandle, SceneViewportExtractRequest,
};
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{MeshRenderer, Mobility, NodeRecord};
use crate::scene::{NodeKind, SystemStage, World};

const LARGE_HIERARCHY_NODE_COUNT: usize = 256;

#[test]
fn spawn_node_kind_ordinals_compare_kinds_without_candidate_clones() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let ordinal_for = source
        .split("pub(super) fn ordinal_for")
        .nth(1)
        .and_then(|text| text.split("pub(super) fn node_kind").next())
        .expect("read ordinal_for body");

    assert!(
        ordinal_for.contains("let mut ordinal = 1;")
            && ordinal_for.contains("for entity in self.entities.iter().copied()")
            && ordinal_for.contains("self.kinds.get(&entity) == Some(&kind)")
            && ordinal_for.contains("ordinal += 1;")
            && ordinal_for.contains("ordinal")
            && !ordinal_for.contains("kind.clone()")
            && !ordinal_for.contains("self.node_kind(**entity)"),
        "spawn-node kind ordinal lookup must compare stored kinds by reference without cloning the requested NodeKind per candidate entity"
    );
}

#[test]
fn spawn_node_reuses_copy_node_kind_without_spawn_path_clones() {
    let component_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("components")
            .join("scene.rs"),
    )
    .replace("\r\n", "\n");
    let bootstrap_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("bootstrap.rs"),
    );
    let derived_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let spawn_node = bootstrap_source
        .split("pub fn spawn_node")
        .nth(1)
        .and_then(|text| text.split("pub fn spawn_mesh_node").next())
        .expect("read spawn_node body");
    let node_kind = derived_source
        .split("pub(super) fn node_kind")
        .nth(1)
        .and_then(|text| text.split("pub(crate) fn run_internal_scene_system").next())
        .expect("read node_kind body");

    assert!(
        component_source.contains(
            "#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]\npub enum NodeKind"
        ),
        "NodeKind must stay copyable so spawn-node bootstrap paths can pass kinds by value without cloning"
    );
    assert!(
        spawn_node.contains("self.ordinal_for(kind)")
            && spawn_node.contains("self.kinds.insert(id, kind);")
            && !spawn_node.contains("kind.clone()"),
        "spawn_node must reuse Copy NodeKind values for ordinal lookup, storage insertion, and component-kind branching"
    );
    assert!(
        node_kind.contains("self.kinds.get(&entity).copied()") && !node_kind.contains(".cloned()"),
        "node_kind lookups must copy stored NodeKind values instead of cloning them"
    );
}

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
fn derived_state_projected_reads_use_direct_parent_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let world_matrix = source
        .split("fn project_world_matrix_for_read_inner")
        .nth(1)
        .and_then(|text| text.split("fn parent_for_read").next())
        .expect("read projected world matrix body");
    let parent_for_read = source
        .split("fn parent_for_read")
        .nth(1)
        .and_then(|text| text.split("fn active_self_chain_value").next())
        .expect("read parent_for_read body");
    let active_chain = source
        .split("fn active_self_chain_value")
        .nth(1)
        .and_then(|text| text.split("fn rebuild_hierarchy_validity").next())
        .expect("read active self chain body");

    assert!(
        world_matrix.contains("let Some(parent) = self.parent_for_read(entity) else")
            && world_matrix.contains("return Some(local_matrix);")
            && world_matrix.contains(
                "let Some(parent_matrix) = self.project_world_matrix_for_read_inner(parent, seen) else"
            )
            && world_matrix.contains("return None;")
            && world_matrix.contains("Some(parent_matrix * local_matrix)")
            && !world_matrix.contains(".map(|parent|")
            && !world_matrix.contains(".unwrap_or(Some(local_matrix))"),
        "projected world-matrix reads must branch directly on parent and recursive parent matrix presence"
    );
    assert!(
        parent_for_read.contains("let Some(hierarchy) = self.hierarchy.get(&entity) else")
            && parent_for_read.contains("let Some(parent) = hierarchy.parent else")
            && parent_for_read.contains("if parent == entity || !self.contains_entity(parent)")
            && parent_for_read.contains("Some(parent)")
            && !parent_for_read.contains(".and_then(|hierarchy| hierarchy.parent)")
            && !parent_for_read.contains(".filter("),
        "parent_for_read must resolve valid parents through direct Option branches"
    );
    assert!(
        active_chain.contains("if let Some(parent) = self.parent_for_read(entity)")
            && active_chain.contains("if !self.active_self_chain_value(parent, seen)")
            && active_chain.contains("return false;")
            && active_chain.contains("self.active_self_value(entity)")
            && !active_chain.contains(".map(|parent|")
            && !active_chain.contains(".unwrap_or(true)"),
        "active-chain reads must branch directly on optional parent state"
    );
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

    assert!(
        active_read.contains("let Some(active) = self.active_in_hierarchy.get(&entity) else")
            && active_read.contains("return Some(active.0);")
            && active_read.contains("if !self.contains_entity(entity)")
            && active_read
                .contains("Some(self.active_self_chain_value(entity, &mut HashSet::new()))")
            && !active_read.contains(".map(|active| active.0)")
            && !active_read.contains(".then(||"),
        "projected active reads must branch directly for cached and dirty paths"
    );
    assert!(
        world_transform.contains("let Some(world) = self.world_matrices.get(&entity) else")
            && world_transform.contains("return Some(matrix_to_transform(world.0));")
            && world_transform.contains(
                "let Some(world_matrix) = self.project_world_matrix_for_read(entity) else"
            )
            && world_transform.contains("Some(matrix_to_transform(world_matrix))")
            && !world_transform.contains(".map(|world| matrix_to_transform(world.0))")
            && !world_transform.contains(".map(matrix_to_transform)"),
        "projected world-transform reads must branch directly for cached and dirty paths"
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
    let world_matrix = source
        .split("fn project_world_matrix_for_read_inner")
        .nth(1)
        .and_then(|text| text.split("fn parent_for_read").next())
        .expect("read project_world_matrix_for_read_inner body");
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
        world_matrix,
        propagate_world,
        local_value,
        active_value,
        refresh,
    ]
    .join("\n");

    assert!(
        local_value.contains("let Some(local) = self.local_transforms.get(&entity) else")
            && local_value.contains("return Transform::default();")
            && local_value.contains("local.transform"),
        "local transform defaults must use a direct lookup branch"
    );
    assert!(
        active_value.contains("let Some(active) = self.active_self.get(&entity) else")
            && active_value.contains("return true;")
            && active_value.contains("active.0"),
        "active-self defaults must use a direct lookup branch"
    );
    assert!(
        project_node.contains("let Some(name) = self.names.get(&entity) else")
            && project_node.contains("let Some(kind) = self.node_kind(entity) else")
            && project_node.contains("name: name.0.clone()")
            && project_node.contains("transform: self.local_transform_value(entity)"),
        "projected node reads must branch directly for name/kind and reuse local transform helper"
    );
    assert!(
        world_matrix.contains("let local = self.local_transform_value(entity);")
            && propagate_world.contains("let local = self.local_transform_value(entity);")
            && refresh.contains("let Some(name) = self.names.get(&entity) else")
            && refresh.contains("name: name.0.clone()")
            && refresh.contains("transform: self.local_transform_value(entity)"),
        "world-matrix and node-cache rebuilds must reuse direct default/name branches"
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
            && node_records.contains("for entity in self.entities.iter().copied()")
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
        parent_of.contains("let Some(hierarchy) = self.hierarchy.get(&entity) else")
            && parent_of.contains("return None;")
            && parent_of.contains("hierarchy.parent")
            && !parent_of.contains(".and_then(|hierarchy| hierarchy.parent)"),
        "parent_of must branch directly on hierarchy presence"
    );
    assert!(
        active_self.contains("let Some(active) = self.active_self.get(&entity) else")
            && active_self.contains("return None;")
            && active_self.contains("Some(active.0)")
            && !active_self.contains(".map(|active| active.0)"),
        "active_self must branch directly on fixed active component presence"
    );
    assert!(
        render_layer_mask.contains("let Some(mask) = self.render_layer_masks.get(&entity) else")
            && render_layer_mask.contains("return None;")
            && render_layer_mask.contains("Some(mask.0)")
            && !render_layer_mask.contains(".map(|mask| mask.0)"),
        "render_layer_mask must branch directly on fixed render-layer component presence"
    );
}

#[test]
fn retained_node_cache_refresh_reuses_pre_sized_storage() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let refresh = source
        .split("pub(super) fn refresh_node_cache")
        .nth(1)
        .and_then(|text| text.split("fn prepare_render_extract").next())
        .expect("read refresh_node_cache body");

    assert!(
        refresh.contains("self.node_cache.clear();")
            && refresh.contains("self.node_cache.reserve(self.entities.len());")
            && refresh.contains("for entity in self.entities.iter().copied()")
            && refresh.contains("self.node_cache.push(SceneNode")
            && refresh.contains("parent: self.parent_of(entity)")
            && !refresh.contains("self.node_cache = self")
            && !refresh.contains(".filter_map(")
            && !refresh.contains(".collect()"),
        "refresh_node_cache must reuse retained cache storage with direct pushes instead of assigning a freshly collected Vec"
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

#[test]
fn projected_reads_stay_fresh_until_post_update_refreshes_retained_cache() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();
    world.set_active_self(parent, false).unwrap();

    assert!(world.has_pending_scene_systems());
    assert!(world
        .nodes()
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent.is_none()));
    assert!(world
        .node_records()
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent == Some(parent)));
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(
        world
            .world_matrix(child)
            .unwrap()
            .to_scale_rotation_translation()
            .2,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    let refreshed_nodes = world.nodes().to_vec();
    assert!(refreshed_nodes
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent == Some(parent)));
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.nodes(), refreshed_nodes.as_slice());
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn no_op_mutators_do_not_mark_derived_state_dirty() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    let static_child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap();
    world.set_active_self(parent, false).unwrap();
    world.set_render_layer_mask(child, 0b0010).unwrap();
    world.set_mobility(static_child, Mobility::Static).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    assert!(!world.set_parent_checked(child, Some(parent)).unwrap());
    assert!(!world
        .update_transform(child, Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)))
        .unwrap());
    assert!(!world.set_active_self(parent, false).unwrap());
    assert!(!world.set_render_layer_mask(child, 0b0010).unwrap());
    assert!(!world.set_mobility(static_child, Mobility::Static).unwrap());

    assert!(!world.has_pending_scene_systems());
    let static_reparent_error = world.set_parent_checked(static_child, None).unwrap_err();
    assert!(static_reparent_error.contains("Static"));
    assert!(!world.has_pending_scene_systems());

    assert!(!world.has_pending_scene_systems());
}

#[test]
fn imported_records_validate_missing_parents_and_preserve_out_of_order_links() {
    let mut missing_parent_record = detached_node_record(10, NodeKind::Mesh);
    missing_parent_record.parent = Some(999);
    let mut world = World::empty();
    world.insert_node_record(missing_parent_record).unwrap();
    assert_eq!(world.node_record(10).unwrap().parent, Some(999));

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.node_record(10).unwrap().parent, None);

    let mut parent_record = detached_node_record(42, NodeKind::Cube);
    parent_record.transform = Transform::from_translation(Vec3::new(3.0, 0.0, 0.0));
    let mut child_record = detached_node_record(43, NodeKind::Mesh);
    child_record.parent = Some(parent_record.id);
    child_record.transform = Transform::from_translation(Vec3::new(4.0, 0.0, 0.0));

    let mut world = World::empty();
    world.insert_node_record(child_record).unwrap();
    world.insert_node_record(parent_record).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);

    assert_eq!(world.node_record(43).unwrap().parent, Some(42));
    assert_eq!(
        world.world_transform(43).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
}

#[test]
fn hierarchy_cycle_rejection_preserves_existing_parent_state() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(child, Some(parent)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    let error = world.set_parent_checked(parent, Some(child)).unwrap_err();

    assert!(error.contains("cycle"));
    assert_eq!(world.find_node(parent).unwrap().parent, None);
    assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn active_hierarchy_propagates_inactive_and_reactivated_ancestors() {
    let mut world = World::new();
    let root = world.spawn_node(NodeKind::Cube);
    let middle = world.spawn_node(NodeKind::Cube);
    let leaf = world.spawn_node(NodeKind::Mesh);
    world.set_parent_checked(middle, Some(root)).unwrap();
    world.set_parent_checked(leaf, Some(middle)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world.set_active_self(root, false).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(middle), Some(false));
    assert_eq!(world.active_in_hierarchy(leaf), Some(false));
    assert!(world
        .build_prepared_render_frame_extract(&RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(201),
            SceneViewportExtractRequest::default(),
        ))
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != leaf));

    world.set_active_self(root, true).unwrap();
    world.set_active_self(middle, false).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(root), Some(true));
    assert_eq!(world.active_in_hierarchy(middle), Some(false));
    assert_eq!(world.active_in_hierarchy(leaf), Some(false));

    world.set_active_self(middle, true).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);
    assert_eq!(world.active_in_hierarchy(leaf), Some(true));
    assert!(world
        .build_prepared_render_frame_extract(&RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(202),
            SceneViewportExtractRequest::default(),
        ))
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == leaf));
}

#[test]
fn post_update_propagates_large_hierarchy_transform_and_active_state() {
    let mut world = World::new();
    let mut entities = Vec::with_capacity(LARGE_HIERARCHY_NODE_COUNT);
    for index in 0..LARGE_HIERARCHY_NODE_COUNT {
        let entity = world.spawn_node(if index + 1 == LARGE_HIERARCHY_NODE_COUNT {
            NodeKind::Mesh
        } else {
            NodeKind::Cube
        });
        world
            .update_transform(
                entity,
                Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            )
            .unwrap();
        if let Some(parent) = entities.last().copied() {
            world.set_parent_checked(entity, Some(parent)).unwrap();
        }
        entities.push(entity);
    }
    let hidden_ancestor = entities[LARGE_HIERARCHY_NODE_COUNT / 2];
    let deepest = *entities.last().unwrap();
    world.set_active_self(hidden_ancestor, false).unwrap();

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);

    assert_eq!(
        world.world_transform(deepest).unwrap().translation,
        Vec3::new(LARGE_HIERARCHY_NODE_COUNT as f32, 0.0, 0.0)
    );
    assert_eq!(world.active_in_hierarchy(deepest), Some(false));
    assert!(world
        .nodes()
        .iter()
        .find(|node| node.id == deepest)
        .is_some_and(|node| node.parent == Some(entities[LARGE_HIERARCHY_NODE_COUNT - 2])));
}

#[test]
fn mobility_changes_refresh_visibility_buckets_without_transform_rebuild() {
    let mut world = World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let mesh = world.spawn_node(NodeKind::Mesh);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(world.set_mobility(mesh, Mobility::Static).unwrap());
    assert!(world.update_transform(mesh, Transform::default()).is_err());
    assert!(world.set_parent_checked(mesh, Some(parent)).is_err());
    let static_extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(301),
        SceneViewportExtractRequest::default(),
    ));
    assert!(static_extract.visibility.static_entities.contains(&mesh));
    assert!(!static_extract.visibility.dynamic_entities.contains(&mesh));

    assert!(world.set_mobility(mesh, Mobility::Dynamic).unwrap());
    let dynamic_extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(302),
        SceneViewportExtractRequest::default(),
    ));
    assert!(dynamic_extract.visibility.dynamic_entities.contains(&mesh));
    assert!(!dynamic_extract.visibility.static_entities.contains(&mesh));
}

#[test]
fn render_extract_prepare_flushes_direct_frame_and_legacy_viewport_paths() {
    let mut world = pending_reparented_world();
    let child = world
        .node_records()
        .into_iter()
        .find(|node| matches!(node.kind, NodeKind::Mesh))
        .unwrap()
        .id;
    assert!(world.has_pending_scene_systems());

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest::default());
    assert!(packet.scene.meshes.iter().all(|mesh| mesh.node_id != child));
    assert!(world.has_pending_scene_systems());

    let frame = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(401),
        SceneViewportExtractRequest::default(),
    ));
    assert!(frame
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != child));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn property_path_node_cache_changes_mark_dirty_and_zero_morph_extension_is_not_noop() {
    let mut world = World::new();
    let mesh = world.spawn_node(NodeKind::Mesh);
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    let tint_path = ComponentPropertyPath::parse("MeshRenderer.tint").unwrap();
    assert!(world
        .set_property(
            mesh,
            &tint_path,
            ScenePropertyValue::Vec4([0.25, 0.5, 0.75, 1.0]),
        )
        .unwrap());
    assert!(world.has_pending_scene_systems());
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    let morph_path = ComponentPropertyPath::parse("MeshRenderer.morph_weights.2").unwrap();
    assert!(world
        .set_property(mesh, &morph_path, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    assert_eq!(
        world.get::<MeshRenderer>(mesh).unwrap().morph_weights,
        vec![0.0; 3]
    );
    assert!(world.has_pending_scene_systems());
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(!world
        .set_property(mesh, &morph_path, ScenePropertyValue::Scalar(0.0))
        .unwrap());
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn active_camera_selection_marks_render_extract_freshness_without_rebuilding_scheduler() {
    let mut world = World::new();
    let original_camera = world.active_camera();
    let second_camera = world.spawn_node(NodeKind::Camera);
    world
        .update_transform(
            second_camera,
            Transform::from_translation(Vec3::new(11.0, 0.0, 0.0)),
        )
        .unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());

    world.set_active_camera(original_camera);
    assert!(!world.has_pending_scene_systems());
    world.set_active_camera(second_camera);
    assert!(world.has_pending_scene_systems());

    let frame = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(501),
        SceneViewportExtractRequest::default(),
    ));
    assert_eq!(
        frame.view.camera.transform.translation,
        Vec3::new(11.0, 0.0, 0.0)
    );
    assert!(!world.has_pending_scene_systems());
}

fn detached_node_record(id: u64, kind: NodeKind) -> NodeRecord {
    let mut source = World::empty();
    let entity = source.spawn_node(kind);
    let mut record = source.node_record(entity).unwrap();
    record.id = id;
    record.name = format!("Imported {id}");
    record
}

fn pending_reparented_world() -> World {
    let mut world = World::new();
    let first_parent = world.spawn_node(NodeKind::Cube);
    let second_parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);
    world
        .update_transform(
            first_parent,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(
            second_parent,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(first_parent)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world
        .set_parent_checked(child, Some(second_parent))
        .unwrap();
    world.set_active_self(second_parent, false).unwrap();
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(12.0, 0.0, 0.0)
    );
    world
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
