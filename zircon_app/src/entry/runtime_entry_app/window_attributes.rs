use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Position, Size};
use winit::monitor::Fullscreen;
use winit::window::WindowAttributes;
use zircon_runtime::core::framework::window::{
    WindowDescriptor, WindowMode, WindowPosition, WindowResizeConstraints,
};

pub(super) fn runtime_window_attributes(descriptor: &WindowDescriptor) -> WindowAttributes {
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

    match descriptor.position {
        WindowPosition::Automatic | WindowPosition::Centered => {}
        WindowPosition::At { x, y } => {
            attributes = attributes.with_position(Position::Physical(PhysicalPosition::new(x, y)));
        }
    }

    match descriptor.mode {
        WindowMode::Windowed => {}
        WindowMode::BorderlessFullscreen | WindowMode::Fullscreen => {
            attributes = attributes.with_fullscreen(Some(Fullscreen::Borderless(None)));
        }
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
    use zircon_runtime::core::framework::window::{
        WindowResizeConstraints, WindowResolution, DEFAULT_WINDOW_TITLE,
    };

    #[test]
    fn default_window_descriptor_builds_runtime_window_attributes() {
        let attributes = runtime_window_attributes(&WindowDescriptor::default());

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

        let attributes = runtime_window_attributes(&descriptor);

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
}
