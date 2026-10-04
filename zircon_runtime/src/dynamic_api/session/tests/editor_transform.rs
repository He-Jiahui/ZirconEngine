use super::*;
use crate::core::math::{Quat, Vec3};
use crate::scene::components::NodeKind;

fn transform(x: f32) -> Transform {
    Transform {
        translation: Vec3::new(x, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }
}

fn request(
    entity: u64,
    interaction_id: u64,
    sequence: u64,
    epoch: u64,
    phase: ZrRuntimeEditorTransformPhaseV1,
    expected: Transform,
    target: Transform,
) -> ZrRuntimeEditorTransformWriteV1 {
    ZrRuntimeEditorTransformWriteV1::new(
        entity,
        interaction_id,
        sequence,
        epoch,
        phase,
        expected,
        target,
    )
}

#[test]
fn preview_commit_is_ordered_and_compare_and_set() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let initial = world.local_transform(entity).unwrap();
    let preview = transform(2.0);
    let committed = transform(3.0);
    let mut state = RuntimeEditorTransformState::default();

    state
        .handle(
            &mut world,
            7,
            request(
                entity,
                11,
                1,
                7,
                ZrRuntimeEditorTransformPhaseV1::Begin,
                initial,
                initial,
            ),
        )
        .unwrap();
    assert_eq!(world.local_transform(entity), Some(initial));
    state
        .handle(
            &mut world,
            7,
            request(
                entity,
                11,
                2,
                7,
                ZrRuntimeEditorTransformPhaseV1::Preview,
                initial,
                preview,
            ),
        )
        .unwrap();
    assert_eq!(world.local_transform(entity), Some(preview));
    assert!(matches!(
        state.handle(
            &mut world,
            7,
            request(
                entity,
                11,
                4,
                7,
                ZrRuntimeEditorTransformPhaseV1::Commit,
                preview,
                committed,
            ),
        ),
        Err(RuntimeEditorTransformWriteError::SequenceMismatch { .. })
    ));
    state
        .handle(
            &mut world,
            7,
            request(
                entity,
                11,
                3,
                7,
                ZrRuntimeEditorTransformPhaseV1::Commit,
                preview,
                committed,
            ),
        )
        .unwrap();
    assert_eq!(world.local_transform(entity), Some(committed));
}

#[test]
fn cancel_restores_initial_and_world_replacement_retires_owner() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let initial = world.local_transform(entity).unwrap();
    let preview = transform(2.0);
    let mut state = RuntimeEditorTransformState::default();
    state
        .handle(
            &mut world,
            7,
            request(
                entity,
                11,
                1,
                7,
                ZrRuntimeEditorTransformPhaseV1::Begin,
                initial,
                initial,
            ),
        )
        .unwrap();
    state
        .handle(
            &mut world,
            7,
            request(
                entity,
                11,
                2,
                7,
                ZrRuntimeEditorTransformPhaseV1::Preview,
                initial,
                preview,
            ),
        )
        .unwrap();
    state
        .handle(
            &mut world,
            7,
            request(
                entity,
                11,
                3,
                7,
                ZrRuntimeEditorTransformPhaseV1::Cancel,
                preview,
                initial,
            ),
        )
        .unwrap();
    assert_eq!(world.local_transform(entity), Some(initial));

    state
        .handle(
            &mut world,
            7,
            request(
                entity,
                13,
                1,
                7,
                ZrRuntimeEditorTransformPhaseV1::Begin,
                initial,
                initial,
            ),
        )
        .unwrap();
    assert!(matches!(
        state.handle(
            &mut world,
            8,
            request(
                entity,
                13,
                2,
                7,
                ZrRuntimeEditorTransformPhaseV1::Preview,
                initial,
                preview,
            ),
        ),
        Err(RuntimeEditorTransformWriteError::WorldReplaced { .. })
    ));
    assert!(matches!(
        state.handle(
            &mut world,
            8,
            request(
                entity,
                13,
                3,
                8,
                ZrRuntimeEditorTransformPhaseV1::Cancel,
                initial,
                initial,
            ),
        ),
        Err(RuntimeEditorTransformWriteError::InteractionMissing)
    ));
    state
        .handle(
            &mut world,
            8,
            request(
                entity,
                17,
                1,
                8,
                ZrRuntimeEditorTransformPhaseV1::Begin,
                initial,
                initial,
            ),
        )
        .unwrap();
}
