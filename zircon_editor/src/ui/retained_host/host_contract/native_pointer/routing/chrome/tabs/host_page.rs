use crate::ui::retained_host::host_contract::data::HostPageChromeData;

use super::super::super::{geometry::contains, ChromePointerRoute};

pub(in crate::ui::retained_host::host_contract::native_pointer::routing::chrome) fn route_host_page_tabs(
    page_chrome: &HostPageChromeData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let tabs = &page_chrome.tab_frames;
    for (row, tab) in tabs.iter().enumerate() {
        if contains(&tab.close_frame, x, y) {
            return Some(ChromePointerRoute::HostPageTab {
                index: row,
                close: true,
            });
        }
        if contains(&tab.frame, x, y) {
            return Some(ChromePointerRoute::HostPageTab {
                index: row,
                close: false,
            });
        }
    }
    if contains(&page_chrome.overflow_frame, x, y) {
        return Some(ChromePointerRoute::HostPageOverflow);
    }
    None
}
