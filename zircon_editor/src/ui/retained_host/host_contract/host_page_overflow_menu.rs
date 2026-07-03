use super::data::{FrameRect, HostWindowPresentationData};
use super::menu_popup_metrics::{
    menu_popup_outer_padding, menu_popup_row_stride, MENU_POPUP_ANCHOR_GAP, MENU_POPUP_EDGE_INSET,
    MENU_POPUP_ROW_HEIGHT,
};
use crate::ui::retained_host::popup_anchor_metrics::clamp_popup_x_to_bounds;
use crate::ui::workbench::page_tabs::MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH;

pub(in crate::ui::retained_host::host_contract) struct HostPageOverflowRowHit {
    pub page_index: usize,
    pub frame: FrameRect,
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_popup_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    if !presentation.host_page_overflow_menu_state.open {
        return None;
    }
    let overflow = &presentation.host_scene_data.page_chrome.overflow_frame;
    if overflow.width <= 0.0 || overflow.height <= 0.0 {
        return None;
    }
    let item_count = presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices
        .len();
    if item_count == 0 {
        return None;
    }
    let height = menu_popup_outer_padding()
        + item_count as f32 * MENU_POPUP_ROW_HEIGHT
        + item_count.saturating_sub(1) as f32 * (menu_popup_row_stride() - MENU_POPUP_ROW_HEIGHT);
    let shell_width = presentation
        .host_layout
        .status_bar_frame
        .width
        .max(presentation.host_layout.center_band_frame.width)
        .max(overflow.x + overflow.width);
    let x = clamp_popup_x_to_bounds(
        overflow.x + overflow.width - MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH,
        0.0,
        shell_width,
        MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH,
    );
    Some(FrameRect {
        x,
        y: overflow.y + overflow.height + MENU_POPUP_ANCHOR_GAP,
        width: MAIN_PAGE_TAB_OVERFLOW_POPUP_WIDTH,
        height,
    })
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_row_frame(
    popup: &FrameRect,
    row: usize,
) -> FrameRect {
    FrameRect {
        x: popup.x + MENU_POPUP_EDGE_INSET,
        y: popup.y + MENU_POPUP_EDGE_INSET + row as f32 * menu_popup_row_stride(),
        width: (popup.width - MENU_POPUP_EDGE_INSET * 2.0).max(0.0),
        height: MENU_POPUP_ROW_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_row_hit(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<HostPageOverflowRowHit> {
    let popup = host_page_overflow_popup_frame(presentation)?;
    for row in 0..presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices
        .len()
    {
        let frame = host_page_overflow_row_frame(&popup, row);
        if contains(&frame, x, y) {
            return Some(HostPageOverflowRowHit {
                page_index: presentation
                    .host_scene_data
                    .page_chrome
                    .overflow_hidden_tab_indices[row],
                frame,
            });
        }
    }
    None
}

pub(in crate::ui::retained_host::host_contract) fn host_page_overflow_popup_contains(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> bool {
    host_page_overflow_popup_frame(presentation).is_some_and(|frame| contains(&frame, x, y))
}

fn contains(frame: &FrameRect, x: f32, y: f32) -> bool {
    x >= frame.x && y >= frame.y && x <= frame.x + frame.width && y <= frame.y + frame.height
}
