use serde::{Deserialize, Serialize};

/// 限制透明绘制分区对应的场景色复制步数；超额请求在设置构造时收敛到此上限。
pub const MAX_SCREEN_SPACE_TRANSMISSION_STEPS: usize = 4;

/// Per-view screen-space specular transmission budget.
///
/// A zero step budget keeps the transmission draw but samples only the
/// environment fallback. Positive values request one scene-color copy per
/// depth-sorted draw partition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScreenSpaceTransmissionSettings {
    steps: u8,
}

impl ScreenSpaceTransmissionSettings {
    /// 构造每个深度排序绘制分区的场景色采样预算；零仍保留环境回退绘制。
    pub const fn new(steps: usize) -> Self {
        Self {
            steps: if steps > MAX_SCREEN_SPACE_TRANSMISSION_STEPS {
                MAX_SCREEN_SPACE_TRANSMISSION_STEPS as u8
            } else {
                steps as u8
            },
        }
    }

    pub const fn steps(self) -> usize {
        self.steps as usize
    }
}

impl Default for ScreenSpaceTransmissionSettings {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
#[path = "tests/screen_space_transmission.rs"]
mod tests;
