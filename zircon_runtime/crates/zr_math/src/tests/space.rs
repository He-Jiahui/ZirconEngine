use crate::{
    Normal3, Position3, SpaceKind, SpatialError, UnitDirection3, ValidatedTransform, Vec3, Vector3,
};

#[test]
fn position_and_vector_operations_require_a_matching_space() {
    let world = Position3::try_new(Vec3::new(1.0, 2.0, 3.0), SpaceKind::World)
        .expect("finite world position is valid");
    let delta = Vector3::try_new(Vec3::new(2.0, -1.0, 4.0), SpaceKind::World)
        .expect("finite world vector is valid");
    let moved = world
        .checked_add(delta)
        .expect("matching spaces can compose");
    assert_eq!(moved.value(), Vec3::new(3.0, 1.0, 7.0));

    let local =
        Position3::try_new(Vec3::ZERO, SpaceKind::Local).expect("finite local position is valid");
    assert!(matches!(
        world.checked_sub(local),
        Err(SpatialError::SpaceMismatch {
            left: SpaceKind::World,
            right: SpaceKind::Local,
        })
    ));
}

#[test]
fn normal_rejects_zero_input_and_validated_types_reject_invalid_wire_values() {
    assert!(Normal3::try_new(Vec3::ZERO, SpaceKind::World).is_err());

    let zero_direction = serde_json::to_value(Vec3::ZERO).expect("Vec3 serializes");
    assert!(serde_json::from_value::<UnitDirection3>(zero_direction).is_err());

    let zero_scale = serde_json::json!({
        "translation": [0.0, 0.0, 0.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": [1.0, 0.0, 1.0],
    });
    assert!(serde_json::from_value::<ValidatedTransform>(zero_scale).is_err());
}
