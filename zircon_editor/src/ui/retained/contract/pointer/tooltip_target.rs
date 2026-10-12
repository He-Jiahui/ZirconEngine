use zircon_runtime_interface::ui::event_ui::UiNodeId;

use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostChromeTabData, HostSideDockSurfaceData, HostWindowPresentationData,
};
use crate::ui::retained_host::primitives::SharedString;

use super::routing::ChromePointerRoute;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostChromeTooltipTarget {
    pub(crate) identity: SharedString,
    pub(crate) label: SharedString,
    pub(crate) frame: FrameRect,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum WorkbenchTooltipPointerTarget {
    SurfaceNode(UiNodeId),
    HostChrome(HostChromeTooltipTarget),
}

pub(in crate::ui::retained_host::host_contract) fn tooltip_target_for_chrome_route(
    presentation: &HostWindowPresentationData,
    route: &ChromePointerRoute,
) -> Option<HostChromeTooltipTarget> {
    match route {
        ChromePointerRoute::DocumentTab {
            surface_key,
            index,
            close: false,
            ..
        } => document_tab_target(presentation, surface_key.as_str(), *index),
        ChromePointerRoute::DrawerHeaderTab {
            surface_key, index, ..
        } => drawer_tab_target(presentation, surface_key.as_str(), *index),
        _ => None,
    }
}

fn document_tab_target(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
    index: usize,
) -> Option<HostChromeTooltipTarget> {
    let scene = &presentation.host_scene_data;
    if let Some(dock) = scene
        .document_surfaces()
        .iter()
        .find(|dock| surface_key == dock.surface_key.as_str())
    {
        let tab = dock.tab_frames.get(index)?;
        return tooltip_target_from_tab(
            tab,
            dock.region_frame.x + dock.header_frame.x,
            dock.region_frame.y + dock.header_frame.y,
            surface_key,
            index,
        );
    }

    let window = scene
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == surface_key)?;
    let tab = window.tab_frames.get(index)?;
    tooltip_target_from_tab(
        tab,
        window.frame.x + window.header_frame.x,
        window.frame.y + window.header_frame.y,
        surface_key,
        index,
    )
}

fn drawer_tab_target(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
    index: usize,
) -> Option<HostChromeTooltipTarget> {
    let scene = &presentation.host_scene_data;
    if surface_key == "left" || surface_key == scene.left_dock.surface_key.as_str() {
        return side_drawer_tab_target(&scene.left_dock, "left", index);
    }
    if surface_key == "right" || surface_key == scene.right_dock.surface_key.as_str() {
        return side_drawer_tab_target(&scene.right_dock, "right", index);
    }
    if surface_key == "bottom" || surface_key == scene.bottom_dock.surface_key.as_str() {
        let dock = &scene.bottom_dock;
        let tab = dock.tab_frames.get(index)?;
        return tooltip_target_from_tab(
            tab,
            dock.region_frame.x + dock.header_frame.x,
            dock.region_frame.y + dock.header_frame.y,
            "bottom",
            index,
        );
    }
    None
}

fn side_drawer_tab_target(
    dock: &HostSideDockSurfaceData,
    fallback_surface_key: &str,
    index: usize,
) -> Option<HostChromeTooltipTarget> {
    let panel_x = if dock.rail_before_panel {
        dock.region_frame.x + dock.rail_width_px
    } else {
        dock.region_frame.x
    };
    let tab = dock.tab_frames.get(index)?;
    tooltip_target_from_tab(
        tab,
        panel_x + dock.header_frame.x,
        dock.region_frame.y + dock.header_frame.y,
        fallback_surface_key,
        index,
    )
}

fn tooltip_target_from_tab(
    tab: &HostChromeTabData,
    origin_x: f32,
    origin_y: f32,
    fallback_surface_key: &str,
    index: usize,
) -> Option<HostChromeTooltipTarget> {
    if tab.tab.title.is_empty() {
        return None;
    }
    let identity = if !tab.control_id.is_empty() {
        tab.control_id.clone()
    } else if !tab.tab.id.is_empty() {
        tab.tab.id.clone()
    } else {
        format!("{fallback_surface_key}:{index}")
    };
    Some(HostChromeTooltipTarget {
        identity,
        label: tab.tab.title.clone(),
        frame: FrameRect {
            x: origin_x + tab.frame.x,
            y: origin_y + tab.frame.y,
            width: tab.frame.width,
            height: tab.frame.height,
        },
    })
}

#[cfg(test)]
#[path = "tests/tooltip_target.rs"]
mod tests;
