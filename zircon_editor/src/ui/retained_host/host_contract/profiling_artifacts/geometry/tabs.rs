use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{
    FloatingWindowData, FrameRect, HostBottomDockSurfaceData, HostChromeTabData,
    HostDocumentDockSurfaceData, HostSideDockSurfaceData,
};
use super::super::UiProfileTabFrame;
use super::frame_math::{is_visible_frame, translated};

pub(in crate::ui::retained_host::host_contract) fn collect_document_tabs(
    dock: &HostDocumentDockSurfaceData,
) -> Vec<UiProfileTabFrame> {
    let header = translated(&dock.header_frame, dock.region_frame.x, dock.region_frame.y);
    collect_tabs(
        "document_tab",
        dock.surface_key.as_str(),
        &dock.tab_frames,
        &header,
    )
}

pub(in crate::ui::retained_host::host_contract) fn collect_side_dock_tabs(
    surface: &str,
    dock: &HostSideDockSurfaceData,
    out: &mut Vec<UiProfileTabFrame>,
) {
    let header = translated(&dock.header_frame, dock.region_frame.x, dock.region_frame.y);
    out.extend(collect_tabs(
        "drawer_tab",
        surface,
        &dock.tab_frames,
        &header,
    ));
}

pub(in crate::ui::retained_host::host_contract) fn collect_bottom_dock_tabs(
    surface: &str,
    dock: &HostBottomDockSurfaceData,
    out: &mut Vec<UiProfileTabFrame>,
) {
    let header = translated(&dock.header_frame, dock.region_frame.x, dock.region_frame.y);
    out.extend(collect_tabs(
        "drawer_tab",
        surface,
        &dock.tab_frames,
        &header,
    ));
}

pub(in crate::ui::retained_host::host_contract) fn collect_floating_window_tabs(
    window: &FloatingWindowData,
    out: &mut Vec<UiProfileTabFrame>,
) {
    let header = translated(&window.header_frame, window.frame.x, window.frame.y);
    out.extend(collect_tabs(
        "floating_tab",
        window.window_id.as_str(),
        &window.tab_frames,
        &header,
    ));
}

pub(in crate::ui::retained_host::host_contract) fn collect_host_page_tabs(
    tabs: &ModelRc<HostChromeTabData>,
) -> Vec<UiProfileTabFrame> {
    collect_tabs("host_page_tab", "host_page", tabs, &FrameRect::default())
}

fn collect_tabs(
    kind: &str,
    surface: &str,
    tabs: &ModelRc<HostChromeTabData>,
    origin: &FrameRect,
) -> Vec<UiProfileTabFrame> {
    let mut out = Vec::new();
    for row in 0..tabs.row_count() {
        let Some(tab) = tabs.row_data(row) else {
            continue;
        };
        let frame = translated(&tab.frame, origin.x, origin.y);
        if !is_visible_frame(&frame) {
            continue;
        }
        out.push(UiProfileTabFrame {
            id: tab.control_id.to_string(),
            title: tab.tab.title.to_string(),
            kind: kind.to_string(),
            surface: surface.to_string(),
            frame: frame.into(),
            close_frame: translated(&tab.close_frame, origin.x, origin.y).into(),
            active: tab.tab.active,
        });
    }
    out
}
