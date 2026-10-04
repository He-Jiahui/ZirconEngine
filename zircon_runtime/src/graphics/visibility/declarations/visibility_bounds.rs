use crate::core::math::{Real, Vec3};

/// 用于 CPU 视锥及空间索引的世界空间包围球；无效资源边界由生产者采取 fail-open。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VisibilityBounds {
    pub center: Vec3,
    pub radius: Real,
}
