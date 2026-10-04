//! 节点生成以可复制的 NodeKind 和缓存的种类计数完成默认记录构造，
//! 避免批量生成因重复扫描实体或复制路径对象而退化。

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
    // BUG: [CR-SCENE-TEST-CONTRACT-0001] 回归保护：读取迁移后的 components/scene/identity.rs，
    // 避免旧 components/scene.rs 路径在检查 NodeKind 的 Copy 契约前失败。
    let component_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("components")
            .join("scene")
            .join("identity.rs"),
    )
    .replace("\r\n", "\n");
    let component_source = component_source
        .lines()
        .filter(|line| !line.trim_start().starts_with("///"))
        .collect::<Vec<_>>()
        .join("\n");
    let bootstrap_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("bootstrap.rs"),
    );
    let records_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("records.rs"),
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
        .and_then(|text| text.split("pub(super) fn default_node_record").next())
        .expect("read spawn_node body");
    let default_record = bootstrap_source
        .split("pub(super) fn default_node_record")
        .nth(1)
        .and_then(|text| text.split("pub fn spawn_mesh_node").next())
        .expect("read default_node_record body");
    let insert_record = records_source
        .split("pub fn insert_node_record(")
        .nth(1)
        .and_then(|text| text.split("fn prepare_owned_node_record_batch(").next())
        .expect("read insert_node_record body");
    let insert_owned_records = records_source
        .split("pub fn insert_owned_node_records(")
        .nth(1)
        .and_then(|text| text.split("pub fn rename_node(").next())
        .expect("read insert_owned_node_records body");
    let prepare_records = records_source
        .split("fn prepare_owned_node_record_batch(")
        .nth(1)
        .and_then(|text| {
            text.split("pub(in crate::scene) fn validate_owned_node_records(")
                .next()
        })
        .expect("read prepare_owned_node_record_batch body");
    let commit_records = records_source
        .split("fn commit_node_record_batch(")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn insert_prevalidated_node_record(")
                .next()
        })
        .expect("read commit_node_record_batch body");
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
            && spawn_node.contains("self.insert_node_record(record)?;")
            && !spawn_node.contains("kind.clone()")
            && !default_record.contains("kind.clone()"),
        "spawn_node must reuse Copy NodeKind values while one prevalidated record transaction publishes the complete row"
    );
    assert!(
        insert_record.contains("self.insert_owned_node_records(vec![record])")
            && prepare_records.contains("self.validate_node_record_batch_input(&records)?;")
            && commit_records.contains("self.insert_prevalidated_node_record(record);"),
        "spawn records must reach prevalidated publication through the owned batch validation path"
    );
    let prepare_call = insert_owned_records
        .find("self.prepare_owned_node_record_batch(records)?")
        .expect("owned records must propagate preparation failure");
    let commit_call = insert_owned_records
        .find("self.commit_node_record_batch(batch)")
        .expect("owned records must publish through the prepared batch commit");
    assert!(
        prepare_call < commit_call,
        "owned records must validate before committing prevalidated rows"
    );
    assert!(
        node_kind.contains("self.kinds.get(&entity).copied()") && !node_kind.contains(".cloned()"),
        "node_kind lookups must copy stored NodeKind values instead of cloning them"
    );
}
