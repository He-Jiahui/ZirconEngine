use crate::ui::retained_host::host_contract::data::{FrameRect, HostChromeTabData};

use super::super::super::super::{
    ChromePointerRoute,
    geometry::{contains, translated},
};

pub(super) fn route_document_tab_body(
    surface_key: &str,
    header_frame: &FrameRect,
    tab: &HostChromeTabData,
    row: usize,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let tab_frame = translated(&tab.frame, header_frame.x, header_frame.y);
    if !contains(&tab_frame, x, y) {
        return None;
    }
    Some(ChromePointerRoute::DocumentTab {
        surface_key: surface_key.into(),
        index: row,
        tab_x: tab.frame.x,
        tab_width: tab.frame.width,
        local_x: x - header_frame.x,
        local_y: y - header_frame.y,
        close: false,
    })
}
