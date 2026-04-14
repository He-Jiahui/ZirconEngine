slint::include_modules!();

use slint::{ComponentHandle, PhysicalSize};

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn configure_shell(ui: &WorkbenchShell) {
    ui.show().expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(1440, 900));

    ui.set_drawers_visible(true);
    ui.set_center_band_frame(frame(0.0, 50.0, 1440.0, 830.0));
    ui.set_status_bar_frame(frame(0.0, 880.0, 1440.0, 20.0));
    ui.set_document_region_frame(frame(34.0, 50.0, 1314.0, 666.0));
    ui.set_right_region_frame(frame(1348.0, 50.0, 0.0, 666.0));
    ui.set_bottom_region_frame(frame(0.0, 788.0, 1440.0, 0.0));
    ui.set_drag_active(true);
}

#[test]
fn workbench_shell_allows_dragging_into_empty_right_region() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = WorkbenchShell::new().expect("workbench shell should instantiate");
    configure_shell(&ui);

    ui.set_drag_pointer_x(1428.0);
    ui.set_drag_pointer_y(240.0);

    assert_eq!(ui.get_active_drag_target_group().as_str(), "right");
}

#[test]
fn workbench_shell_allows_dragging_into_empty_bottom_region() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = WorkbenchShell::new().expect("workbench shell should instantiate");
    configure_shell(&ui);

    ui.set_drag_pointer_x(720.0);
    ui.set_drag_pointer_y(860.0);

    assert_eq!(ui.get_active_drag_target_group().as_str(), "bottom");
}

#[test]
fn workbench_shell_prefers_right_target_in_bottom_right_overlap_when_pointer_is_closer_to_right_edge()
{
    i_slint_backend_testing::init_no_event_loop();

    let ui = WorkbenchShell::new().expect("workbench shell should instantiate");
    configure_shell(&ui);

    ui.set_drag_pointer_x(1428.0);
    ui.set_drag_pointer_y(860.0);

    assert_eq!(ui.get_active_drag_target_group().as_str(), "right");
}

#[test]
fn workbench_shell_prefers_bottom_target_in_bottom_right_overlap_when_pointer_is_closer_to_bottom_edge()
{
    i_slint_backend_testing::init_no_event_loop();

    let ui = WorkbenchShell::new().expect("workbench shell should instantiate");
    configure_shell(&ui);

    ui.set_drag_pointer_x(1380.0);
    ui.set_drag_pointer_y(860.0);

    assert_eq!(ui.get_active_drag_target_group().as_str(), "bottom");
}
