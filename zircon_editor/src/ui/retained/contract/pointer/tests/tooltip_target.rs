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
