//! 叠加轴映射到交互工具轴，保证绘制元素与返回拖拽身份使用一致的三轴约定。

use crate::scene::viewport::GizmoAxis;
use crate::scene::viewport::OverlayAxis;

pub(in crate::scene::viewport::pointer) fn gizmo_axis(axis: OverlayAxis) -> GizmoAxis {
    match axis {
        OverlayAxis::X => GizmoAxis::X,
        OverlayAxis::Y => GizmoAxis::Y,
        OverlayAxis::Z => GizmoAxis::Z,
    }
}
