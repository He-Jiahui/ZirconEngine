use super::*;

fn xyz(degrees: [f32; 3]) -> Quat {
    let [x, y, z] = degrees.map(f32::to_radians);
    Quat::from_euler(EulerRot::XYZ, x, y, z)
}

fn same_orientation(actual: Quat, expected: Quat) {
    assert!(actual.is_finite() && actual.is_normalized());
    assert!(
        actual.dot(expected).abs() > 0.999999,
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn rotation_each_axis_replaces_local_degrees_without_rounding_other_axes() {
    let original = [13.123456, 24.234567, -35.345678];
    for axis in 0..3 {
        let mut updates = [None; 3];
        updates[axis] = Some(47.125);
        let actual = with_local_xyz_degrees(xyz(original), updates).unwrap();
        let mut expected = original;
        expected[axis] = 47.125;
        same_orientation(actual, xyz(expected));
        let degrees = local_xyz_degrees(actual).unwrap();
        for other in 0..3 {
            assert!((degrees[other] - expected[other]).abs() < 0.00002);
        }
    }
}

#[test]
fn rotation_unchanged_display_and_whole_turns_preserve_quaternion_bits() {
    let original = -xyz([13.123456, 24.234567, -35.345678]) * 2.0;
    let displayed = local_xyz_degrees(original)
        .unwrap()
        .map(|value| format!("{value:.2}").parse::<f32>().unwrap());
    for axis in 0..3 {
        for turns in [0.0, 360.0, -360.0] {
            let mut updates = [None; 3];
            // Use exact whole-degree display values for the whole-turn identity check.
            let (rotation, value) = if turns == 0.0 {
                (original, displayed[axis])
            } else {
                (Quat::IDENTITY, turns)
            };
            updates[axis] = Some(value);
            let actual = with_local_xyz_degrees(rotation, updates).unwrap();
            assert_eq!(
                actual.to_array().map(f32::to_bits),
                rotation.to_array().map(f32::to_bits)
            );
        }
    }
}

#[test]
fn rotation_multiple_axis_updates_keep_unchanged_display_axes_at_full_precision() {
    let original = xyz([13.123456, 24.234567, -35.345678]);
    let actual = with_local_xyz_degrees(original, [Some(13.12), Some(45.0), None]).unwrap();
    let angles = local_xyz_degrees(actual).unwrap();
    assert!((angles[0] - 13.123456).abs() < 0.00002);
    assert!((angles[2] + 35.345678).abs() < 0.00002);
    same_orientation(actual, xyz([13.123456, 45.0, -35.345678]));
}

#[test]
fn rotation_invalid_quaternions_and_nonfinite_degrees_are_rejected() {
    for invalid in [
        Quat::from_array([0.0; 4]),
        Quat::from_array([f32::NAN, 0.0, 0.0, 1.0]),
        Quat::from_array([f32::INFINITY, 0.0, 0.0, 1.0]),
        Quat::from_array([f32::MAX; 4]),
    ] {
        assert!(local_xyz_degrees(invalid).is_none());
        assert!(with_local_xyz_degrees(invalid, [Some(30.0); 3]).is_none());
    }
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(with_local_xyz_degrees(Quat::IDENTITY, [Some(invalid), None, None]).is_none());
    }
}

#[test]
fn rotation_wraparound_and_large_finite_degrees_produce_normalized_storage() {
    for degrees in [-450.0, -180.0, 180.0, 450.0, f32::MAX, -f32::MAX] {
        let actual = with_local_xyz_degrees(Quat::IDENTITY, [None, None, Some(degrees)]).unwrap();
        same_orientation(
            actual,
            Quat::from_rotation_z(degrees.rem_euclid(360.0).to_radians()),
        );
    }
}

#[test]
fn rotation_tiny_negative_edits_are_not_lost_during_angle_wrapping() {
    let angle = -0.00001_f32;
    let actual = with_local_xyz_degrees(Quat::IDENTITY, [None, None, Some(angle)]).unwrap();
    let expected = Quat::from_rotation_z(angle.to_radians());
    assert!((actual.z - expected.z).abs() < 1e-12);
    assert_ne!(actual, Quat::IDENTITY);
}

#[test]
fn rotation_singularity_uses_displayed_canonical_xyz_and_unchanged_submit_is_exact() {
    for pitch in [-90.0, -89.999, 89.999, 90.0] {
        let original = xyz([31.23456, pitch, -27.34567]);
        let degrees = local_xyz_degrees(original).unwrap();
        same_orientation(xyz(degrees), original);
        let displayed = degrees.map(|value| Some(format!("{value:.2}").parse().unwrap()));
        assert_eq!(with_local_xyz_degrees(original, displayed), Some(original));
        let actual = with_local_xyz_degrees(original, [None, Some(45.0), None]).unwrap();
        same_orientation(actual, xyz([degrees[0], 45.0, degrees[2]]));
    }
    for pitch in [89.9, 90.0, 90.1] {
        let actual =
            with_local_xyz_degrees(xyz([30.0, 80.0, -20.0]), [None, Some(pitch), None]).unwrap();
        same_orientation(actual, xyz([30.0, pitch, -20.0]));
    }
}
