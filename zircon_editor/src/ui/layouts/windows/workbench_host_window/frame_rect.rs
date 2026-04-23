use crate::ui::workbench::autolayout::ShellFrame;

use super::FrameRect;

pub(crate) fn frame_rect(frame: ShellFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
