use super::super::super::super::data::FrameRect;
use super::super::metrics::WorkbenchPopupRowMetrics;
use crate::ui::retained_host::host_contract::menu_popup_text_width;
use crate::ui::retained_host::menu_popup_contract::MENU_POPUP_LABEL_SHORTCUT_GAP;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct PopupRowTextColumns {
    pub label: FrameRect,
    pub shortcut: Option<FrameRect>,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_text_columns(
    row_rect: &FrameRect,
    metrics: &WorkbenchPopupRowMetrics,
    shortcut: &str,
    adornment_present: bool,
) -> PopupRowTextColumns {
    let right_reserved = if adornment_present {
        metrics.adornment_reserved_width
    } else {
        metrics.text_right
    };
    let content_left = row_rect.x + metrics.text_left;
    let content_right = (row_rect.x + row_rect.width - right_reserved).max(content_left);
    let text_y = row_rect.y + metrics.text_top;
    let text_height = popup_row_text_height(row_rect, metrics);
    let shortcut = (!shortcut.is_empty()).then(|| {
        let available_width = (content_right - content_left).max(0.0);
        let width = menu_popup_text_width(shortcut).min(available_width);
        FrameRect {
            x: content_right - width,
            y: text_y,
            width,
            height: text_height,
        }
    });
    let label_right = shortcut
        .as_ref()
        .map(|shortcut| (shortcut.x - MENU_POPUP_LABEL_SHORTCUT_GAP).max(content_left))
        .unwrap_or(content_right);

    PopupRowTextColumns {
        label: FrameRect {
            x: content_left,
            y: text_y,
            width: (label_right - content_left).max(0.0),
            height: text_height,
        },
        shortcut,
    }
}

fn popup_row_text_height(row_rect: &FrameRect, metrics: &WorkbenchPopupRowMetrics) -> f32 {
    (row_rect.height - metrics.text_top - metrics.text_bottom).max(0.0)
}

#[cfg(test)]
#[path = "tests/geometry.rs"]
mod tests;
