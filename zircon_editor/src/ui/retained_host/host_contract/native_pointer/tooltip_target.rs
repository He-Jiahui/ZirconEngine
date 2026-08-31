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
    let dock = &scene.document_dock;
    if surface_key == "document" || surface_key == dock.surface_key.as_str() {
        let tab = dock.tab_frames.get(index)?;
        return tooltip_target_from_tab(
            tab,
            dock.region_frame.x + dock.header_frame.x,
            dock.region_frame.y + dock.header_frame.y,
            "document",
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
mod tests {
    use std::rc::Rc;

    use crate::ui::retained_host::host_contract::data::{
        HostChromeTabData, HostWindowPresentationData, TabData,
    };
    use crate::ui::retained_host::primitives::{ModelRc, VecModel};

    use super::*;

    fn tab_frames(tab: HostChromeTabData) -> ModelRc<HostChromeTabData> {
        ModelRc::from(Rc::new(VecModel::from(vec![tab])))
    }

    fn tab(control_id: &str, title: &str, frame: FrameRect) -> HostChromeTabData {
        HostChromeTabData {
            control_id: control_id.into(),
            tab: TabData {
                id: control_id.into(),
                title: title.into(),
                ..TabData::default()
            },
            frame,
            ..HostChromeTabData::default()
        }
    }

    #[test]
    fn document_route_projects_the_exact_published_tab_frame() {
        let mut presentation = HostWindowPresentationData::default();
        let dock = &mut presentation.host_scene_data.document_dock;
        dock.region_frame = FrameRect {
            x: 100.0,
            y: 40.0,
            ..FrameRect::default()
        };
        dock.header_frame = FrameRect {
            x: 8.0,
            y: 3.0,
            ..FrameRect::default()
        };
        dock.tab_frames = tab_frames(tab(
            "DocumentSceneTab",
            "Scene",
            FrameRect {
                x: 24.0,
                y: 2.0,
                width: 96.0,
                height: 28.0,
            },
        ));
        let route = ChromePointerRoute::DocumentTab {
            surface_key: "document".into(),
            index: 0,
            tab_x: 24.0,
            tab_width: 96.0,
            local_x: 48.0,
            local_y: 12.0,
            close: false,
        };

        let target = tooltip_target_for_chrome_route(&presentation, &route).unwrap();

        assert_eq!(target.identity, "DocumentSceneTab");
        assert_eq!(target.label, "Scene");
        assert_eq!(target.frame.x, 132.0);
        assert_eq!(target.frame.y, 45.0);
        assert_eq!(target.frame.width, 96.0);
        assert_eq!(target.frame.height, 28.0);
    }

    #[test]
    fn side_drawer_route_includes_the_leading_activity_rail_once() {
        let mut presentation = HostWindowPresentationData::default();
        let dock = &mut presentation.host_scene_data.left_dock;
        dock.surface_key = "left".into();
        dock.region_frame = FrameRect {
            x: 10.0,
            y: 20.0,
            ..FrameRect::default()
        };
        dock.rail_before_panel = true;
        dock.rail_width_px = 32.0;
        dock.header_frame = FrameRect {
            x: 2.0,
            y: 4.0,
            ..FrameRect::default()
        };
        dock.tab_frames = tab_frames(tab(
            "LeftSceneTreeTab",
            "Scene Tree",
            FrameRect {
                x: 6.0,
                y: 1.0,
                width: 88.0,
                height: 26.0,
            },
        ));
        let route = ChromePointerRoute::DrawerHeaderTab {
            surface_key: "left".into(),
            index: 0,
            tab_x: 6.0,
            tab_width: 88.0,
            local_x: 30.0,
            local_y: 12.0,
        };

        let target = tooltip_target_for_chrome_route(&presentation, &route).unwrap();

        assert_eq!(target.identity, "LeftSceneTreeTab");
        assert_eq!(target.label, "Scene Tree");
        assert_eq!(target.frame.x, 50.0);
        assert_eq!(target.frame.y, 25.0);
        assert_eq!(target.frame.width, 88.0);
        assert_eq!(target.frame.height, 26.0);
    }
}
