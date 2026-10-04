use super::*;
use crate::scene::{
    dynamic_scene::{session::RuntimeSessionMetadata, DynamicResource, DynamicScene},
    NodeKind, World,
};

#[test]
fn runtime_session_archive_payload_limit_matrix_stops_stream_writes_at_bound() {
    let chunk = [0u8; 64 * 1024];
    for mebibytes in [1usize, 64, 512] {
        let limit = mebibytes * 1024 * 1024;
        let mut budget = ArchiveByteBudget::new(limit);
        for _ in 0..limit / chunk.len() {
            budget
                .write_all(&chunk)
                .expect("stream should accept bytes through its exact limit");
        }
        assert_eq!(budget.written_bytes, limit);
        assert!(budget.write_all(&[0]).is_err());
        assert_eq!(budget.overflow_at, Some(limit.saturating_add(1)));
    }
    assert_eq!(
        MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES,
        512 * 1024 * 1024
    );
}

#[test]
fn runtime_session_archive_slot_and_entity_scale_matrix_builds_linear_indexes() {
    let mut source = World::empty();
    source
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    let mut scene =
        DynamicScene::from_world(&source).expect("source scene should capture one real entity");
    scene.resources.push(DynamicResource::new(
        "zircon_runtime::tests::ArchiveScaleResource",
        Vec::new(),
    ));
    let template = RuntimeSessionSlot {
        slot_id: "template".to_owned(),
        metadata: RuntimeSessionMetadata::default(),
        scene,
    };

    for count in [1usize, 1_000, 100_000] {
        let payload = RuntimeSessionArchivePayload::new(
            super::super::RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
            (0..count)
                .map(|index| {
                    let mut slot = template.clone();
                    slot.slot_id = format!("slot-{index:06}");
                    slot
                })
                .collect(),
        );
        let manifest = build_manifest(&payload);
        let index = build_slot_index(&manifest);
        assert_eq!(manifest.slot_count(), count);
        assert_eq!(index.len(), count);

        let statistics = build_statistics(&payload);
        assert_eq!(statistics.total_entity_count, count);
        assert_eq!(statistics.total_resource_count, count);
        assert_eq!(statistics.max_slot_entity_count, 1);
        assert_eq!(statistics.max_slot_resource_count, 1);
    }
}
