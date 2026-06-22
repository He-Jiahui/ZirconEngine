use crate::ui::retained_host::host_contract::data::HostChromeTabData;
use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::{geometry::contains, ChromePointerRoute};

pub(in crate::ui::retained_host::host_contract::native_pointer::routing::chrome) fn route_host_page_tabs(
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
