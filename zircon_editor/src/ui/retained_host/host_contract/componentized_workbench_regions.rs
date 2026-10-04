mod panes;

pub(in crate::ui::retained_host) use panes::{
    authored_hierarchy, authored_panes, is_authored_tree_row, owns_ordinary_panes, point_in_frame,
};

use super::data::{FrameRect, HostWindowLayoutData};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ComponentizedWorkbenchChromeRegions {
    pub(super) top_chrome: FrameRect,
    pub(super) status_bar: FrameRect,
}

pub(super) fn componentized_workbench_chrome_regions(
    layout: &HostWindowLayoutData,
    frame_bounds: &FrameRect,
) -> Option<ComponentizedWorkbenchChromeRegions> {
    if !visible_rect(&layout.center_band_frame) || !visible_rect(&layout.status_bar_frame) {
        return None;
    }

    let top_height = layout
        .center_band_frame
        .y
        .clamp(frame_bounds.y, frame_bounds.y + frame_bounds.height)
        - frame_bounds.y;
    Some(ComponentizedWorkbenchChromeRegions {
        top_chrome: FrameRect {
            x: frame_bounds.x,
            y: frame_bounds.y,
            width: frame_bounds.width,
            height: top_height,
        },
        status_bar: intersect_rect(&layout.status_bar_frame, frame_bounds).unwrap_or_default(),
    })
}

fn visible_rect(rect: &FrameRect) -> bool {
    rect.width > 0.0 && rect.height > 0.0
}

fn intersect_rect(rect: &FrameRect, bounds: &FrameRect) -> Option<FrameRect> {
    let left = rect.x.max(bounds.x);
    let top = rect.y.max(bounds.y);
    let right = (rect.x + rect.width).min(bounds.x + bounds.width);
    let bottom = (rect.y + rect.height).min(bounds.y + bounds.height);
    (right > left && bottom > top).then_some(FrameRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

#[cfg(test)]
#[path = "tests/componentized_workbench_regions.rs"]
mod tests;
