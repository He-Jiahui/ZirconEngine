use super::*;

#[test]
fn view_state_preserves_finite_camera_and_history_invalidation() {
    let input = HybridGiRuntimePrepareInput::new(None, &[], &[], &[], &[], None, false, None, 7)
        .with_view_state(Some(Vec3::new(1.0, 2.0, 3.0)), true);

    assert_eq!(input.camera_position(), Some(Vec3::new(1.0, 2.0, 3.0)));
    assert!(input.history_invalidated());
    assert_eq!(input.generation(), 7);
}

#[test]
fn view_state_discards_nonfinite_camera_without_changing_history_semantics() {
    let input = HybridGiRuntimePrepareInput::new(None, &[], &[], &[], &[], None, false, None, 3)
        .with_view_state(Some(Vec3::new(f32::NAN, 0.0, 0.0)), false);

    assert_eq!(input.camera_position(), None);
    assert!(!input.history_invalidated());
    assert_eq!(input.generation(), 3);
}

#[test]
fn hybrid_gi_prepare_input_source_world_is_explicit_and_preserves_generation() {
    use crate::core::framework::render::RenderWorldSnapshotHandle;
    let input = HybridGiRuntimePrepareInput::new(None, &[], &[], &[], &[], None, false, None, 99);
    assert_eq!(input.source_world(), None);
    let world = RenderWorldSnapshotHandle::new(7).with_generation(13);
    let input = input.with_source_world(world).with_view_state(None, false);
    assert_eq!(input.source_world(), Some(world));
    assert_eq!(input.source_world().unwrap().generation(), 13);
    assert_eq!(input.generation(), 99);
}
