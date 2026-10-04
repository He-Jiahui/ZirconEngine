use crate::core::math::Real;

/// 由相机或体积覆盖的模糊半径；效果栈仅在正半径时加入模糊节点。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderBlurSettings {
    pub radius: Real,
}

impl Default for RenderBlurSettings {
    fn default() -> Self {
        Self { radius: 0.0 }
    }
}

impl RenderBlurSettings {
    pub fn is_enabled(self) -> bool {
        self.radius > 0.0
    }

    pub fn render_radius(self) -> Real {
        self.radius.max(0.0)
    }
}

#[cfg(test)]
#[path = "tests/blur_settings.rs"]
mod tests;
