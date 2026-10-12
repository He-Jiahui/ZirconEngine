use std::rc::Rc;

use super::*;
use crate::ui::retained_host::host_contract::data::TabData;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

#[test]
fn overflow_palette_projects_each_runtime_theme_role() {
    let mut palette = PALETTE;
    palette.popup = [1, 2, 3, 4];
    palette.border = [5, 6, 7, 8];
    palette.surface_hover = [9, 10, 11, 12];
    palette.accent = [13, 14, 15, 16];
    palette.text = [17, 18, 19, 20];
    palette.text_muted = [21, 22, 23, 24];

    assert_eq!(
        page_overflow_palette(palette),
        PageOverflowPalette {
            popup: [1, 2, 3, 4],
            border: [5, 6, 7, 8],
            hover: [9, 10, 11, 12],
            accent: [13, 14, 15, 16],
            text: [17, 18, 19, 20],
            text_muted: [21, 22, 23, 24],
        }
    );
}

#[test]
fn overflowing_row_title_reserves_the_scrollbar_and_its_gap() {
    let row = FrameRect {
        x: 100.0,
        y: 40.0,
        width: 100.0,
        height: 28.0,
    };
    let scrollbar_reserve = 12.0;

    let title = overflow_row_title_frame(&row, 0.0, scrollbar_reserve);

    assert_eq!(
        title.x + title.width,
        row.x + row.width - MENU_POPUP_TEXT_INSET_X - scrollbar_reserve
    );
}

#[test]
fn overflow_row_title_uses_a_finite_runtime_text_slot_with_ellipsis() {
    let presentation = overflow_presentation("A long hidden editor tab title that must ellipsize");
    let mut frame = HostRgbaFrame::recording_only(240, 180);

    draw_host_page_overflow_menu(&mut frame, &presentation);

    let popup = host_page_overflow_popup_frame(&presentation)
        .expect("open overflow should provide a popup frame");
    let row = host_page_overflow_row_frame(&presentation, &popup, 0);
    let command = frame
        .into_recorded_commands()
        .into_iter()
        .find(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
        .expect("overflow row title should use Runtime Text");
    let HostRecordedPaintKind::Text { text, .. } = &command.kind else {
        unreachable!("filtered command should be text");
    };

    assert!(text.ends_with('\u{2026}'));
    assert!(command.frame.x >= row.x + MENU_POPUP_TEXT_INSET_X);
    assert!(command.frame.x + command.frame.width <= row.x + row.width - MENU_POPUP_TEXT_INSET_X);
}

#[test]
fn overflow_popup_and_hover_row_use_panel_and_small_radius_tiers() {
    let mut presentation = overflow_presentation("Scene");
    presentation
        .host_page_overflow_menu_state
        .hovered_page_index = 0;
    let mut frame = HostRgbaFrame::recording_only(240, 180);

    draw_host_page_overflow_menu(&mut frame, &presentation);

    let metrics = current_host_metrics();
    let palette = page_overflow_palette(current_host_palette());
    let commands = frame.into_recorded_commands();
    let popup = commands
        .iter()
        .find(|command| {
            matches!(
                &command.kind,
                HostRecordedPaintKind::Quad { color, .. } if *color == palette.popup
            )
        })
        .expect("page overflow should paint its popup surface");
    let hover = commands
        .iter()
        .find(|command| {
            matches!(
                &command.kind,
                HostRecordedPaintKind::Quad { color, .. } if *color == palette.hover
            )
        })
        .expect("hovered overflow row should paint its surface");

    assert!(matches!(
        &popup.kind,
        HostRecordedPaintKind::Quad { corner_radius, .. }
            if *corner_radius == metrics.radius_panel
    ));
    assert!(matches!(
        &hover.kind,
        HostRecordedPaintKind::Quad { corner_radius, .. }
            if *corner_radius == metrics.radius_small
    ));
}

fn overflow_presentation(title: &str) -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_layout.status_bar_frame = FrameRect {
        x: 0.0,
        y: 160.0,
        width: 240.0,
        height: 20.0,
    };
    presentation.host_scene_data.page_chrome.overflow_frame = FrameRect {
        x: 188.0,
        y: 24.0,
        width: 34.0,
        height: 28.0,
    };
    presentation.host_scene_data.page_chrome.tabs = model_rc(vec![TabData {
        id: "long-tab".into(),
        title: title.into(),
        ..TabData::default()
    }]);
    let metrics = current_host_metrics();
    presentation
        .host_scene_data
        .page_chrome
        .overflow_widest_title_width_px =
        measure_runtime_text_width(title, metrics.font_body) + metrics.text_clip_guard;
    presentation
        .host_scene_data
        .page_chrome
        .overflow_hidden_tab_indices = vec![0];
    presentation.host_page_overflow_menu_state = HostPageOverflowMenuStateData {
        open: true,
        hovered_page_index: -1,
        scroll_offset: 0.0,
    };
    presentation
}

fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(rows)))
}
