use woc_client::{
    apply_radial_deadzone, rising_edges, stick_to_look, stick_to_move_flags, GamepadLookDelta,
    GamepadMoveFlags, GamepadStickVector,
};

fn magnitude(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

#[test]
fn radial_deadzone_zeroes_resting_drift() {
    assert_eq!(
        apply_radial_deadzone(0.1, 0.05, 0.2),
        GamepadStickVector::default()
    );
    assert_eq!(
        apply_radial_deadzone(0.0, 0.0, 0.2),
        GamepadStickVector::default()
    );
}

#[test]
fn radial_deadzone_rescales_edge_and_full_deflection() {
    let just_out = apply_radial_deadzone(0.0, -0.2001, 0.2);
    assert!(magnitude(just_out.x, just_out.y) < 0.01);

    let full = apply_radial_deadzone(0.0, -1.0, 0.2);
    assert!((magnitude(full.x, full.y) - 1.0).abs() < 1.0e-6);
}

#[test]
fn radial_deadzone_clamps_square_corner_over_deflection() {
    let corner = apply_radial_deadzone(1.0, 1.0, 0.2);
    assert!((magnitude(corner.x, corner.y) - 1.0).abs() < 1.0e-6);
}

#[test]
fn left_stick_maps_cardinals_and_diagonals_to_move_flags() {
    assert_eq!(
        stick_to_move_flags(0.1, 0.1, 0.25),
        GamepadMoveFlags::default()
    );
    assert!(stick_to_move_flags(0.0, -1.0, 0.2).forward);
    assert!(stick_to_move_flags(0.0, 1.0, 0.2).back);

    assert_eq!(
        stick_to_move_flags(-0.9, -0.9, 0.2),
        GamepadMoveFlags {
            forward: true,
            back: false,
            strafe_left: true,
            strafe_right: false,
        }
    );
    assert_eq!(
        stick_to_move_flags(0.9, 0.9, 0.2),
        GamepadMoveFlags {
            forward: false,
            back: true,
            strafe_left: false,
            strafe_right: true,
        }
    );
}

#[test]
fn movement_uses_the_target_strict_deadzone_boundary() {
    assert_eq!(
        stick_to_move_flags(0.2, 0.0, 0.2),
        GamepadMoveFlags {
            forward: false,
            back: false,
            strafe_left: false,
            strafe_right: true,
        }
    );
}

#[test]
fn right_stick_look_is_deadzone_filtered_and_frame_scaled() {
    assert_eq!(
        stick_to_look(0.1, 0.1, 0.2, 2.0, false, 0.016),
        GamepadLookDelta::default()
    );

    let first = stick_to_look(1.0, 0.0, 0.2, 2.0, false, 0.016);
    let second = stick_to_look(1.0, 0.0, 0.2, 2.0, false, 0.032);
    assert!(first.yaw < 0.0);
    assert!((second.yaw - first.yaw * 2.0).abs() < 1.0e-6);
}

#[test]
fn right_stick_invert_y_flips_only_pitch() {
    let normal = stick_to_look(0.0, -1.0, 0.2, 2.0, false, 0.016);
    let inverted = stick_to_look(0.0, -1.0, 0.2, 2.0, true, 0.016);
    assert_eq!(normal.yaw, inverted.yaw);
    assert_eq!(normal.pitch, -inverted.pitch);
}

#[test]
fn rising_edges_reports_only_up_to_down_transitions() {
    assert_eq!(
        rising_edges(&[false, true, false], &[true, true, true]),
        vec![0, 2]
    );
    assert!(rising_edges(&[true, true], &[true, true]).is_empty());
    assert_eq!(rising_edges(&[], &[false, true]), vec![1]);
}
