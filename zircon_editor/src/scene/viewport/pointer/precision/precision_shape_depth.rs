//! 投影深度只作同类近似屏幕评分的次级排序，不能当世界射线交点距离或编辑域身份。

use super::PrecisionShape;

impl PrecisionShape {
    pub(in crate::scene::viewport::pointer) fn depth(&self) -> f32 {
        match self {
            Self::Line { depth, .. } | Self::Circle { depth, .. } | Self::Ring { depth, .. } => {
                *depth
            }
        }
    }
}
