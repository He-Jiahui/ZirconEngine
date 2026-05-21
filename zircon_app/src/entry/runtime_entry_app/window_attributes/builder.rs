use winit::dpi::{LogicalSize, PhysicalSize, Size};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowResizeConstraints};

use super::fullscreen::runtime_window_fullscreen;
use super::monitor::WindowMonitorContext;
use super::position::runtime_window_position;

pub(in crate::entry::runtime_entry_app) fn runtime_window_attributes(
    descriptor: &WindowDescriptor,
    event_loop: &dyn ActiveEventLoop,
) -> WindowAttributes {
    let monitor_context = WindowMonitorContext::for_event_loop(event_loop);
    runtime_window_attributes_with_monitor_context(descriptor, &monitor_context)
}

#[cfg(test)]
fn runtime_window_attributes_with_primary_monitor(
    descriptor: &WindowDescriptor,
    primary_monitor: Option<winit::monitor::MonitorHandle>,
) -> WindowAttributes {
    let monitor_context = WindowMonitorContext::primary_only(primary_monitor);
    runtime_window_attributes_with_monitor_context(descriptor, &monitor_context)
}

fn runtime_window_attributes_with_monitor_context(
    descriptor: &WindowDescriptor,
    monitor_context: &WindowMonitorContext,
) -> WindowAttributes {
    let physical_size = descriptor.resolution.physical_size();
    let constraints = descriptor.resize_constraints.validated();
    let mut attributes = WindowAttributes::default()
        .with_title(descriptor.title.clone())
        .with_surface_size(Size::Physical(PhysicalSize::new(
            physical_size.x,
            physical_size.y,
        )))
        .with_min_surface_size(Size::Logical(LogicalSize::new(
            constraints.min_width as f64,
            constraints.min_height as f64,
        )))
        .with_resizable(descriptor.resizable)
        .with_decorations(descriptor.decorated)
        .with_visible(descriptor.visible)
        .with_active(descriptor.focused);

    if let Some(max_size) = finite_max_surface_size(constraints) {
        attributes = attributes.with_max_surface_size(max_size);
    }

    if let Some(position) =
        runtime_window_position(descriptor.position, &descriptor.resolution, monitor_context)
    {
        attributes = attributes.with_position(winit::dpi::Position::Physical(position));
    }

    if let Some(fullscreen) = runtime_window_fullscreen(descriptor.mode, monitor_context) {
        attributes = attributes.with_fullscreen(Some(fullscreen));
    }

    attributes
}

fn finite_max_surface_size(constraints: WindowResizeConstraints) -> Option<Size> {
    if constraints.max_width.is_finite() && constraints.max_height.is_finite() {
        Some(Size::Logical(LogicalSize::new(
            constraints.max_width as f64,
            constraints.max_height as f64,
        )))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::{PhysicalPosition, Position};
    use winit::monitor::Fullscreen;
    use zircon_runtime::core::framework::window::{
        WindowMode, WindowMonitorSelection, WindowPosition, WindowResizeConstraints,
        WindowResolution, WindowVideoMode, WindowVideoModeSelection, DEFAULT_WINDOW_TITLE,
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
}
