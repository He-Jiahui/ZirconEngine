//! 视口输入使用视口内部像素与已解析选择策略；宿主负责从窗口坐标转换，并在世界域允许时分发。

use crate::scene::selection::SelectionMutation;
use zircon_runtime_interface::math::{UVec2, Vec2};

#[derive(Clone, Debug)]
pub enum ViewportInput {
    PointerMoved(Vec2),
    LeftPressed {
        position: Vec2,
        selection_mutation: SelectionMutation,
    },
    LeftReleased,
    RightPressed(Vec2),
    RightReleased,
    MiddlePressed(Vec2),
    MiddleReleased,
    Scrolled(f32),
    Resized(UVec2),
}
