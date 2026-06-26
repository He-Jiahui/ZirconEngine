use super::super::super::super::data::{HostPageOverflowMenuStateData, HostWindowPresentationData};
use super::super::super::super::host_page_overflow_menu::{
    host_page_overflow_popup_frame, host_page_overflow_row_frame,
};
use super::super::super::super::menu_popup_metrics::{
    MENU_POPUP_TEXT_INSET_X, MENU_POPUP_TEXT_INSET_Y,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::is_visible_frame;
use super::super::super::super::paint_primitives::{
    draw_border, draw_rect, draw_rect_clipped, draw_text_bars_clipped,
};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::style::{SEPARATOR, TOP_BAR};

pub(in super::super) fn draw_host_page_overflow_menu(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let Some(popup) = host_page_overflow_popup_frame(presentation) else {
        return;
    };
    if !is_visible_frame(&popup) {
        return;
    }

    draw_rect(frame, popup.clone(), TOP_BAR);
    draw_border(frame, popup.clone(), SEPARATOR);

    for row in 0..presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices
        .len()
    {
        let page_index = presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices[row];
        let Some(tab) = presentation
            .host_scene_data
            .page_chrome
            .tabs
            .row_data(page_index)
        else {
            continue;
        };
        let row_frame = host_page_overflow_row_frame(&popup, row);
        if tab.active || is_hovered(&presentation.host_page_overflow_menu_state, page_index) {
            draw_rect_clipped(
                frame,
                row_frame.clone(),
                Some(&popup),
                PALETTE.surface_selected,
            );
        }
        draw_text_bars_clipped(
            frame,
            row_frame.x + MENU_POPUP_TEXT_INSET_X,
            row_frame.y + MENU_POPUP_TEXT_INSET_Y,
            tab.title.as_str(),
            Some(&popup),
            if tab.active {
                PALETTE.text
            } else {
                PALETTE.text_muted
            },
        );
    }
}

fn is_hovered(state: &HostPageOverflowMenuStateData, page_index: usize) -> bool {
    state.hovered_page_index >= 0 && state.hovered_page_index as usize == page_index
}
