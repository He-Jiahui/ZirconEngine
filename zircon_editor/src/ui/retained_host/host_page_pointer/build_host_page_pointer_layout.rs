use zircon_runtime_interface::ui::layout::UiFrame;

use crate::ui::retained_host::callback_dispatch::BuiltinHostOuterShellFrames;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::page_tabs::{
    MAIN_PAGE_TAB_GAP, MAIN_PAGE_TAB_HEIGHT, MAIN_PAGE_TAB_MAX_WIDTH, MAIN_PAGE_TAB_STRIP_X,
};

use super::host_page_pointer_item::HostPagePointerItem;
use super::host_page_pointer_layout::HostPagePointerLayout;
use super::tab_strip_geometry::allocate_host_page_tabs;

pub(crate) fn build_host_page_pointer_layout(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    outer_shell_frames: Option<&BuiltinHostOuterShellFrames>,
) -> HostPagePointerLayout {
    let estimated_width = MAIN_PAGE_TAB_STRIP_X * 2.0
        + model.host_strip.pages.len() as f32 * MAIN_PAGE_TAB_MAX_WIDTH
        + model.host_strip.pages.len().saturating_sub(1) as f32 * MAIN_PAGE_TAB_GAP;
    let shared_strip_frame = outer_shell_frames.and_then(|frames| frames.host_page_strip_frame);
    let shared_shell_frame = outer_shell_frames.and_then(|frames| frames.shell_frame);
    let strip_x = shared_strip_frame
        .map(|frame| frame.x)
        .or_else(|| shared_shell_frame.map(|frame| frame.x))
        .unwrap_or(0.0);
    let strip_y = shared_strip_frame
        .map(|frame| frame.y)
        .or_else(|| {
            shared_shell_frame
                .map(|frame| frame.y + metrics.top_bar_height + metrics.separator_thickness)
        })
        .unwrap_or(0.0);
    let strip_width = shared_strip_frame
        .map(|frame| frame.width.max(1.0))
        .or_else(|| shared_shell_frame.map(|frame| frame.width.max(estimated_width.max(1.0))))
        .unwrap_or(estimated_width.max(1.0));
    let strip_height = shared_strip_frame
        .map(|frame| frame.height.max(0.0))
        .unwrap_or_else(|| metrics.host_bar_height.max(MAIN_PAGE_TAB_HEIGHT));
    let strip_frame = UiFrame::new(strip_x, strip_y, strip_width, strip_height);
    let items = model
        .host_strip
        .pages
        .iter()
        .map(|page| HostPagePointerItem {
            page_id: page.id.0.clone(),
            title: page.title.clone(),
        })
        .collect::<Vec<_>>();
    let active_index = items
        .iter()
        .position(|item| item.page_id == model.host_strip.active_page.0.as_str());
    let (tabs, overflow) = allocate_host_page_tabs(strip_frame, &items, active_index);

    HostPagePointerLayout {
        strip_frame,
        items,
        tabs,
        overflow,
    }
}
