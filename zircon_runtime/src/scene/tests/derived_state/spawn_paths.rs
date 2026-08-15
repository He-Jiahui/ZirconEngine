use super::*;

#[test]
fn spawn_node_kind_ordinals_use_cached_kind_counts() {
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
        ordinal_for.contains("self.node_kind_ordinals[node_kind_ordinal_index(kind)]")
            && !ordinal_for.contains("self.entities.iter()")
            && !ordinal_for.contains("kind.clone()"),
        "spawn-node kind ordinal lookup must use the cached count instead of scanning every entity"
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
        spawn_node.contains("let record = self.default_node_record(id, kind);")
            && spawn_node.contains("self.insert_prevalidated_node_record(record);")
            && !spawn_node.contains("kind.clone()"),
        "spawn_node must reuse Copy NodeKind values while one prevalidated record transaction publishes the complete row"
    );
    assert!(
        node_kind.contains("self.kinds.get(&entity).copied()") && !node_kind.contains(".cloned()"),
        "node_kind lookups must copy stored NodeKind values instead of cloning them"
    );
}
