use crate::core::framework::render::FroxelGridQuality;
use crate::core::math::UVec2;

use super::super::SceneHistoryDomain;

const fn domain_bit(domain: SceneHistoryDomain) -> u8 {
    1 << domain as u8
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneHistoryAllocationChanges {
    changed_bits: u8,
}

impl SceneHistoryAllocationChanges {
    pub(crate) fn record(&mut self, domain: SceneHistoryDomain, changed: bool) {
        if changed {
            self.changed_bits |= domain_bit(domain);
        }
    }

    pub(crate) const fn changed(self, domain: SceneHistoryDomain) -> bool {
        self.changed_bits & domain_bit(domain) != 0
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.changed_bits == 0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// 按功能域声明持久历史需求；尺寸变化只重建依赖对应渲染尺寸的纹理。
/// prepare_history_textures 与图资源绑定必须消费同一份需求，避免读写历史错位。
pub(crate) struct SceneFrameHistoryRequirements {
    taa_scene_color: bool,
    hybrid_global_illumination: bool,
    screen_space_reflection: bool,
    hzb_furthest: bool,
    exposure: bool,
    volumetric_scattering: Option<FroxelGridQuality>,
}

impl SceneFrameHistoryRequirements {
    pub(crate) const fn new(
        taa_scene_color: bool,
        hybrid_global_illumination: bool,
        screen_space_reflection: bool,
        hzb_furthest: bool,
        exposure: bool,
        volumetric_scattering: Option<FroxelGridQuality>,
    ) -> Self {
        Self {
            taa_scene_color,
            hybrid_global_illumination,
            screen_space_reflection,
            hzb_furthest,
            exposure,
            volumetric_scattering,
        }
    }

    pub(crate) const fn is_empty(self) -> bool {
        !self.taa_scene_color
            && !self.hybrid_global_illumination
            && !self.screen_space_reflection
            && !self.hzb_furthest
            && !self.exposure
            && self.volumetric_scattering.is_none()
    }

    #[cfg(test)]
    pub(crate) const fn uses_history_size(self) -> bool {
        self.taa_scene_color || self.hybrid_global_illumination || self.screen_space_reflection
    }

    #[cfg(test)]
    pub(crate) const fn uses_render_size(self) -> bool {
        self.hzb_furthest
    }

    /// 把功能开关与分辨率变化映射到各历史域，供调用方仅失效受影响的时间数据。
    pub(crate) fn allocation_changes(
        self,
        current_size: UVec2,
        current_render_size: UVec2,
        next: Self,
        next_size: UVec2,
        next_render_size: UVec2,
    ) -> SceneHistoryAllocationChanges {
        let history_size_changed = current_size != next_size;
        let render_size_changed = current_render_size != next_render_size;
        let mut changes = SceneHistoryAllocationChanges::default();
        changes.record(
            SceneHistoryDomain::TaaSceneColor,
            self.taa_scene_color != next.taa_scene_color
                || (next.taa_scene_color && history_size_changed),
        );
        changes.record(
            SceneHistoryDomain::HybridGlobalIllumination,
            self.hybrid_global_illumination != next.hybrid_global_illumination
                || (next.hybrid_global_illumination && history_size_changed),
        );
        changes.record(
            SceneHistoryDomain::ScreenSpaceReflection,
            self.screen_space_reflection != next.screen_space_reflection
                || (next.screen_space_reflection && history_size_changed),
        );
        changes.record(
            SceneHistoryDomain::HzbFurthest,
            self.hzb_furthest != next.hzb_furthest || (next.hzb_furthest && render_size_changed),
        );
        changes.record(SceneHistoryDomain::Exposure, self.exposure != next.exposure);
        changes.record(
            SceneHistoryDomain::VolumetricScattering,
            self.volumetric_scattering != next.volumetric_scattering,
        );
        changes
    }

    pub(super) const fn taa_scene_color(self) -> bool {
        self.taa_scene_color
    }

    pub(super) const fn hybrid_global_illumination(self) -> bool {
        self.hybrid_global_illumination
    }

    pub(super) const fn screen_space_reflection(self) -> bool {
        self.screen_space_reflection
    }

    pub(super) const fn hzb_furthest(self) -> bool {
        self.hzb_furthest
    }

    pub(super) const fn exposure(self) -> bool {
        self.exposure
    }

    pub(crate) const fn volumetric_scattering(self) -> Option<FroxelGridQuality> {
        self.volumetric_scattering
    }
}

#[cfg(test)]
#[path = "tests/requirements.rs"]
mod tests;
