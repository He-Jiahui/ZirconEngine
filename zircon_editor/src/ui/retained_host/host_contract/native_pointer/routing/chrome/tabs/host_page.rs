use crate::ui::retained_host::host_contract::data::HostPageChromeData;

use super::super::super::{geometry::contains, ChromePointerRoute};

pub(in crate::ui::retained_host::host_contract::native_pointer::routing::chrome) fn route_host_page_tabs(
    page_chrome: &HostPageChromeData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let tabs = &page_chrome.tab_frames;
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
    if contains(&page_chrome.overflow_frame, x, y) {
        return Some(ChromePointerRoute::HostPageOverflow {
            tab_x: page_chrome.overflow_frame.x,
            tab_width: page_chrome.overflow_frame.width,
            local_x: x - page_chrome.overflow_frame.x,
            local_y: y - page_chrome.overflow_frame.y,
        });
    }
    None
}
