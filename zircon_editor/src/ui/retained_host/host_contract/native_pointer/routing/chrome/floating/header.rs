use crate::ui::retained_host::host_contract::data::FloatingWindowData;

use super::super::super::{
    geometry::{contains, translated},
    ChromePointerRoute,
};
use super::super::tabs::route_document_tabs;

pub(super) fn route_floating_window_header_hit(
    window: &FloatingWindowData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let header_frame = translated(&window.header_frame, window.frame.x, window.frame.y);
    if !contains(&header_frame, x, y) {
        return None;
    }

    if let Some(route) = route_document_tabs(
        window.window_id.as_str(),
        &header_frame,
        &window.tab_frames,
        x,
        y,
    ) {
        return Some(route);
    }

    Some(ChromePointerRoute::FloatingWindowHeader {
        window_id: window.window_id.clone(),
    })
}
