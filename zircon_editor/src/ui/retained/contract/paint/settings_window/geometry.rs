//! 设置文字与图标的内部留白共用此投影；至少一单位的载荷尺寸不是父边界保证，消费者仍依赖窗口/列表裁剪。

use super::super::super::data::FrameRect;

pub(super) fn inset_rect(frame: &FrameRect, x: f32, y: f32) -> FrameRect {
    FrameRect {
        x: frame.x + x,
        y: frame.y + y,
        width: (frame.width - x * 2.0).max(1.0),
        height: (frame.height - y * 2.0).max(1.0),
    }
}
