use serde::{Deserialize, Serialize};

use super::{CorePipelineKind, RenderPhase};
use crate::core::framework::render::RenderMaterialAlphaMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// 材质队列的受限值域；绝对 authored 值与相对偏移归一化后才映射到绘制阶段。
pub struct RenderQueueValue(u16);

impl RenderQueueValue {
    pub const BACKGROUND: Self = Self(1_000);
    pub const GEOMETRY: Self = Self(2_000);
    pub const ALPHA_TEST: Self = Self(2_450);
    pub const GEOMETRY_LAST: Self = Self(2_500);
    pub const TRANSPARENT: Self = Self(3_000);
    pub const OVERLAY: Self = Self(4_000);
    pub const MAX: Self = Self(5_000);

    pub const fn new(raw: u16) -> Self {
        Self(if raw > Self::MAX.0 { Self::MAX.0 } else { raw })
    }

    pub const fn raw(self) -> u16 {
        self.0
    }

    pub fn from_alpha_mode(mode: &RenderMaterialAlphaMode) -> Self {
        match mode {
            RenderMaterialAlphaMode::Opaque => Self::GEOMETRY,
            RenderMaterialAlphaMode::Mask { .. } => Self::ALPHA_TEST,
            RenderMaterialAlphaMode::Blend => Self::TRANSPARENT,
        }
    }

    /// 零值沿用透明模式默认值，合法绝对值覆盖它，其余值按受限偏移解释。
    pub fn from_authored_queue(mode: &RenderMaterialAlphaMode, authored_queue: i32) -> Self {
        let default_queue = Self::from_alpha_mode(mode);
        if authored_queue == 0 {
            return default_queue;
        }

        if (i32::from(Self::BACKGROUND.0)..=i32::from(Self::MAX.0)).contains(&authored_queue) {
            return Self(clamp_queue_i32(authored_queue));
        }

        default_queue.with_material_offset_i32(authored_queue)
    }

    pub fn with_material_offset(self, offset: i16) -> Self {
        let clamped_offset = offset.clamp(-100, 100);
        Self(clamp_queue_i32(
            i32::from(self.0) + i32::from(clamped_offset),
        ))
    }

    pub fn with_material_offset_i32(self, offset: i32) -> Self {
        Self(clamp_queue_i32(self.0 as i32 + offset.clamp(-100, 100)))
    }

    pub fn phase(self, pipeline: CorePipelineKind) -> RenderPhase {
        if self.0 >= Self::OVERLAY.0 {
            return RenderPhase::Overlay;
        }

        match pipeline {
            CorePipelineKind::Core2d => self.phase_2d(),
            CorePipelineKind::Core3d => self.phase_3d(),
        }
    }

    fn phase_2d(self) -> RenderPhase {
        if self.0 >= Self::GEOMETRY_LAST.0 + 1 {
            RenderPhase::Transparent2d
        } else if self.0 >= Self::ALPHA_TEST.0 {
            RenderPhase::AlphaMask2d
        } else {
            RenderPhase::Opaque2d
        }
    }

    fn phase_3d(self) -> RenderPhase {
        if self.0 >= Self::GEOMETRY_LAST.0 + 1 {
            RenderPhase::Transparent3d
        } else if self.0 >= Self::ALPHA_TEST.0 {
            RenderPhase::AlphaMask3d
        } else {
            RenderPhase::Opaque3d
        }
    }
}

impl<'de> Deserialize<'de> for RenderQueueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::new(u16::deserialize(deserializer)?))
    }
}

impl Default for RenderQueueValue {
    fn default() -> Self {
        Self::GEOMETRY
    }
}

fn clamp_queue_i32(value: i32) -> u16 {
    value.clamp(0, i32::from(RenderQueueValue::MAX.0)) as u16
}

#[cfg(test)]
#[path = "tests/render_queue.rs"]
mod tests;
