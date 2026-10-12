use super::*;

#[test]
fn overflow_row_hit_cannot_select_the_clipped_portion_outside_its_popup() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation.host_layout.status_bar_frame.y = 88.0;

    let popup = host_page_overflow_popup_frame(&presentation)
        .expect("tiny shell should still have a bounded popup viewport");
    let first_row = host_page_overflow_row_frame(&presentation, &popup, 0);
    let clipped_y = popup.y + popup.height + 1.0;

    assert!(clipped_y < first_row.y + first_row.height);
    assert!(host_page_overflow_row_hit_in_popup(
        &presentation,
        &popup,
        first_row.x + 1.0,
        clipped_y,
    )
    .is_none());
}

#[test]
fn overflow_row_hit_does_not_activate_through_the_scrollbar_gutter() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = (0..32).collect();
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 60.0,
    };
    let viewport = host_page_overflow_content_viewport_frame(&popup);

    assert!(host_page_overflow_row_hit_in_popup(
        &presentation,
        &popup,
        viewport.x + viewport.width - 1.0,
        viewport.y + 1.0,
    )
    .is_none());
}

#[test]
fn overflow_row_hit_rejects_the_uniform_row_gap() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![0, 0];
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: menu_popup_outer_padding() + menu_popup_row_stride() * 2.0,
    };
    let first = host_page_overflow_row_frame(&presentation, &popup, 0);
    let gap_y =
        first.y + MENU_POPUP_ROW_HEIGHT + (menu_popup_row_stride() - MENU_POPUP_ROW_HEIGHT) * 0.5;

    assert!(
        host_page_overflow_row_hit_in_popup(&presentation, &popup, first.x + 1.0, gap_y,).is_none()
    );
}

#[test]
fn overflow_row_hit_rejects_a_row_only_touching_the_viewport_bottom() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![0, 0];
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: menu_popup_outer_padding() + menu_popup_row_stride(),
    };
    let viewport = host_page_overflow_content_viewport_frame(&popup);

    assert!(host_page_overflow_row_hit_in_popup(
        &presentation,
        &popup,
        viewport.x + 1.0,
        viewport.y + viewport.height,
    )
    .is_none());
}

#[test]
fn overflow_content_viewport_keeps_its_actual_small_height() {
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: menu_popup_outer_padding(),
    };

    assert_eq!(host_page_overflow_content_viewport_height(&popup), 0.0);
}

#[test]
fn overflow_popup_is_absent_when_the_shell_cannot_offer_a_usable_content_viewport() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation.host_layout.status_bar_frame.y = 64.0;

    assert!(host_page_overflow_popup_frame(&presentation).is_none());
}

#[test]
fn overflow_popup_is_absent_when_a_collapsed_anchor_leaves_no_horizontal_viewport() {
    let mut presentation = overflow_presentation(
        menu_popup_shell_padding() + MENU_POPUP_EDGE_INSET * 2.0,
        "Hidden tab",
    );
    presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
        x: 0.0,
        y: 24.0,
        width: 1.0,
        height: 28.0,
    };

    assert!(host_page_overflow_popup_frame(&presentation).is_none());
}

#[test]
fn overflow_visible_rows_are_exactly_the_rows_intersecting_the_scrolled_viewport() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = (0..32).collect();
    presentation.host_page_overflow_menu_state.scroll_offset = 60.0;
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 60.0,
    };
    let viewport = host_page_overflow_content_viewport_frame(&popup);
    let visible = host_page_overflow_visible_row_range(&presentation, &popup);

    assert!(!visible.is_empty());
    for row in visible.clone() {
        let frame = host_page_overflow_row_frame(&presentation, &popup, row);
        assert!(frame.y < viewport.y + viewport.height);
        assert!(frame.y + frame.height > viewport.y);
    }
    if visible.start > 0 {
        let before = host_page_overflow_row_frame(&presentation, &popup, visible.start - 1);
        assert!(before.y + before.height <= viewport.y);
    }
    if visible.end
        < presentation
            .host_scene_data
            .page_chrome
            .overflow_hidden_tab_indices
            .len()
    {
        let after = host_page_overflow_row_frame(&presentation, &popup, visible.end);
        assert!(after.y >= viewport.y + viewport.height);
    }
}

#[test]
fn overflow_visible_rows_exclude_a_row_that_only_touches_the_viewport_top() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = (0..32).collect();
    presentation.host_page_overflow_menu_state.scroll_offset = MENU_POPUP_ROW_HEIGHT;
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 60.0,
    };

    assert_eq!(
        host_page_overflow_visible_row_range(&presentation, &popup).start,
        1
    );
}

#[test]
fn overflow_scroll_clamps_against_row_content_and_the_exact_inner_viewport() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = (0..32).collect();
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 60.0,
    };
    let expected = host_page_overflow_scroll_content_extent(&presentation)
        - host_page_overflow_content_viewport_frame(&popup).height;

    assert_eq!(
        host_page_overflow_scroll_offset_for_delta(&presentation, &popup, f32::MAX),
        expected
    );
}

#[test]
fn overflow_scroll_ignores_a_non_finite_input_delta() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = (0..32).collect();
    presentation.host_page_overflow_menu_state.scroll_offset = 30.0;
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 60.0,
    };

    assert_eq!(
        host_page_overflow_scroll_offset_for_delta(&presentation, &popup, f32::NAN),
        30.0
    );
}

#[test]
fn overflow_scroll_recovers_a_non_finite_stored_offset() {
    let mut presentation = overflow_presentation(240.0, "Hidden tab");
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = (0..32).collect();
    presentation.host_page_overflow_menu_state.scroll_offset = f32::NAN;
    let popup = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 60.0,
    };

    assert_eq!(
        host_page_overflow_scroll_offset_for_delta(&presentation, &popup, 0.0),
        0.0
    );
}
