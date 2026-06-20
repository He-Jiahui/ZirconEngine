use crate::ui::workbench::autolayout::ShellFrame;
use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

pub(super) fn is_valid_size(size: UiSize) -> bool {
    size.width > 0.0 && size.height > 0.0
}

pub(super) fn frame_size(frame: ShellFrame) -> Option<UiSize> {
    let size = UiSize::new(frame.width.max(0.0), frame.height.max(0.0));
    is_valid_size(size).then_some(size)
}

pub(super) fn ui_frame_size(frame: UiFrame) -> Option<UiSize> {
    let size = UiSize::new(frame.width.max(0.0), frame.height.max(0.0));
    is_valid_size(size).then_some(size)
}
