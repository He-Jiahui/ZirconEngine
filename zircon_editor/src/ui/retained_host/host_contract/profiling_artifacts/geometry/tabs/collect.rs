use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::super::data::{FrameRect, HostChromeTabData};
use super::super::super::UiProfileTabFrame;
use super::super::frame_math::{is_visible_frame, translated};

pub(super) fn collect_tabs(
    kind: &str,
    surface: &str,
    tabs: &ModelRc<HostChromeTabData>,
    origin: &FrameRect,
) -> Vec<UiProfileTabFrame> {
    let mut out = Vec::with_capacity(tabs.row_count());
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

#[cfg(test)]
#[path = "collect/capacity_tests.rs"]
mod capacity_tests;
