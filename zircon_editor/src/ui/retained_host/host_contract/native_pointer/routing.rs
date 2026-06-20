mod chrome;
mod geometry;
mod panes;
mod workbench;

use crate::ui::retained_host::primitives::SharedString;

use super::super::data::FrameRect;
use super::super::surface_hit_test::{TemplateNodePointerHit, ViewportToolbarPointerHit};

pub(in crate::ui::retained_host::host_contract) use chrome::route_top_level_chrome;
pub(in crate::ui::retained_host::host_contract) use geometry::contains;
pub(in crate::ui::retained_host::host_contract) use panes::{
    route_pointer_move_to_pane, route_pointer_to_pane,
};
pub(in crate::ui::retained_host::host_contract) use workbench::route_pointer_to_workbench_window;

pub(in crate::ui::retained_host::host_contract) enum ChromePointerRoute {
    ActivityRail {
        side: SharedString,
        local_x: f32,
        local_y: f32,
    },
    HostPageTab {
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
    },
    DocumentTab {
        surface_key: SharedString,
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
        close: bool,
    },
    DrawerHeaderTab {
        surface_key: SharedString,
        index: usize,
        tab_x: f32,
        tab_width: f32,
        local_x: f32,
        local_y: f32,
    },
    FloatingWindowHeader {
        window_id: SharedString,
    },
    Resize,
}

pub(in crate::ui::retained_host::host_contract) struct PanePointerRoute {
    pub(in crate::ui::retained_host::host_contract) target: PanePointerTarget,
    pub(in crate::ui::retained_host::host_contract) frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract) local_x: f32,
    pub(in crate::ui::retained_host::host_contract) local_y: f32,
    pub(in crate::ui::retained_host::host_contract) width: f32,
    pub(in crate::ui::retained_host::host_contract) height: f32,
}

impl PanePointerRoute {
    pub(in crate::ui::retained_host::host_contract) fn new(
        target: PanePointerTarget,
        frame: &FrameRect,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            target,
            frame: frame.clone(),
            local_x: x - frame.x,
            local_y: y - frame.y,
            width: frame.width,
            height: frame.height,
        }
    }
}

pub(in crate::ui::retained_host::host_contract) enum PanePointerTarget {
    Hierarchy,
    Welcome,
    Console,
    Inspector,
    BrowserAssetDetails,
    AssetTree(SharedString),
    AssetContent(SharedString),
    AssetReference(SharedString, SharedString),
    TemplateNode(TemplateNodePointerHit),
    ViewportToolbar(ViewportToolbarPointerHit),
    Viewport(SharedString),
    UiAsset,
    Other,
}
