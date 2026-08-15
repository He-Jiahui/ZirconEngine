use super::super::super::data::{
    paint_pane_interaction_state, paint_text_input_focus, paint_viewport_image,
    HostWindowPresentationData,
};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::intersect;
use super::super::docks;

#[derive(Clone, Copy)]
struct DockDamageRoute {
    left: bool,
    document: bool,
    right: bool,
    bottom: bool,
}

pub(super) fn draw_dock_layers(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let scene = &presentation.host_scene_data;
    let route = dock_damage_route(presentation, frame.paint_clip());
    let visited = [route.left, route.document, route.right, route.bottom]
        .into_iter()
        .filter(|visited| *visited)
        .count();
    zircon_runtime::profile_counter!("editor", "ui.paint.dock_route_visit_count", visited);
    if visited == 0 {
        return;
    }
    let viewport_image = paint_viewport_image(presentation);
    let interaction = paint_pane_interaction_state(presentation);
    let text_input_focus = paint_text_input_focus(presentation);
    if route.left {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_left_dock");
        docks::draw_side_dock(
            frame,
            &scene.left_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
    if route.document {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock");
        docks::draw_document_dock(
            frame,
            &scene.document_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
    if route.right {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_right_dock");
        docks::draw_side_dock(
            frame,
            &scene.right_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
    if route.bottom {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock");
        docks::draw_bottom_dock(
            frame,
            &scene.bottom_dock,
            &interaction,
            viewport_image.as_deref(),
            Some(&text_input_focus),
        );
    }
}

fn dock_damage_route(
    presentation: &HostWindowPresentationData,
    damage: Option<&super::super::super::data::FrameRect>,
) -> DockDamageRoute {
    let scene = &presentation.host_scene_data;
    let intersects = |region: &super::super::super::data::FrameRect| {
        damage.map_or(true, |damage| intersect(region, damage).is_some())
    };
    DockDamageRoute {
        left: intersects(&scene.left_dock.region_frame),
        document: intersects(&scene.document_dock.region_frame),
        right: intersects(&scene.right_dock.region_frame),
        bottom: intersects(&scene.bottom_dock.region_frame),
    }
}

pub(super) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    zircon_runtime::profile_scope!("editor", "host_painter", "painter_floating_layer");
    let interaction = paint_pane_interaction_state(presentation);
    let viewport_image = paint_viewport_image(presentation);
    let text_input_focus = paint_text_input_focus(presentation);
    docks::draw_floating_layer(
        frame,
        presentation,
        &interaction,
        viewport_image.as_deref(),
        Some(&text_input_focus),
    );
}

#[cfg(test)]
mod tests {
    use super::dock_damage_route;
    use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

    #[test]
    fn left_dock_damage_does_not_visit_unrelated_docks() {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_scene_data.left_dock.region_frame = rect(0.0, 50.0, 240.0, 500.0);
        presentation.host_scene_data.document_dock.region_frame = rect(240.0, 50.0, 800.0, 500.0);
        presentation.host_scene_data.right_dock.region_frame = rect(1040.0, 50.0, 240.0, 500.0);
        presentation.host_scene_data.bottom_dock.region_frame = rect(0.0, 550.0, 1280.0, 170.0);

        let route = dock_damage_route(&presentation, Some(&rect(12.0, 72.0, 80.0, 32.0)));

        assert!(route.left);
        assert!(!route.document);
        assert!(!route.right);
        assert!(!route.bottom);
    }

    fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
