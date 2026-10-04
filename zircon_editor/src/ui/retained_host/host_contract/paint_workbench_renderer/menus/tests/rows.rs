use std::rc::Rc;

use super::*;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};

#[test]
fn menu_row_text_frame_is_finite_and_vertically_centered() {
    let row = FrameRect {
        x: 40.0,
        y: 32.0,
        width: 240.0,
        height: 28.0,
    };

    let frame = menu_row_text_frame(&row, 48.0, 168.0, 16.0);

    assert_eq!(frame.x, 48.0);
    assert_eq!(frame.y, 38.0);
    assert_eq!(frame.width, 120.0);
    assert_eq!(frame.height, 16.0);
}

#[test]
fn menu_row_label_uses_a_finite_runtime_text_slot_with_ellipsis() {
    let items = model_rc(vec![HostMenuChromeItemData {
        label: "A long menu item label that must ellipsize inside its popup row".into(),
        enabled: true,
        ..HostMenuChromeItemData::default()
    }]);
    let popup = FrameRect {
        x: 20.0,
        y: 24.0,
        width: 140.0,
        height: 48.0,
    };
    let mut frame = HostRgbaFrame::recording_only(180, 96);

    draw_menu_popup_rows(
        &mut frame,
        &items,
        &popup,
        0,
        0.0,
        &HostWindowPresentationData::default(),
    );

    let command = frame
        .into_recorded_commands()
        .into_iter()
        .find(|command| matches!(&command.kind, HostRecordedPaintKind::Text { .. }))
        .expect("menu label should use Runtime Text");
    let HostRecordedPaintKind::Text { text, .. } = &command.kind else {
        unreachable!("filtered command should be text");
    };

    assert!(text.ends_with('\u{2026}'));
    assert!(command.frame.x >= popup.x + MENU_POPUP_TEXT_INSET_X);
    assert!(command.frame.x + command.frame.width <= popup.x + popup.width);
}

#[test]
fn hovered_menu_row_uses_the_small_radius_tier() {
    let items = model_rc(vec![HostMenuChromeItemData {
        label: "Open".into(),
        enabled: true,
        ..HostMenuChromeItemData::default()
    }]);
    let popup = FrameRect {
        x: 20.0,
        y: 24.0,
        width: 180.0,
        height: 48.0,
    };
    let mut presentation = HostWindowPresentationData::default();
    presentation.menu_state.hovered_menu_item_path = vec![0];
    let mut frame = HostRgbaFrame::recording_only(240, 96);

    draw_menu_popup_rows(&mut frame, &items, &popup, 0, 0.0, &presentation);

    let hover_color = current_host_palette().surface_hover;
    let hover = frame
        .into_recorded_commands()
        .into_iter()
        .find(|command| {
            matches!(
                &command.kind,
                HostRecordedPaintKind::Quad { color, .. } if *color == hover_color
            )
        })
        .expect("hovered menu row should paint a surface quad");
    let HostRecordedPaintKind::Quad { corner_radius, .. } = hover.kind else {
        unreachable!("filtered command should be a quad");
    };

    assert_eq!(corner_radius, current_host_metrics().radius_small);
}

fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(rows)))
}
