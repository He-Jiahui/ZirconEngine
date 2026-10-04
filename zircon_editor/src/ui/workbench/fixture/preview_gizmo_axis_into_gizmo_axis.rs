use crate::scene::viewport::GizmoAxis;

use super::PreviewGizmoAxis;

impl PreviewGizmoAxis {
    /// 使预览悬停轴与运行时视口轴共用身份；实际拾取结果来自宿主。
    pub(crate) fn into_gizmo_axis(self) -> GizmoAxis {
        match self {
            Self::X => GizmoAxis::X,
            Self::Y => GizmoAxis::Y,
            Self::Z => GizmoAxis::Z,
        }
    }
}
