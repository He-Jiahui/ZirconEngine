use super::*;

#[test]
fn interaction_and_sequence_ids_do_not_wrap() {
    let mut owner = PlayGizmoInteractionController {
        next_interaction_id: Some(u64::MAX),
        projection: None,
        active: None,
    };

    assert_eq!(owner.take_interaction_id().unwrap(), u64::MAX);
    assert!(matches!(
        owner.take_interaction_id(),
        Err(PlayGizmoError::InteractionIdExhausted)
    ));
}

#[test]
fn hover_never_consumes_the_scene_pick_path() {
    assert!(!PlayGizmoPointerOutcome::Hover {
        axis: Some(GizmoAxis::X),
        changed: true,
    }
    .consumed());
    assert!(PlayGizmoPointerOutcome::Began { axis: GizmoAxis::X }.consumed());
}
