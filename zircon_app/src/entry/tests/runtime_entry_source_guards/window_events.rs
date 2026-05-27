use super::super::source_assertions::assert_source_order;
use super::sources::{
    entry_root, runtime_application_handler_source, runtime_window_events_source,
};

#[test]
fn runtime_entry_window_event_dispatch_stays_in_child_module() {
    let runtime_app_source = include_str!("../../runtime_entry_app/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_events_root_source =
        include_str!("../../runtime_entry_app/window_events/mod.rs");
    let root = entry_root();

    assert!(
        runtime_app_source.contains("mod window_events;"),
        "runtime entry app should declare a child module for concrete winit window-event dispatch"
    );
    assert!(
        runtime_window_events_root_source.contains("mod dispatch;"),
        "runtime window-events root should stay structural and delegate dispatch"
    );
    assert!(
        !root.join("runtime_entry_app/window_events.rs").exists(),
        "runtime window events should stay folder-backed instead of returning to an umbrella window_events.rs file"
    );
    assert_source_order(
        runtime_handler_source,
        &[
            "fn window_event",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"window_event\");",
            "self.handle_window_event(event_loop, event);",
        ],
        "ApplicationHandler::window_event should only profile and delegate concrete dispatch",
    );
    for forbidden in [
        "WindowEvent::CloseRequested",
        "WindowEvent::Destroyed",
        "WindowEvent::Moved",
        "WindowEvent::Occluded",
        "WindowEvent::ThemeChanged",
        "WindowEvent::ScaleFactorChanged",
        "WindowEvent::SurfaceResized",
        "WindowEvent::Focused",
        "WindowEvent::PointerEntered",
        "WindowEvent::PointerLeft",
        "WindowEvent::DragEntered",
        "WindowEvent::DragDropped",
        "WindowEvent::DragLeft",
        "WindowEvent::PointerMoved",
        "WindowEvent::PointerButton",
        "WindowEvent::KeyboardInput",
        "WindowEvent::Ime",
        "WindowEvent::MouseWheel",
        "WindowEvent::RedrawRequested",
    ] {
        assert!(
            !runtime_handler_source.contains(forbidden),
            "ApplicationHandler should not own concrete window-event arm `{forbidden}`"
        );
    }
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "fn handle_window_event",
            "WindowEvent::CloseRequested",
            "self.handle_window_close_requested(event_loop);",
            "WindowEvent::Destroyed",
            "self.handle_window_destroyed(event_loop);",
            "WindowEvent::Moved(position)",
            "self.handle_window_moved(event_loop, position);",
            "WindowEvent::Occluded(occluded)",
            "self.handle_window_occluded(event_loop, occluded);",
            "WindowEvent::ThemeChanged(theme)",
            "self.handle_window_theme_changed(event_loop, theme);",
            "WindowEvent::ScaleFactorChanged { scale_factor, .. }",
            "self.handle_window_scale_factor_changed(event_loop, scale_factor);",
            "WindowEvent::SurfaceResized(size)",
            "self.resize_surface_presenter(event_loop, size);",
            "WindowEvent::Focused(focused)",
            "self.handle_window_focus_changed(event_loop, focused);",
            "WindowEvent::PointerEntered",
            "self.handle_pointer_entered(event_loop);",
            "WindowEvent::PointerLeft { position, kind, .. }",
            "self.handle_pointer_left(event_loop, position, kind);",
            "WindowEvent::DragEntered { paths, .. }",
            "self.handle_files_hovered(event_loop, paths);",
            "WindowEvent::DragDropped { paths, .. }",
            "self.handle_files_dropped(event_loop, paths);",
            "WindowEvent::DragLeft { .. }",
            "self.handle_file_drag_cancelled(event_loop);",
            "WindowEvent::PointerMoved",
            "self.handle_pointer_moved(event_loop, position, source);",
            "WindowEvent::PointerButton",
            "self.handle_pointer_button(event_loop, state, button, position);",
            "WindowEvent::KeyboardInput { event, .. }",
            "self.handle_keyboard_input(event_loop, event);",
            "WindowEvent::Ime(ime)",
            "self.handle_ime_input(event_loop, ime);",
            "WindowEvent::MouseWheel { delta, .. }",
            "self.handle_mouse_wheel(event_loop, delta);",
            "WindowEvent::RedrawRequested",
            "self.present_redraw_frame(event_loop);",
        ],
        "runtime window-event dispatcher should preserve every concrete winit arm and delegate to focused child behavior modules",
    );
}
