use super::*;

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
