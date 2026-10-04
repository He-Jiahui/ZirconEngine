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
