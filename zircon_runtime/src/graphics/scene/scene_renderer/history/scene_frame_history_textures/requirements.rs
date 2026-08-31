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
mod tests {
    use crate::core::framework::render::FroxelGridQuality;
    use crate::core::math::UVec2;

    use super::{SceneFrameHistoryRequirements, SceneHistoryDomain};

    #[test]
    fn disabled_features_require_no_physical_history() {
        assert!(SceneFrameHistoryRequirements::default().is_empty());
    }

    #[test]
    fn viewport_independent_history_does_not_inherit_resize_coupling() {
        let exposure = SceneFrameHistoryRequirements::new(false, false, false, false, true, None);
        let volumetric = SceneFrameHistoryRequirements::new(
            false,
            false,
            false,
            false,
            false,
            Some(FroxelGridQuality::High),
        );

        assert!(!exposure.uses_history_size());
        assert!(!exposure.uses_render_size());
        assert!(!volumetric.uses_history_size());
        assert!(!volumetric.uses_render_size());
    }

    #[test]
    fn image_domains_declare_their_exact_extent_dependency() {
        let taa = SceneFrameHistoryRequirements::new(true, false, false, false, false, None);
        let gi = SceneFrameHistoryRequirements::new(false, true, false, false, false, None);
        let ssr = SceneFrameHistoryRequirements::new(false, false, true, false, false, None);
        let hzb = SceneFrameHistoryRequirements::new(false, false, false, true, false, None);

        assert!(taa.uses_history_size());
        assert!(gi.uses_history_size());
        assert!(ssr.uses_history_size());
        assert!(hzb.uses_render_size());
    }

    #[test]
    fn allocation_change_contract_is_domain_local() {
        let current = SceneFrameHistoryRequirements::new(true, true, false, true, true, None);
        let next = SceneFrameHistoryRequirements::new(true, true, true, true, true, None);
        let changes = current.allocation_changes(
            UVec2::new(1920, 1080),
            UVec2::new(1280, 720),
            next,
            UVec2::new(1920, 1080),
            UVec2::new(1280, 720),
        );

        assert!(changes.changed(SceneHistoryDomain::ScreenSpaceReflection));
        for unchanged in [
            SceneHistoryDomain::TaaSceneColor,
            SceneHistoryDomain::HybridGlobalIllumination,
            SceneHistoryDomain::HzbFurthest,
            SceneHistoryDomain::Exposure,
            SceneHistoryDomain::VolumetricScattering,
        ] {
            assert!(!changes.changed(unchanged));
        }
    }

    #[test]
    fn extent_changes_only_rebuild_extent_dependent_domains() {
        let requirements = SceneFrameHistoryRequirements::new(
            true,
            true,
            true,
            true,
            true,
            Some(FroxelGridQuality::High),
        );
        let changes = requirements.allocation_changes(
            UVec2::new(1920, 1080),
            UVec2::new(1280, 720),
            requirements,
            UVec2::new(2560, 1440),
            UVec2::new(1600, 900),
        );

        for changed in [
            SceneHistoryDomain::TaaSceneColor,
            SceneHistoryDomain::HybridGlobalIllumination,
            SceneHistoryDomain::ScreenSpaceReflection,
            SceneHistoryDomain::HzbFurthest,
        ] {
            assert!(changes.changed(changed));
        }
        assert!(!changes.changed(SceneHistoryDomain::Exposure));
        assert!(!changes.changed(SceneHistoryDomain::VolumetricScattering));
    }

    #[test]
    fn stable_requirements_and_extents_produce_no_allocation_changes() {
        let requirements = SceneFrameHistoryRequirements::new(
            false,
            false,
            false,
            false,
            true,
            Some(FroxelGridQuality::Medium),
        );
        let changes = requirements.allocation_changes(
            UVec2::new(640, 360),
            UVec2::new(640, 360),
            requirements,
            UVec2::new(640, 360),
            UVec2::new(640, 360),
        );

        assert!(changes.is_empty());
    }
}
