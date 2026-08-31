use super::super::super::super::data::FrameRect;
use super::super::metrics::WorkbenchPopupRowMetrics;
use crate::ui::retained_host::host_contract::menu_popup_text_width;
use crate::ui::retained_host::menu_popup_contract::MENU_POPUP_LABEL_SHORTCUT_GAP;

pub(super) struct PopupRowTextColumns {
    pub label: FrameRect,
    pub shortcut: Option<FrameRect>,
}

pub(super) fn popup_row_text_columns(
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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_template_nodes::template_popup_rows::metrics::workbench_popup_row_metrics;

    #[test]
    fn shortcut_and_adornment_columns_never_overlap_the_label_or_each_other() {
        let metrics = workbench_popup_row_metrics();
        let row = FrameRect {
            x: 20.0,
            y: 16.0,
            width: 240.0,
            height: 28.0,
        };

        let columns = popup_row_text_columns(&row, &metrics, "Ctrl+Shift+S", true);
        let shortcut = columns.shortcut.expect("shortcut column");
        let adornment_left = row.x + row.width - metrics.adornment_reserved_width;

        assert!(
            columns.label.x + columns.label.width + MENU_POPUP_LABEL_SHORTCUT_GAP
                <= shortcut.x + f32::EPSILON
        );
        assert!(shortcut.x + shortcut.width <= adornment_left + f32::EPSILON);
    }

    #[test]
    fn measured_shortcuts_align_to_the_same_trailing_content_edge() {
        let metrics = workbench_popup_row_metrics();
        let row = FrameRect {
            x: 20.0,
            y: 16.0,
            width: 240.0,
            height: 28.0,
        };

        let short = popup_row_text_columns(&row, &metrics, "F5", false)
            .shortcut
            .expect("short shortcut");
        let long = popup_row_text_columns(&row, &metrics, "Ctrl+Shift+F5", false)
            .shortcut
            .expect("long shortcut");

        assert!(long.x < short.x);
        assert!((long.x + long.width - (short.x + short.width)).abs() <= f32::EPSILON);
        assert!(long.x + long.width <= row.x + row.width - metrics.text_right + f32::EPSILON);
    }
}
