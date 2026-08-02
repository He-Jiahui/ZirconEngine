use super::super::super::super::data::FrameRect;
use super::super::super::super::menu_popup_metrics::{
    MENU_POPUP_TEXT_INSET_X, menu_popup_text_width,
};
use crate::ui::retained_host::menu_popup_contract::MENU_POPUP_LABEL_SHORTCUT_GAP;

pub(super) struct MenuRowTextColumns {
    pub(super) label_clip: FrameRect,
    pub(super) shortcut_x: Option<f32>,
}

pub(super) fn menu_row_text_columns(
    row: &FrameRect,
    popup: &FrameRect,
    shortcut: &str,
) -> MenuRowTextColumns {
    let shortcut_x = (!shortcut.is_empty()).then(|| {
        let shortcut_width = menu_popup_text_width(shortcut);
        (row.x + row.width - MENU_POPUP_TEXT_INSET_X - shortcut_width)
            .max(row.x + MENU_POPUP_TEXT_INSET_X)
    });
    let label_right = shortcut_x
        .map(|x| x - MENU_POPUP_LABEL_SHORTCUT_GAP)
        .unwrap_or(row.x + row.width);

    MenuRowTextColumns {
        label_clip: clipped_label_column(row, popup, label_right),
        shortcut_x,
    }
}

fn clipped_label_column(row: &FrameRect, popup: &FrameRect, label_right: f32) -> FrameRect {
    let x = row.x.max(popup.x);
    let y = row.y.max(popup.y);
    let right = label_right
        .min(row.x + row.width)
        .min(popup.x + popup.width);
    let bottom = (row.y + row.height).min(popup.y + popup.height);
    FrameRect {
        x,
        y,
        width: (right - x).max(0.0),
        height: (bottom - y).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_column_reserves_non_overlapping_runtime_measured_label_clip() {
        let row = FrameRect {
            x: 100.0,
            y: 40.0,
            width: 280.0,
            height: 28.0,
        };
        let popup = FrameRect {
            x: 94.0,
            y: 34.0,
            width: 292.0,
            height: 192.0,
        };

        let columns = menu_row_text_columns(&row, &popup, "Ctrl+Shift+U");
        let shortcut_x = columns.shortcut_x.expect("shortcut column");

        assert!(
            columns.label_clip.x + columns.label_clip.width
                <= shortcut_x - MENU_POPUP_LABEL_SHORTCUT_GAP
        );
        assert!(
            shortcut_x + menu_popup_text_width("Ctrl+Shift+U")
                <= row.x + row.width - MENU_POPUP_TEXT_INSET_X + f32::EPSILON
        );
    }

    #[test]
    fn scrolled_row_label_clip_stays_inside_popup_viewport() {
        let row = FrameRect {
            x: 100.0,
            y: 12.0,
            width: 220.0,
            height: 28.0,
        };
        let popup = FrameRect {
            x: 94.0,
            y: 28.0,
            width: 232.0,
            height: 192.0,
        };

        let columns = menu_row_text_columns(&row, &popup, "");

        assert_eq!(columns.label_clip.y, popup.y);
        assert_eq!(columns.label_clip.height, 12.0);
        assert_eq!(columns.shortcut_x, None);
    }
}
