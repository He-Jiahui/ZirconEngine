use crate::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceRecord, ResourceState,
};

#[derive(Debug)]
struct TestPayload;

fn record(locator_text: &str, kind: ResourceKind) -> ResourceRecord {
    let locator = ResourceLocator::parse(locator_text).expect("valid locator");
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator)
}

#[test]
fn resource_manager_exports_ready_records_for_kind_with_live_revisions() {
    let manager = ResourceManager::new();

    let first_shader =
        record("res://shaders/live.wgsl", ResourceKind::Shader).with_source_hash("shader-hash-a");
    let shader_id = first_shader.id;
    manager.register_ready(first_shader, TestPayload).unwrap();
    manager
        .register_ready(
            record("res://shaders/live.wgsl", ResourceKind::Shader)
                .with_source_hash("shader-hash-b"),
            TestPayload,
        )
        .unwrap();
    manager
        .register_ready(
            record("res://models/mesh.glb", ResourceKind::Model).with_source_hash("model-hash"),
            TestPayload,
        )
        .unwrap();
    manager
        .register_record(record("res://shaders/pending.wgsl", ResourceKind::Shader))
        .unwrap();
    manager
        .register_record(
            record("res://shaders/error.wgsl", ResourceKind::Shader)
                .with_state(ResourceState::Error),
        )
        .unwrap();

    let records = manager.ready_records_for_kind(ResourceKind::Shader);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, shader_id);
    assert_eq!(records[0].kind, ResourceKind::Shader);
    assert_eq!(records[0].state, ResourceState::Ready);
    assert_eq!(records[0].revision, 2);
    assert_eq!(records[0].source_hash, "shader-hash-b");
}
