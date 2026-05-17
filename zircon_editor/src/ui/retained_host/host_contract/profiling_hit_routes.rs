use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::layout::UiPoint;

use super::data::{
    FrameRect, HostChromeTabData, HostWindowPresentationData, HostWindowSceneData, PaneData,
};
use super::surface_hit_test;
use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn route_contains_profile_frame(
    presentation: &HostWindowPresentationData,
    kind: &str,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    let scene = &presentation.host_scene_data;
    match kind {
        "resize_splitter" => match surface {
            "left" => contains(&scene.resize_layer.left_splitter_frame, x, y),
            "right" => contains(&scene.resize_layer.right_splitter_frame, x, y),
            "bottom" => contains(&scene.resize_layer.bottom_splitter_frame, x, y),
            _ => false,
        },
        "document_tab" => tab_route_hit(
            &scene.document_dock.tab_frames,
            id,
            x,
            y,
            Some(&translated(
                &scene.document_dock.header_frame,
                scene.document_dock.region_frame.x,
                scene.document_dock.region_frame.y,
            )),
        ),
        "drawer_tab" => {
            (surface == "left"
                && tab_route_hit(
                    &scene.left_dock.tab_frames,
                    id,
                    x,
                    y,
                    Some(&translated(
                        &scene.left_dock.header_frame,
                        scene.left_dock.region_frame.x,
                        scene.left_dock.region_frame.y,
                    )),
                ))
                || (surface == "right"
                    && tab_route_hit(
                        &scene.right_dock.tab_frames,
                        id,
                        x,
                        y,
                        Some(&translated(
                            &scene.right_dock.header_frame,
                            scene.right_dock.region_frame.x,
                            scene.right_dock.region_frame.y,
                        )),
                    ))
                || (surface == "bottom"
                    && tab_route_hit(
                        &scene.bottom_dock.tab_frames,
                        id,
                        x,
                        y,
                        Some(&translated(
                            &scene.bottom_dock.header_frame,
                            scene.bottom_dock.region_frame.x,
                            scene.bottom_dock.region_frame.y,
                        )),
                    ))
        }
        "floating_tab" => floating_tab_route_hit(scene, id, surface, x, y),
        "host_page_tab" => tab_route_hit(&scene.page_chrome.tab_frames, id, x, y, None),
        "activity_rail_button" => activity_rail_route_hit(scene, id, surface, x, y),
        "viewport_toolbar_control" => viewport_toolbar_route_hit(scene, id, x, y),
        "template_control" => template_route_hit(scene, id, x, y),
        _ => false,
    }
}

fn tab_route_hit(
    tabs: &ModelRc<HostChromeTabData>,
    id: &str,
    x: f32,
    y: f32,
    origin: Option<&FrameRect>,
) -> bool {
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let frame = match origin {
            Some(origin) => translated(&tab.frame, origin.x, origin.y),
            None => tab.frame.clone(),
        };
        if tab.control_id.as_str() == id && contains(&frame, x, y) {
            return true;
        }
    }
    false
}

fn floating_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if window.window_id.as_str() != surface {
            continue;
        }
        let header = translated(&window.header_frame, window.frame.x, window.frame.y);
        for tab_row in 0..window.tab_frames.row_count() {
            let Some(tab) = window.tab_frames.row_data(tab_row) else {
                continue;
            };
            if tab.control_id.as_str() == id
                && contains(&translated(&tab.frame, header.x, header.y), x, y)
            {
                return true;
            }
        }
    }
    false
}

fn activity_rail_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    let dock = match surface {
        "left" => &scene.left_dock,
        "right" => &scene.right_dock,
        _ => return false,
    };
    if dock.rail_width_px <= 0.0 || !contains(&dock.region_frame, x, y) {
        return false;
    }
    let rail_x = if dock.rail_before_panel {
        dock.region_frame.x
    } else {
        dock.region_frame.x + (dock.region_frame.width - dock.rail_width_px).max(0.0)
    };
    let rail = FrameRect {
        x: rail_x,
        y: dock.region_frame.y,
        width: dock.rail_width_px.min(dock.region_frame.width.max(0.0)),
        height: dock.region_frame.height,
    };
    for row in 0..dock.rail_button_frames.row_count() {
        let Some(button) = dock.rail_button_frames.row_data(row) else {
            continue;
        };
        let expected_id = format!("activity_rail.{surface}.{}", button.control_id);
        if expected_id == id && contains(&translated(&button.frame, rail.x, rail.y), x, y) {
            return true;
        }
    }
    false
}

fn viewport_toolbar_route_hit(scene: &HostWindowSceneData, id: &str, x: f32, y: f32) -> bool {
    pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.document_dock.surface_key.as_str(),
        &scene.document_dock.pane,
        &translated(
            &scene.document_dock.content_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.left_dock.surface_key.as_str(),
        &scene.left_dock.pane,
        &side_dock_content_frame(&scene.left_dock),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.right_dock.surface_key.as_str(),
        &scene.right_dock.pane,
        &side_dock_content_frame(&scene.right_dock),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.bottom_dock.surface_key.as_str(),
        &scene.bottom_dock.pane,
        &translated(
            &scene.bottom_dock.content_frame,
            scene.bottom_dock.region_frame.x,
            scene.bottom_dock.region_frame.y,
        ),
    ) || floating_windows_hit_toolbar(scene, id, x, y)
}

fn pane_route_hits_viewport_toolbar(
    id: &str,
    x: f32,
    y: f32,
    surface_key: &str,
    pane: &PaneData,
    content: &FrameRect,
) -> bool {
    let expected_prefix = format!("viewport_toolbar_control.{surface_key}.");
    if !id.starts_with(&expected_prefix)
        || !matches!(pane.kind.as_str(), "Scene" | "Game")
        || !pane.show_toolbar
        || !contains(content, x, y)
    {
        return false;
    }
    let toolbar_height = 28.0_f32.min(content.height);
    let toolbar = FrameRect {
        x: content.x,
        y: content.y,
        width: content.width,
        height: toolbar_height,
    };
    surface_hit_test::hit_test_viewport_toolbar(surface_key, &pane.viewport, &toolbar, x, y)
        .is_some_and(|hit| {
            format!("viewport_toolbar_control.{surface_key}.{}", hit.control_id) == id
        })
}

fn floating_windows_hit_toolbar(scene: &HostWindowSceneData, id: &str, x: f32, y: f32) -> bool {
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if pane_route_hits_viewport_toolbar(
            id,
            x,
            y,
            window.window_id.as_str(),
            &window.active_pane,
            &floating_window_content_frame(&window.frame, &window.header_frame),
        ) {
            return true;
        }
    }
    false
}

fn template_route_hit(scene: &HostWindowSceneData, id: &str, x: f32, y: f32) -> bool {
    pane_route_hits_template(
        id,
        x,
        y,
        "document",
        &scene.document_dock.pane,
        &translated(
            &scene.document_dock.content_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
    ) || pane_route_hits_template(
        id,
        x,
        y,
        "left",
        &scene.left_dock.pane,
        &side_dock_content_frame(&scene.left_dock),
    ) || pane_route_hits_template(
        id,
        x,
        y,
        "right",
        &scene.right_dock.pane,
        &side_dock_content_frame(&scene.right_dock),
    ) || pane_route_hits_template(
        id,
        x,
        y,
        "bottom",
        &scene.bottom_dock.pane,
        &translated(
            &scene.bottom_dock.content_frame,
            scene.bottom_dock.region_frame.x,
            scene.bottom_dock.region_frame.y,
        ),
    ) || floating_windows_hit_template(scene, id, x, y)
}

fn pane_route_hits_template(
    id: &str,
    x: f32,
    y: f32,
    surface: &str,
    pane: &PaneData,
    content: &FrameRect,
) -> bool {
    let expected_prefix = format!("template.{surface}.");
    if !id.starts_with(&expected_prefix) || !contains(content, x, y) {
        return false;
    }
    let mut body = content.clone();
    if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar_height = 28.0_f32.min(content.height);
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    let Some(surface_frame) = pane.body_surface_frame.as_ref() else {
        return false;
    };
    let point = UiPoint::new(x - body.x, y - body.y);
    let Some(node_id) = hit_test_surface_frame(surface_frame, point).top_hit else {
        return false;
    };
    let Some(node) = surface_frame.arranged_tree.get(node_id) else {
        return false;
    };
    let Some(control_id) = node.control_id.as_deref() else {
        return false;
    };
    format!("template.{surface}.{control_id}") == id
}

fn floating_windows_hit_template(scene: &HostWindowSceneData, id: &str, x: f32, y: f32) -> bool {
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if pane_route_hits_template(
            id,
            x,
            y,
            window.window_id.as_str(),
            &window.active_pane,
            &floating_window_content_frame(&window.frame, &window.header_frame),
        ) {
            return true;
        }
    }
    false
}

fn side_dock_content_frame(dock: &super::data::HostSideDockSurfaceData) -> FrameRect {
    let panel_x = if dock.rail_before_panel {
        dock.region_frame.x + dock.rail_width_px
    } else {
        dock.region_frame.x
    };
    translated(&dock.content_frame, panel_x, dock.region_frame.y)
}

fn floating_window_content_frame(frame: &FrameRect, header: &FrameRect) -> FrameRect {
    FrameRect {
        x: frame.x + 1.0,
        y: frame.y + header.height.max(0.0) + 1.0,
        width: (frame.width - 2.0).max(0.0),
        height: (frame.height - header.height.max(0.0) - 2.0).max(0.0),
    }
}

fn translated(frame: &FrameRect, origin_x: f32, origin_y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + origin_x,
        y: frame.y + origin_y,
        width: frame.width,
        height: frame.height,
    }
}

fn contains(frame: &FrameRect, x: f32, y: f32) -> bool {
    frame.width > 0.0
        && frame.height > 0.0
        && x >= frame.x
        && y >= frame.y
        && x < frame.x + frame.width
        && y < frame.y + frame.height
}
