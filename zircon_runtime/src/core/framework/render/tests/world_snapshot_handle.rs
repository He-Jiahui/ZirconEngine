use super::RenderWorldSnapshotHandle;

#[test]
fn snapshot_handle_keeps_world_identity_and_source_generation_distinct() {
    let snapshot = RenderWorldSnapshotHandle::new(7).with_generation(19);

    assert_eq!(snapshot.raw(), 7);
    assert_eq!(snapshot.generation(), 19);
}
