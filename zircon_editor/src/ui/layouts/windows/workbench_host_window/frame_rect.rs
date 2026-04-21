use crate::ui::slint_host::FrameRect;
use crate::ui::workbench::autolayout::ShellFrame;

pub(crate) fn frame_rect(frame: ShellFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
