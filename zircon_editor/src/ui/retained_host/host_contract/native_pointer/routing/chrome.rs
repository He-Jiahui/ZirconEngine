use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{
    FrameRect, HostChromeControlFrameData, HostChromeTabData, HostWindowPresentationData,
};
use super::{
    geometry::{contains, translated},
    ChromePointerRoute,
};

pub(in crate::ui::retained_host::host_contract::native_pointer) fn route_top_level_chrome(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let scene = &presentation.host_scene_data;
    for splitter in [
        &scene.resize_layer.left_splitter_frame,
        &scene.resize_layer.right_splitter_frame,
        &scene.resize_layer.bottom_splitter_frame,
    ] {
        if contains(splitter, x, y) {
            return Some(ChromePointerRoute::Resize);
        }
    }

    if let Some(route) = route_document_tabs(
        "document",
        &translated(
            &scene.document_dock.header_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
        &scene.document_dock.tab_frames,
        x,
        y,
    ) {
        return Some(route);
    }
    if let Some(route) = route_activity_rail(
        &scene.left_dock.region_frame,
        true,
        scene.left_dock.rail_width_px,
        &scene.left_dock.rail_button_frames,
        x,
        y,
    ) {
        return Some(route);
    }
    if let Some(route) = route_activity_rail(
        &scene.right_dock.region_frame,
        false,
        scene.right_dock.rail_width_px,
        &scene.right_dock.rail_button_frames,
        x,
        y,
    ) {
        return Some(route);
    }
    if let Some(route) = route_drawer_header(
        "left",
        &scene.left_dock.region_frame,
        &scene.left_dock.header_frame,
        &scene.left_dock.tab_frames,
        x,
        y,
    ) {
        return Some(route);
    }
    if let Some(route) = route_drawer_header(
        "right",
        &scene.right_dock.region_frame,
        &scene.right_dock.header_frame,
        &scene.right_dock.tab_frames,
        x,
        y,
    ) {
        return Some(route);
    }
    if let Some(route) = route_drawer_header(
        "bottom",
        &scene.bottom_dock.region_frame,
        &scene.bottom_dock.header_frame,
        &scene.bottom_dock.tab_frames,
        x,
        y,
    ) {
        return Some(route);
    }
    if let Some(route) = route_host_page_tabs(&scene.page_chrome.tab_frames, x, y) {
        return Some(route);
    }

    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if contains(
            &translated(&window.header_frame, window.frame.x, window.frame.y),
            x,
            y,
        ) {
            if let Some(route) = route_document_tabs(
                window.window_id.as_str(),
                &translated(&window.header_frame, window.frame.x, window.frame.y),
                &window.tab_frames,
                x,
                y,
            ) {
                return Some(route);
            }
            return Some(ChromePointerRoute::FloatingWindowHeader {
                window_id: window.window_id.clone(),
            });
        }
    }

    None
}

fn route_host_page_tabs(
    tabs: &ModelRc<HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for row in 0..tabs.row_count() {
        let tab = tabs.row_data(row)?;
        if contains(&tab.frame, x, y) {
            return Some(ChromePointerRoute::HostPageTab {
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - tab.frame.x,
                local_y: y - tab.frame.y,
            });
        }
    }
    None
}

fn route_document_tabs(
    surface_key: &str,
    header_frame: &FrameRect,
    tabs: &ModelRc<HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for row in 0..tabs.row_count() {
        let tab = tabs.row_data(row)?;
        let close_frame = translated(&tab.close_frame, header_frame.x, header_frame.y);
        if contains(&close_frame, x, y) {
            return Some(ChromePointerRoute::DocumentTab {
                surface_key: surface_key.into(),
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - header_frame.x,
                local_y: y - header_frame.y,
                close: true,
            });
        }
        let tab_frame = translated(&tab.frame, header_frame.x, header_frame.y);
        if contains(&tab_frame, x, y) {
            return Some(ChromePointerRoute::DocumentTab {
                surface_key: surface_key.into(),
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - header_frame.x,
                local_y: y - header_frame.y,
                close: false,
            });
        }
    }
    None
}

fn route_drawer_header(
    surface_key: &str,
    region: &FrameRect,
    header: &FrameRect,
    tabs: &ModelRc<HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let header_origin = translated(header, region.x, region.y);
    for row in 0..tabs.row_count() {
        let tab = tabs.row_data(row)?;
        let tab_frame = translated(&tab.frame, header_origin.x, header_origin.y);
        if contains(&tab_frame, x, y) {
            return Some(ChromePointerRoute::DrawerHeaderTab {
                surface_key: surface_key.into(),
                index: row,
                tab_x: tab.frame.x,
                tab_width: tab.frame.width,
                local_x: x - header_origin.x,
                local_y: y - header_origin.y,
            });
        }
    }
    None
}

fn route_activity_rail(
    region: &FrameRect,
    rail_before_panel: bool,
    rail_width: f32,
    buttons: &ModelRc<HostChromeControlFrameData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    if !contains(region, x, y) || rail_width <= 0.0 {
        return None;
    }
    let rail_x = if rail_before_panel {
        region.x
    } else {
        region.x + (region.width - rail_width).max(0.0)
    };
    let rail = FrameRect {
        x: rail_x,
        y: region.y,
        width: rail_width.min(region.width.max(0.0)),
        height: region.height,
    };
    if !contains(&rail, x, y) {
        return None;
    }
    for row in 0..buttons.row_count() {
        let button = buttons.row_data(row)?;
        let button_frame = translated(&button.frame, rail.x, rail.y);
        if contains(&button_frame, x, y) {
            return Some(ChromePointerRoute::ActivityRail {
                side: if rail_before_panel { "left" } else { "right" }.into(),
                local_x: x - rail.x,
                local_y: y - rail.y,
            });
        }
    }
    None
}
