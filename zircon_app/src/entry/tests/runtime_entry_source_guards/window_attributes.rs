use super::super::source_assertions::assert_source_order;
use super::sources::runtime_application_handler_source;

#[test]
fn runtime_entry_window_attributes_use_monitor_aware_creation_policy() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_window_creation_source = include_str!("../../runtime_entry_app/window_creation.rs");
    let window_attributes_root_source =
        include_str!("../../runtime_entry_app/window_attributes/mod.rs");
    let window_attributes_builder_source =
        include_str!("../../runtime_entry_app/window_attributes/builder.rs");
    let window_attributes_fullscreen_source =
        include_str!("../../runtime_entry_app/window_attributes/fullscreen.rs");
    let window_attributes_monitor_source =
        include_str!("../../runtime_entry_app/window_attributes/monitor.rs");
    let window_attributes_position_source =
        include_str!("../../runtime_entry_app/window_attributes/position.rs");
    let window_attributes_video_mode_source =
        include_str!("../../runtime_entry_app/window_attributes/video_mode.rs");
    let window_attributes_source = [
        window_attributes_root_source,
        window_attributes_builder_source,
        window_attributes_fullscreen_source,
        window_attributes_monitor_source,
        window_attributes_position_source,
        window_attributes_video_mode_source,
    ]
    .join("\n");

    assert_source_order(
        runtime_handler_source,
        &[
            "fn can_create_surfaces",
            "self.create_primary_window_surface(event_loop);",
        ],
        "runtime entry ApplicationHandler should delegate primary window creation to the window-creation module",
    );
    assert_source_order(
        runtime_window_creation_source,
        &[
            "fn create_primary_window_surface",
            "runtime_window_attributes(&self.window_descriptor, event_loop)",
            "event_loop.create_window(window_attributes)",
        ],
        "runtime entry should provide active event-loop monitor context before creating the winit window",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "fn runtime_window_attributes(",
            "WindowMonitorContext::for_event_loop(event_loop)",
            "runtime_window_attributes_with_monitor_context(descriptor, &monitor_context)",
        ],
        "runtime window attributes should seed monitor context from the active event loop",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "fn runtime_window_attributes_with_monitor_context",
            "runtime_window_position",
            "centered_window_position",
        ],
        "window attributes should use primary monitor context for centered placement",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "fn for_event_loop(event_loop: &dyn ActiveEventLoop)",
            "event_loop.primary_monitor()",
            "event_loop.available_monitors().collect::<Vec<_>>()",
        ],
        "window monitor context should capture primary and available monitors from winit",
    );
    assert!(
        window_attributes_builder_source.contains("runtime_window_attributes_with_primary_monitor"),
        "window attribute tests should keep a primary-monitor helper for monitor-independent coverage"
    );
    for required in [
        "mod builder;",
        "mod fullscreen;",
        "mod monitor;",
        "mod position;",
        "mod video_mode;",
        "pub(super) use builder::runtime_window_attributes;",
    ] {
        assert!(
            window_attributes_root_source.contains(required),
            "window attributes root should preserve structural wiring `{required}`"
        );
    }
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "WindowPosition::Centered",
            "monitor.position()?",
            "monitor.current_video_mode()?.size()",
            "saturating_i64_to_i32",
        ],
        "centered placement should derive a safe physical position from the selected monitor",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "WindowMode::BorderlessFullscreen",
            "WindowMode::BorderlessFullscreenOn(monitor)",
            "borderless_fullscreen_for_selection",
            "WindowMode::Fullscreen",
            "WindowMode::FullscreenOn",
            "Fullscreen::Exclusive(monitor, video_mode)",
            "Fullscreen::Borderless(monitor)",
        ],
        "fullscreen policy should prefer exclusive current-video-mode fullscreen and fall back to borderless",
    );
    for required in [
        "WindowMonitorSelection::Current",
        "WindowMonitorSelection::Primary",
        "WindowMonitorSelection::Index(index)",
        "WindowVideoModeSelection::Current",
        "WindowVideoModeSelection::Specific(requested)",
        "video_mode_matches",
    ] {
        assert!(
            window_attributes_source.contains(required),
            "monitor-aware window creation should preserve `{required}`"
        );
    }
}
