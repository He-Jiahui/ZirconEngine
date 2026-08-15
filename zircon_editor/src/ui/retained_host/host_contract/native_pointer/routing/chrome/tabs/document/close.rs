use crate::ui::retained_host::host_contract::data::{FrameRect, HostChromeTabData};

use super::super::super::super::{
    geometry::{contains, translated},
    ChromePointerRoute,
};

pub(super) fn route_document_tab_close(
    surface_key: &str,
    header_frame: &FrameRect,
    tab: &HostChromeTabData,
    row: usize,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let close_frame = translated(&tab.close_frame, header_frame.x, header_frame.y);
    if !contains(&close_frame, x, y) {
        return None;
    }
    Some(ChromePointerRoute::DocumentTab {
        surface_key: surface_key.into(),
        index: row,
        tab_x: tab.frame.x,
        tab_width: tab.frame.width,
        local_x: x - header_frame.x,
        local_y: y - header_frame.y,
        close: true,
    })
}
