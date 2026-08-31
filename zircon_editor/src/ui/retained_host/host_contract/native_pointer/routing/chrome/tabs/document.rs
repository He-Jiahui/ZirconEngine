mod body;
mod close;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostChromeTabData};
use crate::ui::retained_host::primitives::ModelRc;

use self::body::route_document_tab_body;
use self::close::route_document_tab_close;
use super::super::super::ChromePointerRoute;

pub(in crate::ui::retained_host::host_contract::native_pointer::routing::chrome) fn route_document_tabs(
    surface_key: &str,
    header_frame: &FrameRect,
    tabs: &ModelRc<HostChromeTabData>,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    for (row, tab) in tabs.iter().enumerate() {
        if let Some(route) = route_document_tab_close(surface_key, header_frame, tab, row, x, y) {
            return Some(route);
        }
        if let Some(route) = route_document_tab_body(surface_key, header_frame, tab, row, x, y) {
            return Some(route);
        }
    }
    None
}
