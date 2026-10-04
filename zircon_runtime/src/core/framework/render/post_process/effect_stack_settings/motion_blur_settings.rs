use crate::core::math::Real;

const DEFAULT_MOTION_BLUR_SAMPLES: u32 = 1;
const MAX_MOTION_BLUR_SAMPLES: u32 = 32;

/// 时间重建后的运动模糊请求，执行时依赖场景速度及运动向量预通道。
/// 诊断报告会单独列出资源缺口，因此启用请求本身不代表当前帧可完整执行。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMotionBlurSettings {
    pub shutter_angle: Real,
    pub samples: u32,
}

impl Default for RenderMotionBlurSettings {
    fn default() -> Self {
        Self {
            shutter_angle: 0.0,
            samples: DEFAULT_MOTION_BLUR_SAMPLES,
        }
    }
}

impl RenderMotionBlurSettings {
    pub fn is_enabled(self) -> bool {
        self.shutter_angle > 0.0 && self.samples > 0
    }

    pub fn render_shutter_angle(self) -> Real {
        if self.is_enabled() {
            self.shutter_angle.max(0.0)
        } else {
            0.0
        }
    }

    pub fn render_samples(self) -> u32 {
        self.samples.min(MAX_MOTION_BLUR_SAMPLES)
    }
}

#[cfg(test)]
#[path = "tests/motion_blur_settings.rs"]
mod tests;
