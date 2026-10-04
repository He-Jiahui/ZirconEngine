use super::*;

#[test]
fn saved_position_remains_on_its_monitor_and_clamps_to_work_area() {
    let left_monitor = tauri::PhysicalRect {
        position: tauri::PhysicalPosition::new(-1920, 0),
        size: tauri::PhysicalSize::new(1920, 1040),
    };
    let normal_size = tauri::PhysicalSize::new(1280, 800);

    assert!(position_in_work_area((-1500, 120), &left_monitor));
    assert!(!position_in_work_area((200, 120), &left_monitor));
    assert_eq!(
        clamp_position_to_work_area((-1500, 120), normal_size, &left_monitor),
        tauri::PhysicalPosition::new(-1500, 120)
    );
    assert_eq!(
        clamp_position_to_work_area((-200, 600), normal_size, &left_monitor),
        tauri::PhysicalPosition::new(-1280, 240)
    );
}

#[test]
fn saved_position_outside_a_removed_monitor_is_rejected() {
    let current_monitor = tauri::PhysicalRect {
        position: tauri::PhysicalPosition::new(0, 0),
        size: tauri::PhysicalSize::new(1600, 900),
    };
    assert!(!position_in_work_area((-4000, 240), &current_monitor));
    assert!(position_in_work_area((100, 200), &current_monitor));
}

#[test]
fn saved_hub_size_restores_in_logical_pixels_and_respects_native_minimum() {
    let mut config = HubConfig::default();
    let native = WindowConfig {
        width: 1568.0,
        height: 1003.0,
        min_width: Some(960.0),
        min_height: Some(680.0),
        ..WindowConfig::default()
    };

    assert_eq!(saved_logical_size(&config, &native, None), None);
    assert_eq!(
        saved_logical_size(&config, &native, Some(LogicalSize::new(1200.0, 720.0))),
        Some(LogicalSize::new(1200.0, 720.0))
    );

    config.window.position_x = Some(-1500);
    config.window.position_y = Some(100);
    assert_eq!(
        saved_logical_size(&config, &native, Some(LogicalSize::new(1200.0, 720.0))),
        Some(LogicalSize::new(1200.0, 720.0))
    );
    config.window.position_x = None;
    config.window.position_y = None;

    config.window.width = Some(1280);
    config.window.height = Some(800);
    assert_eq!(
        saved_logical_size(&config, &native, None),
        Some(LogicalSize::new(1280.0, 800.0))
    );

    config.window.width = Some(320);
    config.window.height = Some(240);
    assert_eq!(
        saved_logical_size(&config, &native, None),
        Some(LogicalSize::new(960.0, 680.0))
    );

    config.window.width = Some(0);
    config.window.height = Some(900);
    assert_eq!(
        saved_logical_size(&config, &native, None),
        Some(LogicalSize::new(1568.0, 900.0))
    );

    config.window.width = Some(2000);
    config.window.height = Some(1400);
    assert_eq!(
        saved_logical_size(&config, &native, Some(LogicalSize::new(1200.0, 720.0))),
        Some(LogicalSize::new(1200.0, 720.0))
    );
    assert_eq!(
        saved_logical_size(&config, &native, Some(LogicalSize::new(800.0, 600.0))),
        Some(LogicalSize::new(960.0, 680.0))
    );
}
