use crate::ui::retained_host::{FrameRect, UiHostWindow};

const BUTTON_WIDTH: f32 = 88.0;
const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_GAP: f32 = 10.0;

pub(super) struct ClosePromptLayout {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) dialog: FrameRect,
    pub(super) save: FrameRect,
    pub(super) discard: FrameRect,
    pub(super) cancel: FrameRect,
}

pub(super) fn close_prompt_layout(ui: &UiHostWindow) -> ClosePromptLayout {
    let size = ui.window().size();
    let width = size.width as f32;
    let height = size.height as f32;
    let dialog_width = (width - 48.0).clamp(280.0, 500.0);
    let dialog_height = 176.0;
    let dialog = FrameRect {
        x: ((width - dialog_width) * 0.5).max(16.0),
        y: ((height - dialog_height) * 0.5).max(16.0),
        width: dialog_width,
        height: dialog_height,
    };
    ClosePromptLayout {
        width,
        height,
        save: button_frame(&dialog, 2),
        discard: button_frame(&dialog, 1),
        cancel: button_frame(&dialog, 0),
        dialog,
    }
}

fn button_frame(dialog: &FrameRect, reverse_index: usize) -> FrameRect {
    let right = dialog.x + dialog.width - 18.0;
    FrameRect {
        x: right - BUTTON_WIDTH - reverse_index as f32 * (BUTTON_WIDTH + BUTTON_GAP),
        y: dialog.y + dialog.height - BUTTON_HEIGHT - 18.0,
        width: BUTTON_WIDTH,
        height: BUTTON_HEIGHT,
    }
}
