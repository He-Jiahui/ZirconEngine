use super::*;
#[test]
fn local_xyz_quaternion_is_displayed_in_degrees_without_mutating_storage() {
    assert_eq!(
        InspectorSnapshot::rotation_degrees_from_quaternion(Quat::IDENTITY),
        Some(["0.00".to_string(), "0.00".to_string(), "0.00".to_string()])
    );
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        30_f32.to_radians(),
        10_f32.to_radians(),
        (-20_f32).to_radians(),
    );
    let expected = Some([
        "30.00".to_string(),
        "10.00".to_string(),
        "-20.00".to_string(),
    ]);
    assert_eq!(
        InspectorSnapshot::rotation_degrees_from_quaternion(rotation),
        expected
    );
    assert_eq!(
        InspectorSnapshot::rotation_degrees_from_quaternion(-rotation),
        expected
    );
}
#[test]
fn invalid_or_missing_quaternion_is_not_reported_as_identity() {
    assert_eq!(
        InspectorSnapshot::rotation_degrees_from_quaternion(Quat::from_array([0.0; 4])),
        None
    );
    assert_eq!(
        InspectorSnapshot::rotation_degrees_from_quaternion(Quat::from_array([
            f32::NAN,
            0.0,
            0.0,
            1.0
        ])),
        None
    );
}
