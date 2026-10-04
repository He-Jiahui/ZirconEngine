use crate::core::math::{Real, Vec3};

/// 场景作者的调色参数快照；单位默认值是中性设置，提交阶段再决定是否执行效果。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderColorGradingSettings {
    pub exposure: Real,
    pub contrast: Real,
    pub saturation: Real,
    pub gamma: Real,
    pub tint: Vec3,
}

impl Default for RenderColorGradingSettings {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            tint: Vec3::ONE,
        }
    }
}
