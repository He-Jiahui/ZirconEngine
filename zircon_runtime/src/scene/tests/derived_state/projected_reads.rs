use super::*;
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
fn derived_state_projected_reads_use_direct_parent_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("derived_state.rs"),
    );
    let world_matrix = source
        .split("pub(super) fn project_world_matrix_for_read")
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
        world_matrix.contains("let mut lineage = Vec::new();")
            && world_matrix.contains("let mut seen = HashSet::new();")
            && world_matrix.contains("loop {")
            && world_matrix.contains("let Some(parent) = self.parent_for_read(current) else")
            && world_matrix.contains("for current in lineage.iter().rev().copied()")
            && world_matrix.contains(
                "world = world * transform_to_mat4(self.local_transform_value(current));"
            )
            && world_matrix.contains("Some(world)")
            && !world_matrix.contains("project_world_matrix_for_read_inner")
            && !world_matrix.contains(".map(|parent|")
            && !world_matrix.contains(".unwrap_or(Some(local_matrix))"),
        "projected world-matrix reads must compose the parent lineage iteratively without a recursive ancestor walk"
    );
    assert!(
        parent_for_read.contains("let Some(hierarchy) = self.get::<Hierarchy>(entity) else")
            && parent_for_read.contains("let Some(parent) = hierarchy.parent else")
            && parent_for_read.contains("if parent == entity || !self.contains_entity(parent)")
            && parent_for_read.contains("Some(parent)")
            && !parent_for_read.contains(".and_then(|hierarchy| hierarchy.parent)")
            && !parent_for_read.contains(".filter("),
        "parent_for_read must resolve valid parents through direct Option branches"
    );
    assert!(
        active_chain.contains("let mut seen = HashSet::new();")
            && active_chain.contains("loop {")
            && active_chain
                .contains("if !seen.insert(current) || !self.active_self_value(current)")
            && active_chain.contains("let Some(parent) = self.parent_for_read(current) else")
            && active_chain.contains("return false;")
            && !active_chain.contains(".map(|parent|")
            && !active_chain.contains(".unwrap_or(true)"),
        "active-chain reads must iterate optional parent state without recursive ancestry calls"
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
    assert!(
        source.contains("self.replace_derived_component(entity, WorldMatrix(world));")
            && source
                .contains("self.replace_derived_component(entity, ActiveInHierarchy(active));")
            && !source.contains("self.world_matrices")
            && !world_owner.contains("world_matrices:")
            && !world_owner.contains("active_in_hierarchy:")
            && !fixed_owner.contains("world_matrices")
            && !fixed_owner.contains("active_in_hierarchy")
            && !fixed_owner.contains("TypeId::of::<WorldMatrix>()")
            && !fixed_owner.contains("TypeId::of::<ActiveInHierarchy>()"),
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
    let world_matrix = source
        .split("pub(super) fn project_world_matrix_for_read")
        .nth(1)
        .and_then(|text| text.split("fn parent_for_read").next())
        .expect("read project_world_matrix_for_read body");
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
        world_matrix.contains("let mut lineage = Vec::new();")
            && world_matrix.contains("for current in lineage.iter().rev().copied()")
            && world_matrix.contains("self.local_transform_value(current)")
            && propagate_world.contains("let local = self.local_transform_value(entity);")
            && refresh.contains("let Some(name) = self.get::<Name>(entity) else")
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
            && refresh.contains("for entity in self.stable_entity_ids()")
            && refresh.contains("self.node_cache.push(SceneNode")
            && refresh.contains("parent: self.parent_of(entity)")
            && !refresh.contains("self.node_cache = self")
            && !refresh.contains(".filter_map(")
            && !refresh.contains(".collect()"),
        "refresh_node_cache must reuse retained cache storage with direct pushes instead of assigning a freshly collected Vec"
    );
}
