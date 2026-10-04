use super::*;
use winit::dpi::{PhysicalPosition, Position};
use winit::monitor::Fullscreen;
use zircon_runtime::core::framework::window::{
    WindowMode, WindowMonitorSelection, WindowPosition, WindowResizeConstraints, WindowResolution,
    WindowVideoMode, WindowVideoModeSelection, DEFAULT_WINDOW_TITLE,
};

#[test]
fn default_window_descriptor_builds_runtime_window_attributes() {
    let attributes =
        runtime_window_attributes_with_primary_monitor(&WindowDescriptor::default(), None);

    assert_eq!(attributes.title, DEFAULT_WINDOW_TITLE);
    assert_eq!(
        attributes.surface_size,
        Some(Size::Physical(PhysicalSize::new(1280, 720)))
    );
    assert_eq!(
        attributes.min_surface_size,
        Some(Size::Logical(LogicalSize::new(180.0, 120.0)))
    );
    assert_eq!(attributes.max_surface_size, None);
    assert_eq!(attributes.position, None);
    assert_eq!(attributes.fullscreen, None);
    assert!(attributes.resizable);
    assert!(attributes.decorations);
    assert!(attributes.visible);
    assert!(attributes.active);
}

#[test]
fn custom_window_descriptor_builds_runtime_window_attributes() {
    let descriptor = WindowDescriptor::default()
        .with_title("Zircon Host")
        .with_resolution(WindowResolution::new(1600, 900))
        .with_resize_constraints(WindowResizeConstraints {
            min_width: 320.0,
            min_height: 240.0,
            max_width: 1920.0,
            max_height: 1080.0,
        })
        .with_position(WindowPosition::At { x: 64, y: 96 })
        .with_mode(WindowMode::BorderlessFullscreen)
        .with_resizable(false)
        .with_decorated(false)
        .with_visible(false)
        .with_focused(false);

    let attributes = runtime_window_attributes_with_primary_monitor(&descriptor, None);

    assert_eq!(attributes.title, "Zircon Host");
    assert_eq!(
        attributes.surface_size,
        Some(Size::Physical(PhysicalSize::new(1600, 900)))
    );
    assert_eq!(
        attributes.min_surface_size,
        Some(Size::Logical(LogicalSize::new(320.0, 240.0)))
    );
    assert_eq!(
        attributes.max_surface_size,
        Some(Size::Logical(LogicalSize::new(1920.0, 1080.0)))
    );
    assert_eq!(
        attributes.position,
        Some(Position::Physical(PhysicalPosition::new(64, 96)))
    );
    assert_eq!(attributes.fullscreen, Some(Fullscreen::Borderless(None)));
    assert!(!attributes.resizable);
    assert!(!attributes.decorations);
    assert!(!attributes.visible);
    assert!(!attributes.active);
}

#[test]
fn centered_window_descriptor_waits_for_primary_monitor_context() {
    let descriptor = WindowDescriptor::default().with_position(WindowPosition::Centered);

    let attributes = runtime_window_attributes_with_primary_monitor(&descriptor, None);

    assert_eq!(attributes.position, None);
}

#[test]
fn centered_on_current_window_descriptor_falls_back_to_automatic_during_creation() {
    let descriptor = WindowDescriptor::default()
        .with_position(WindowPosition::CenteredOn(WindowMonitorSelection::Current));

    let attributes = runtime_window_attributes_with_primary_monitor(&descriptor, None);

    assert_eq!(attributes.position, None);
}

#[test]
fn fullscreen_descriptor_falls_back_to_borderless_without_primary_monitor() {
    let descriptor = WindowDescriptor::default().with_mode(WindowMode::Fullscreen);

    let attributes = runtime_window_attributes_with_primary_monitor(&descriptor, None);

    assert_eq!(attributes.fullscreen, Some(Fullscreen::Borderless(None)));
}

#[test]
fn current_monitor_borderless_fullscreen_keeps_winit_current_monitor_policy() {
    let descriptor = WindowDescriptor::default().with_mode(WindowMode::BorderlessFullscreenOn(
        WindowMonitorSelection::Current,
    ));

    let attributes = runtime_window_attributes_with_primary_monitor(&descriptor, None);

    assert_eq!(attributes.fullscreen, Some(Fullscreen::Borderless(None)));
}

#[test]
fn fullscreen_with_specific_video_mode_falls_back_to_borderless_without_monitor() {
    let descriptor = WindowDescriptor::default().with_mode(WindowMode::FullscreenOn {
        monitor: WindowMonitorSelection::Index(1),
        video_mode: WindowVideoModeSelection::Specific(
            WindowVideoMode::new(1920, 1080)
                .with_refresh_rate_millihertz(60_000)
                .with_bit_depth(32),
        ),
    });

    let attributes = runtime_window_attributes_with_primary_monitor(&descriptor, None);

    assert_eq!(attributes.fullscreen, Some(Fullscreen::Borderless(None)));
}
