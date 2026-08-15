use super::support::*;

#[test]
fn workbench_shell_window_starts_at_reference_size_and_can_resize() {
    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");

    let initial = ui.window().size();
    assert_eq!(initial, PhysicalSize::new(1672, 941));

    ui.window()
        .set_size(PhysicalSize::new(initial.width + 120, initial.height + 80));

    let resized = ui.window().size();
    let bootstrap = ui.get_host_window_bootstrap();
    assert_eq!(resized.width, initial.width + 120);
    assert_eq!(resized.height, initial.height + 80);
    assert_eq!(bootstrap.shell_frame.width, resized.width as f32);
    assert_eq!(bootstrap.shell_frame.height, resized.height as f32);

    assert!(!ui.window().is_maximized());
    ui.window().set_maximized(true);
    assert!(ui.window().is_maximized());
    ui.window().set_maximized(false);
    assert!(!ui.window().is_maximized());
}
