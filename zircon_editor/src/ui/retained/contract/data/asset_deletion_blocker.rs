use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::FrameRect;

const DIALOG_MAX_WIDTH: f32 = 620.0;
const DIALOG_MAX_HEIGHT: f32 = 360.0;
const WINDOW_INSET: f32 = 16.0;
const CONTENT_INSET: f32 = 18.0;
const HEADER_HEIGHT: f32 = 76.0;
const FOOTER_HEIGHT: f32 = 58.0;
const REFERENCER_ROW_HEIGHT: f32 = 22.0;
const CLOSE_BUTTON_WIDTH: f32 = 88.0;
const CLOSE_BUTTON_HEIGHT: f32 = 30.0;

/// Retained presentation source for a safe-delete referencer blocker.
///
/// `referencers` intentionally retains the complete offline-registry result. Painting is bounded by
/// `visible_referencer_rows`; consumers such as accessibility and diagnostics still see every row.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HostAssetDeletionBlockerData {
    pub visible: bool,
    pub target: SharedString,
    pub referencers: ModelRc<SharedString>,
    pub visible_referencer_rows: usize,
    pub overflow_label: SharedString,
    pub overlay_frame: FrameRect,
    pub dialog_frame: FrameRect,
    pub referencer_list_frame: FrameRect,
    pub close_button_frame: FrameRect,
}

impl HostAssetDeletionBlockerData {
    pub(crate) fn for_window(
        width: f32,
        height: f32,
        target: SharedString,
        referencers: ModelRc<SharedString>,
    ) -> Self {
        let width = width.max(0.0);
        let height = height.max(0.0);
        let available_width = (width - WINDOW_INSET * 2.0).max(0.0);
        let available_height = (height - WINDOW_INSET * 2.0).max(0.0);
        let dialog_width = available_width.min(DIALOG_MAX_WIDTH);
        let dialog_height = available_height.min(DIALOG_MAX_HEIGHT);
        let dialog_frame = FrameRect {
            x: ((width - dialog_width) * 0.5).max(0.0),
            y: ((height - dialog_height) * 0.5).max(0.0),
            width: dialog_width,
            height: dialog_height,
        };
        let content_width = (dialog_width - CONTENT_INSET * 2.0).max(0.0);
        let list_height = (dialog_height - HEADER_HEIGHT - FOOTER_HEIGHT).max(0.0);
        let referencer_list_frame = FrameRect {
            x: dialog_frame.x + CONTENT_INSET.min(dialog_width * 0.5),
            y: (dialog_frame.y + HEADER_HEIGHT).min(dialog_frame.y + dialog_height),
            width: content_width,
            height: list_height,
        };
        let close_width = CLOSE_BUTTON_WIDTH.min(content_width);
        let close_height = CLOSE_BUTTON_HEIGHT.min(dialog_height);
        let close_button_frame = FrameRect {
            x: (dialog_frame.x + dialog_width - CONTENT_INSET - close_width).max(dialog_frame.x),
            y: (dialog_frame.y + dialog_height - CONTENT_INSET - close_height).max(dialog_frame.y),
            width: close_width,
            height: close_height,
        };
        let visible_referencer_rows =
            ((list_height / REFERENCER_ROW_HEIGHT).floor() as usize).min(referencers.row_count());
        let visible_text_rows = if referencers.row_count() > visible_referencer_rows {
            visible_referencer_rows.saturating_sub(1)
        } else {
            visible_referencer_rows
        };
        let hidden_referencer_count = referencers.row_count().saturating_sub(visible_text_rows);
        let overflow_label = if hidden_referencer_count > 0 {
            format!("{hidden_referencer_count} more referencers")
        } else {
            SharedString::new()
        };
        Self {
            visible: true,
            target,
            referencers,
            visible_referencer_rows,
            overflow_label,
            overlay_frame: FrameRect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
            dialog_frame,
            referencer_list_frame,
            close_button_frame,
        }
    }

    pub(crate) const fn referencer_row_height() -> f32 {
        REFERENCER_ROW_HEIGHT
    }

    pub(crate) fn relayout(&self, width: f32, height: f32) -> Self {
        if !self.visible {
            return Self::default();
        }
        Self::for_window(width, height, self.target.clone(), self.referencers.clone())
    }
}
