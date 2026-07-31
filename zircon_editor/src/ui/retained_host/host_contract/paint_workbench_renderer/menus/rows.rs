mod text_layout;

use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, HostMenuChromeItemData, HostWindowPresentationData};
use super::super::super::menu_popup_metrics::MENU_POPUP_TEXT_INSET_X;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::draw_rect_clipped;
use super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::paint_theme::{current_host_metrics, current_host_palette};
use super::geometry::menu_popup_row_frame;
use text_layout::menu_row_text_columns;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract) fn draw_menu_popup_rows(
    frame: &mut HostRgbaFrame,
    items: &ModelRc<HostMenuChromeItemData>,
    popup: &FrameRect,
    level: usize,
    scroll_px: f32,
    presentation: &HostWindowPresentationData,
) {
    let metrics = current_host_metrics();
    let palette = current_host_palette();
    let line_height = metrics
        .line_height(metrics.font_body)
        .round()
        .max(metrics.font_body.ceil());
    for row in 0..items.row_count() {
        let Some(item) = items.row_data(row) else {
            continue;
        };
        let row_frame = menu_popup_row_frame(popup, row, scroll_px);
        let hovered = presentation
            .menu_state
            .hovered_menu_item_path
            .get(level)
            .is_some_and(|hovered_row| *hovered_row == row);
        if hovered {
            draw_rect_clipped(frame, row_frame.clone(), Some(popup), palette.surface_hover);
        }
        let text_color = if item.enabled {
            palette.text
        } else {
            palette.text_disabled
        };
        let text_columns = menu_row_text_columns(&row_frame, popup, item.shortcut.as_str());
        let label_frame = menu_row_text_frame(
            &row_frame,
            row_frame.x + MENU_POPUP_TEXT_INSET_X,
            text_columns.label_clip.x + text_columns.label_clip.width,
            line_height,
        );
        draw_text_with_size_and_style(
            frame,
            label_frame,
            item.label.as_str(),
            Some(&text_columns.label_clip),
            text_color,
            metrics.font_body,
            line_height,
            UiTextRunPaintStyle::default(),
        );
        if let Some(shortcut_x) = text_columns.shortcut_x {
            let shortcut_frame = menu_row_text_frame(
                &row_frame,
                shortcut_x,
                row_frame.x + row_frame.width - MENU_POPUP_TEXT_INSET_X,
                line_height,
            );
            draw_text_with_size_and_style(
                frame,
                shortcut_frame,
                item.shortcut.as_str(),
                Some(popup),
                text_color,
                metrics.font_body,
                line_height,
                UiTextRunPaintStyle::default(),
            );
        }
    }
}

fn menu_row_text_frame(row: &FrameRect, x: f32, right: f32, line_height: f32) -> FrameRect {
    let line_height = line_height.min(row.height.max(1.0));
    FrameRect {
        x,
        y: row.y + ((row.height - line_height).max(0.0) * 0.5),
        width: (right - x).max(1.0),
        height: line_height,
    }
}

#[cfg(test)]
mod tests {
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

    fn model_rc<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
        ModelRc::from(Rc::new(VecModel::from(rows)))
    }
}
