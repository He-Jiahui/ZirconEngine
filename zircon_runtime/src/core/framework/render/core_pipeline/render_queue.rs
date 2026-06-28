use serde::{Deserialize, Serialize};

use super::{CorePipelineKind, RenderPhase};
use crate::core::framework::render::RenderMaterialAlphaMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RenderQueueValue(pub u16);

impl RenderQueueValue {
    pub const BACKGROUND: Self = Self(1_000);
    pub const GEOMETRY: Self = Self(2_000);
    pub const ALPHA_TEST: Self = Self(2_450);
    pub const GEOMETRY_LAST: Self = Self(2_500);
    pub const TRANSPARENT: Self = Self(3_000);
    pub const OVERLAY: Self = Self(4_000);
    pub const MAX: Self = Self(5_000);

    pub const fn new(raw: u16) -> Self {
        Self(raw)
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

    pub fn from_authored_queue(mode: &RenderMaterialAlphaMode, authored_queue: i32) -> Self {
        let default_queue = Self::from_alpha_mode(mode);
        if authored_queue == 0 {
            return default_queue;
        }

        if (i32::from(Self::BACKGROUND.0)..=i32::from(Self::MAX.0)).contains(&authored_queue) {
            return Self(clamp_queue_i32(authored_queue));
        }

        default_queue.with_material_offset(authored_queue as i16)
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

impl Default for RenderQueueValue {
    fn default() -> Self {
        Self::GEOMETRY
    }
}

fn clamp_queue_i32(value: i32) -> u16 {
    value.clamp(0, i32::from(RenderQueueValue::MAX.0)) as u16
}

#[cfg(test)]
mod tests {
    use super::RenderQueueValue;
    use crate::core::framework::render::{CorePipelineKind, RenderMaterialAlphaMode, RenderPhase};

    #[test]
    fn render_queue_defaults_follow_alpha_mode() {
        assert_eq!(
            RenderQueueValue::from_alpha_mode(&RenderMaterialAlphaMode::Opaque),
            RenderQueueValue::GEOMETRY
        );
        assert_eq!(
            RenderQueueValue::from_alpha_mode(&RenderMaterialAlphaMode::Mask { cutoff: 0.5 }),
            RenderQueueValue::ALPHA_TEST
        );
        assert_eq!(
            RenderQueueValue::from_alpha_mode(&RenderMaterialAlphaMode::Blend),
            RenderQueueValue::TRANSPARENT
        );
    }

    #[test]
    fn render_queue_segments_map_to_core_pipeline_phases() {
        assert_eq!(
            RenderQueueValue::BACKGROUND.phase(CorePipelineKind::Core3d),
            RenderPhase::Opaque3d
        );
        assert_eq!(
            RenderQueueValue::GEOMETRY.phase(CorePipelineKind::Core2d),
            RenderPhase::Opaque2d
        );
        assert_eq!(
            RenderQueueValue::ALPHA_TEST.phase(CorePipelineKind::Core3d),
            RenderPhase::AlphaMask3d
        );
        assert_eq!(
            RenderQueueValue::GEOMETRY_LAST.phase(CorePipelineKind::Core2d),
            RenderPhase::AlphaMask2d
        );
        assert_eq!(
            RenderQueueValue::new(RenderQueueValue::GEOMETRY_LAST.raw() + 1)
                .phase(CorePipelineKind::Core3d),
            RenderPhase::Transparent3d
        );
        assert_eq!(
            RenderQueueValue::TRANSPARENT.phase(CorePipelineKind::Core2d),
            RenderPhase::Transparent2d
        );
        assert_eq!(
            RenderQueueValue::OVERLAY.phase(CorePipelineKind::Core3d),
            RenderPhase::Overlay
        );
    }

    #[test]
    fn authored_unity_queue_values_override_alpha_mode() {
        assert_eq!(
            RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 2_900),
            RenderQueueValue::new(2_900)
        );
        assert_eq!(
            RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 2_900)
                .phase(CorePipelineKind::Core3d),
            RenderPhase::Transparent3d
        );
        assert_eq!(
            RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Blend, 2_000)
                .phase(CorePipelineKind::Core2d),
            RenderPhase::Opaque2d
        );
    }

    #[test]
    fn authored_queue_offsets_are_clamped_to_material_window() {
        assert_eq!(
            RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Blend, -10),
            RenderQueueValue::new(2_990)
        );
        assert_eq!(
            RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, -500),
            RenderQueueValue::new(1_900)
        );
        assert_eq!(
            RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 500),
            RenderQueueValue::new(2_100)
        );
        assert_eq!(
            RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 0),
            RenderQueueValue::GEOMETRY
        );
    }
}
