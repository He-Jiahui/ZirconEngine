use crate::core::math::{Real, Vec2};
use serde::{Deserialize, Serialize};

/// 一个 TAA 样本的像素空间偏移；投影矩阵负责按当前视口尺寸转为裁剪空间。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TemporalJitterSample {
    pub offset_pixels: Vec2,
    pub sequence_index: u32,
}

/// 由帧索引驱动的周期性样本序列；关闭 TAA 时调用方应传默认零偏移。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalJitterSequence {
    period: u32,
}

impl TemporalJitterSequence {
    pub fn new(period: u32) -> Self {
        Self {
            period: period.max(1),
        }
    }

    pub fn sample(self, frame_index: u64) -> TemporalJitterSample {
        let sequence_index = (frame_index % u64::from(self.period)) as u32 + 1;
        TemporalJitterSample {
            offset_pixels: Vec2::new(
                halton(sequence_index, 2) - 0.5,
                halton(sequence_index, 3) - 0.5,
            ),
            sequence_index,
        }
    }

    pub const fn period(self) -> u32 {
        self.period
    }
}

pub fn halton(mut index: u32, base: u32) -> Real {
    if base < 2 {
        return 0.0;
    }
    let mut factor = 1.0;
    let mut result = 0.0;
    let base = base as Real;
    while index > 0 {
        factor /= base;
        result += factor * (index % base as u32) as Real;
        index /= base as u32;
    }
    result
}

#[cfg(test)]
#[path = "tests/temporal_jitter.rs"]
mod tests;
