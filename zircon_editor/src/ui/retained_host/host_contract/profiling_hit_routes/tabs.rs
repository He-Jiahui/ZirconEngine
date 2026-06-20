use crate::ui::retained_host::primitives::ModelRc;

use super::super::data::{FrameRect, HostChromeTabData, HostWindowSceneData};
use super::geometry::{contains, translated};

pub(in crate::ui::retained_host::host_contract) fn document_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
    tab_route_hit(
        &scene.document_dock.tab_frames,
        id,
        x,
        y,
        Some(&translated(
            &scene.document_dock.header_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        )),
    )
}

pub(in crate::ui::retained_host::host_contract) fn drawer_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
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

pub(in crate::ui::retained_host::host_contract) fn floating_tab_route_hit(
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

pub(in crate::ui::retained_host::host_contract) fn host_page_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
    tab_route_hit(&scene.page_chrome.tab_frames, id, x, y, None)
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
