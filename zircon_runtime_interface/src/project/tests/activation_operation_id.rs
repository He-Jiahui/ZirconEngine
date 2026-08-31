use uuid::Uuid;

use super::super::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator,
    ProjectActivationOperationSequence, ProjectLaunchInstanceId,
};

#[test]
fn activation_operation_id_generator_keeps_origin_and_allocates_monotonic_distinct_ids() {
    let origin = ProjectLaunchInstanceId::try_from_uuid(
        Uuid::parse_str("b19a3f59-1fa7-4d89-b46f-7eca1d1f9846").unwrap(),
    )
    .unwrap();
    let generator = ProjectActivationOperationIdGenerator::new(origin);

    let first = generator.allocate().unwrap();
    let second = generator.allocate().unwrap();

    assert_eq!(first.origin_instance(), origin);
    assert_eq!(second.origin_instance(), origin);
    assert_eq!(first.sequence().get(), 1);
    assert_eq!(second.sequence().get(), 2);
    assert_ne!(first.nonce(), second.nonce());
}

#[test]
fn activation_operation_sequence_rejects_zero() {
    assert_eq!(ProjectActivationOperationSequence::new(0), None);
    assert_eq!(ProjectActivationOperationSequence::new(1).unwrap().get(), 1);
}

#[test]
fn activation_operation_id_preserves_explicit_transport_fields() {
    let origin = ProjectLaunchInstanceId::try_from_uuid(
        Uuid::parse_str("a61b09c9-5901-4d65-8a56-fd3bb980c7a8").unwrap(),
    )
    .unwrap();
    let sequence = ProjectActivationOperationSequence::new(73).unwrap();
    let nonce = Uuid::parse_str("35ef8c93-77b3-4b4a-b96d-514d4baabf77").unwrap();

    let operation = ProjectActivationOperationId::try_from_parts(origin, sequence, nonce).unwrap();

    assert_eq!(operation.origin_instance(), origin);
    assert_eq!(operation.sequence(), sequence);
    assert_eq!(operation.nonce(), nonce);
}

#[test]
fn operation_identity_rejects_invalid_or_unrecognized_wire_components() {
    let origin = ProjectLaunchInstanceId::try_from_uuid(
        Uuid::parse_str("c2d9f11d-7800-4b5f-8e7b-45b2394f79b5").unwrap(),
    )
    .unwrap();
    let sequence = ProjectActivationOperationSequence::new(1).unwrap();
    let nonce = Uuid::parse_str("390bc691-a2ac-41dc-a9ed-af4b911310ea").unwrap();
    let operation = ProjectActivationOperationId::try_from_parts(origin, sequence, nonce).unwrap();

    assert!(ProjectLaunchInstanceId::try_from_uuid(Uuid::nil()).is_err());
    assert!(ProjectActivationOperationId::try_from_parts(origin, sequence, Uuid::nil()).is_err());

    let invalid_origin = serde_json::json!({
        "origin_instance": Uuid::nil(),
        "sequence": 1,
        "nonce": nonce,
    });
    assert!(serde_json::from_value::<ProjectActivationOperationId>(invalid_origin).is_err());

    let invalid_sequence = serde_json::json!({
        "origin_instance": origin.as_uuid(),
        "sequence": 0,
        "nonce": nonce,
    });
    assert!(serde_json::from_value::<ProjectActivationOperationId>(invalid_sequence).is_err());

    let mut with_unknown_field = serde_json::to_value(operation).unwrap();
    with_unknown_field
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_string(), serde_json::Value::Null);
    assert!(serde_json::from_value::<ProjectActivationOperationId>(with_unknown_field).is_err());
}
