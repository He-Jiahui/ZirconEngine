slint::include_modules!();

use slint::{ComponentHandle, PhysicalSize};

#[test]
fn workbench_shell_window_can_resize_and_toggle_maximize() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("host window should instantiate");
    ui.show().expect("host window should show in test backend");

    let initial = ui.window().size();
    assert!(initial.width > 0);
    assert!(initial.height > 0);

    ui.window()
        .set_size(PhysicalSize::new(initial.width + 120, initial.height + 80));

    let resized = ui.window().size();
    assert_eq!(resized.width, initial.width + 120);
    assert_eq!(resized.height, initial.height + 80);
    assert_eq!(ui.get_shell_width_px(), resized.width as f32);
    assert_eq!(ui.get_shell_height_px(), resized.height as f32);

    assert!(!ui.window().is_maximized());
    ui.window().set_maximized(true);
    assert!(ui.window().is_maximized());
    ui.window().set_maximized(false);
    assert!(!ui.window().is_maximized());
}
