use super::super::super::super::data::FrameRect;
use super::super::super::super::menu_popup_metrics::{
    menu_popup_text_width, MENU_POPUP_TEXT_INSET_X,
};
use crate::ui::retained_host::menu_popup_contract::MENU_POPUP_LABEL_SHORTCUT_GAP;

pub(super) struct MenuRowTextColumns {
    pub(super) label_clip: FrameRect,
    pub(super) shortcut_x: Option<f32>,
}

// 快捷键列先预留实际测量宽度，再裁剪主标签列，避免两类文字在窄菜单中互相覆盖。
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
#[path = "tests/text_layout.rs"]
mod tests;
