use super::super::super::data::{
    paint_pane_interaction_state, paint_text_input_focus, paint_viewport_images,
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
    let mut route = dock_damage_route(presentation, frame.paint_clip());
    if super::super::super::componentized_workbench_regions::owns_ordinary_panes(presentation) {
        route.left = false;
        route.right = false;
    }
    let visited = [route.left, route.document, route.right, route.bottom]
        .into_iter()
        .filter(|visited| *visited)
        .count();
    zircon_runtime::profile_counter!("editor", "ui.paint.dock_route_visit_count", visited);
    if visited == 0 {
        return;
    }
    let viewport_images = paint_viewport_images(presentation);
    let interaction = paint_pane_interaction_state(presentation);
    let text_input_focus = paint_text_input_focus(presentation);
    if route.left {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_left_dock");
        docks::draw_side_dock(
            frame,
            &scene.left_dock,
            &interaction,
            &viewport_images,
            Some(&text_input_focus),
        );
    }
    if route.document {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_document_dock");
        if scene.document_leaves.is_empty() {
            docks::draw_document_dock(
                frame,
                &scene.document_dock,
                &interaction,
                &viewport_images,
                Some(&text_input_focus),
            );
        } else {
            for leaf in &scene.document_leaves {
                if frame
                    .paint_clip()
                    .is_none_or(|clip| intersect(&leaf.region_frame, clip).is_some())
                {
                    docks::draw_document_dock(
                        frame,
                        leaf,
                        &interaction,
                        &viewport_images,
                        Some(&text_input_focus),
                    );
                }
            }
        }
    }
    if route.right {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_right_dock");
        docks::draw_side_dock(
            frame,
            &scene.right_dock,
            &interaction,
            &viewport_images,
            Some(&text_input_focus),
        );
    }
    if route.bottom {
        zircon_runtime::profile_scope!("editor", "host_painter", "painter_bottom_dock");
        docks::draw_bottom_dock(
            frame,
            &scene.bottom_dock,
            &interaction,
            &viewport_images,
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
        document: if scene.document_leaves.is_empty() {
            intersects(&scene.document_dock.region_frame)
        } else {
            scene
                .document_leaves
                .iter()
                .any(|leaf| intersects(&leaf.region_frame))
        },
        right: intersects(&scene.right_dock.region_frame),
        bottom: intersects(&scene.bottom_dock.region_frame),
    }
}

pub(super) fn draw_floating_layer(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    zircon_runtime::profile_scope!("editor", "host_painter", "painter_floating_layer");
    docks::draw_floating_layer(frame, presentation);
}

#[cfg(test)]
#[path = "tests/dock_layer.rs"]
mod tests;
