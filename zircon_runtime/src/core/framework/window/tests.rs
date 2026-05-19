use crate::core::math::UVec2;

use super::{
    PrimaryWindowHandle, WindowDescriptor, WindowMode, WindowPosition, WindowPresentMode,
    WindowResizeConstraints, WindowResolution, DEFAULT_WINDOW_TITLE,
};

#[test]
fn default_window_descriptor_matches_primary_runtime_window_policy() {
    let window = WindowDescriptor::default();

    assert_eq!(window.title, DEFAULT_WINDOW_TITLE);
    assert_eq!(window.primary_window, Some(PrimaryWindowHandle::default()));
    assert_eq!(window.present_mode, WindowPresentMode::Fifo);
    assert_eq!(window.mode, WindowMode::Windowed);
    assert_eq!(window.position, WindowPosition::Automatic);
    assert_eq!(window.resolution.physical_size(), UVec2::new(1280, 720));
    assert_eq!(window.resize_constraints.min_width, 180.0);
    assert_eq!(window.resize_constraints.min_height, 120.0);
    assert!(window.resizable);
    assert!(window.decorated);
    assert!(window.visible);
    assert!(window.focused);

    let diagnostics = window.format_diagnostics();
    for expected in [
        "window.primary_window=0",
        "window.title=Zircon Runtime",
        "window.present_mode=Fifo",
        "window.mode=Windowed",
        "window.position=Automatic",
        "window.physical_size=1280x720",
        "window.logical_size=1280x720",
        "window.scale_factor=1",
        "window.resizable=true",
        "window.decorated=true",
        "window.visible=true",
        "window.focused=true",
    ] {
        assert!(
            diagnostics.contains(expected),
            "window diagnostics should contain `{expected}`"
        );
    }
}

#[test]
fn window_resolution_tracks_physical_logical_and_scale_factor() {
    let mut resolution = WindowResolution::new(1920, 1080).with_scale_factor_override(2.0);

    assert_eq!(resolution.physical_size(), UVec2::new(1920, 1080));
    assert_eq!(resolution.logical_size(), [960.0, 540.0]);
    assert_eq!(resolution.scale_factor(), 2.0);
    assert_eq!(resolution.base_scale_factor(), 1.0);
    assert_eq!(resolution.scale_factor_override(), Some(2.0));

    resolution.set_logical_size(640.0, 360.0);

    assert_eq!(resolution.physical_size(), UVec2::new(1280, 720));
}

#[test]
fn window_resolution_clamps_zero_physical_size() {
    let resolution = WindowResolution::new(0, 0);

    assert_eq!(resolution.physical_size(), UVec2::new(1, 1));
}

#[test]
fn window_resize_constraints_clamp_invalid_bounds() {
    let constraints = WindowResizeConstraints {
        min_width: 0.0,
        min_height: -10.0,
        max_width: 0.5,
        max_height: 0.25,
    }
    .validated();

    assert_eq!(constraints.min_width, 1.0);
    assert_eq!(constraints.min_height, 1.0);
    assert_eq!(constraints.max_width, 1.0);
    assert_eq!(constraints.max_height, 1.0);
}

#[test]
fn window_resize_constraints_preserve_unbounded_defaults() {
    let constraints = WindowResizeConstraints::default().validated();

    assert_eq!(constraints.max_width, f32::INFINITY);
    assert_eq!(constraints.max_height, f32::INFINITY);
}

#[test]
fn window_resize_constraints_roundtrip_unbounded_json_values() {
    let json = serde_json::to_value(WindowResizeConstraints::default()).unwrap();
    let constraints: WindowResizeConstraints = serde_json::from_value(json).unwrap();

    assert_eq!(constraints.min_width, 180.0);
    assert_eq!(constraints.min_height, 120.0);
    assert_eq!(constraints.max_width, f32::INFINITY);
    assert_eq!(constraints.max_height, f32::INFINITY);
}

#[test]
fn window_descriptor_builder_preserves_host_neutral_settings() {
    let descriptor = WindowDescriptor::default()
        .with_primary_window(PrimaryWindowHandle::new(99))
        .with_title("Zircon Editor")
        .with_resolution(WindowResolution::new(1600, 900))
        .with_position(WindowPosition::At { x: 80, y: 120 })
        .with_present_mode(WindowPresentMode::AutoNoVsync)
        .with_resizable(false)
        .with_decorated(false)
        .with_visible(false)
        .with_focused(false);

    assert_eq!(descriptor.title, "Zircon Editor");
    assert_eq!(descriptor.primary_window.unwrap().raw(), 99);
    assert_eq!(descriptor.resolution.physical_size(), UVec2::new(1600, 900));
    assert_eq!(descriptor.position, WindowPosition::At { x: 80, y: 120 });
    assert_eq!(descriptor.present_mode, WindowPresentMode::AutoNoVsync);
    assert!(!descriptor.resizable);
    assert!(!descriptor.decorated);
    assert!(!descriptor.visible);
    assert!(!descriptor.focused);
}

#[test]
fn window_descriptor_diagnostics_can_record_absent_primary_window() {
    let descriptor = WindowDescriptor::default().without_primary_window();

    assert_eq!(descriptor.primary_window, None);
    assert!(descriptor
        .format_diagnostics()
        .contains("window.primary_window=none"));
}

#[test]
fn window_framework_root_stays_structural_after_folder_split() {
    let window_mod = include_str!("mod.rs");

    for required in [
        "mod constants;",
        "mod descriptor;",
        "mod mode;",
        "mod position;",
        "mod primary_window_handle;",
        "mod present_mode;",
        "mod resize_constraints;",
        "mod resolution;",
        "mod validation;",
        "WindowDescriptor",
        "PrimaryWindowHandle",
        "WindowMode",
        "WindowPosition",
        "WindowPresentMode",
        "WindowResizeConstraints",
        "WindowResolution",
    ] {
        assert!(
            window_mod.contains(required),
            "window framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub struct WindowDescriptor",
        "pub struct WindowResolution",
        "pub struct WindowResizeConstraints",
        "pub enum WindowPresentMode",
        "pub struct PrimaryWindowHandle",
        "impl WindowDescriptor",
        "impl WindowResolution",
    ] {
        assert!(
            !window_mod.contains(forbidden),
            "window framework root should not keep implementation detail `{forbidden}`"
        );
    }
}
