use super::*;
use crate::scene::viewport::{RenderViewportHandle, RenderViewportProduct};
use crate::ui::retained_host::host_contract::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::PaneSurfaceHostContext;

#[test]
fn native_viewport_products_publish_to_each_child_without_changing_input_identity() {
    let first = native_child("native:first");
    let second = native_child("native:second");
    first
        .global::<PaneSurfaceHostContext>()
        .set_scene_viewport_surface_key("native:first");
    second
        .global::<PaneSurfaceHostContext>()
        .set_scene_viewport_surface_key("native:second");

    assert!(publish_viewport_product(
        &second,
        "native:second",
        RenderViewportProduct::new(RenderViewportHandle::new(22), 800, 600, 7),
    ));
    assert!(publish_viewport_product(
        &first,
        "native:first",
        RenderViewportProduct::new(RenderViewportHandle::new(11), 640, 360, 7),
    ));

    for (ui, surface_key, resource_key) in [
        (&first, "native:first", "viewport:11:7"),
        (&second, "native:second", "viewport:22:7"),
    ] {
        let context = ui.global::<PaneSurfaceHostContext>();
        assert_eq!(
            context.scene_viewport_surface_key().as_deref(),
            Some(surface_key)
        );
        let generation = ui.get_host_presentation_generation();
        assert_eq!(
            generation
                .viewport_images()
                .for_surface(surface_key, "Scene")
                .expect("native child viewport product")
                .resource_key,
            resource_key
        );
        assert!(ui.take_external_redraw_for_test().request_redraw());
    }
}

#[test]
fn secondary_dock_product_damages_its_own_region() {
    let ui = UiHostWindow::new().expect("host window");
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.right_dock.surface_key = "dock:right".into();
    presentation.host_scene_data.right_dock.region_frame = FrameRect {
        x: 900.0,
        y: 80.0,
        width: 300.0,
        height: 640.0,
    };
    ui.set_host_presentation(presentation);
    let _ = ui.take_external_redraw_for_test();

    assert!(publish_viewport_product(
        &ui,
        "dock:right",
        RenderViewportProduct::new(RenderViewportHandle::new(31), 300, 640, 9),
    ));

    assert_eq!(
        ui.take_external_redraw_for_test().damage_region(),
        Some(&FrameRect {
            x: 900.0,
            y: 80.0,
            width: 300.0,
            height: 640.0,
        })
    );
}

fn native_child(surface_key: &str) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("native child window");
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_shell.native_floating_window_mode = true;
    presentation.host_shell.native_floating_window_id = surface_key.into();
    ui.set_host_presentation(presentation);
    let _ = ui.take_external_redraw_for_test();
    ui
}
