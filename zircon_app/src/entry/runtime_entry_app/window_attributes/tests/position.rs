use super::*;

#[test]
fn centered_physical_position_uses_monitor_origin_and_size() {
    let position = centered_physical_position(
        PhysicalPosition::new(-1920, 100),
        PhysicalSize::new(1920, 1080),
        PhysicalSize::new(800, 600),
    );

    assert_eq!(position, PhysicalPosition::new(-1360, 340));
}

#[test]
fn centered_physical_position_keeps_oversized_windows_at_monitor_origin() {
    let position = centered_physical_position(
        PhysicalPosition::new(12, -34),
        PhysicalSize::new(640, 480),
        PhysicalSize::new(800, 600),
    );

    assert_eq!(position, PhysicalPosition::new(12, -34));
}

#[test]
fn centered_physical_position_saturates_output_coordinates() {
    let position = centered_physical_position(
        PhysicalPosition::new(i32::MAX, i32::MIN),
        PhysicalSize::new(u32::MAX, u32::MAX),
        PhysicalSize::new(1, 1),
    );

    assert_eq!(position, PhysicalPosition::new(i32::MAX, -1));
}
