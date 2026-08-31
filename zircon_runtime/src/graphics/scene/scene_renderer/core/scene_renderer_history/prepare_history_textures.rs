use crate::core::framework::render::{
    FrameHistoryHandle, FrameHistoryInvalidationReason, FroxelGridQuality, RenderFrameHistoryInput,
};
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;

use super::super::super::history::{
    SceneFrameHistoryRequirements, SceneFrameHistoryTextures, SceneHistoryAllocationChanges,
    SceneHistoryDomain, SceneHistoryFrameTransaction, SceneHistoryResetReason,
};
use super::super::super::post_process::SceneRuntimeFeatureFlags;

pub(crate) fn prepare_history_textures<'a>(
    backend: &RenderBackend,
    history_targets: &'a mut std::collections::HashMap<
        FrameHistoryHandle,
        SceneFrameHistoryTextures,
    >,
    history_input: RenderFrameHistoryInput,
    size: UVec2,
    render_size: UVec2,
    runtime_features: SceneRuntimeFeatureFlags,
    taa_history_enabled: bool,
    screen_space_reflection_history_enabled: bool,
    hzb_history_enabled: bool,
    exposure_history_enabled: bool,
    volumetric_history_quality: Option<FroxelGridQuality>,
) -> (
    Option<&'a mut SceneFrameHistoryTextures>,
    SceneHistoryFrameTransaction,
    bool,
    Option<wgpu::CommandBuffer>,
) {
    let mut history_frame = SceneHistoryFrameTransaction::unavailable();
    let mut history_textures = None;
    let mut taa_history_allocation_changed = false;
    let mut history_initialization_command_buffer = None;
    let mut allocation_changes = SceneHistoryAllocationChanges::default();
    let mut history_created = false;

    let requirements = SceneFrameHistoryRequirements::new(
        taa_history_enabled,
        runtime_features.hybrid_global_illumination_enabled,
        screen_space_reflection_history_enabled,
        hzb_history_enabled,
        exposure_history_enabled,
        volumetric_history_quality,
    );

    if requirements.is_empty() {
        taa_history_allocation_changed = history_input
            .current()
            .and_then(|handle| history_targets.remove(&handle))
            .is_some_and(|history| history.taa_scene_color_current_texture().is_some());
        return (None, history_frame, taa_history_allocation_changed, None);
    }

    if let Some(handle) = history_input.current() {
        let history = match history_targets.entry(handle) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let (changes, initialization_command_buffer) = entry
                    .get_mut()
                    .reconcile_with_requirements_and_initialization(
                        &backend.device,
                        size,
                        render_size,
                        requirements,
                    );
                allocation_changes = changes;
                taa_history_allocation_changed = changes.changed(SceneHistoryDomain::TaaSceneColor);
                history_initialization_command_buffer = initialization_command_buffer;
                entry.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                history_created = true;
                let (history, initialization_command_buffer) =
                    SceneFrameHistoryTextures::new_with_requirements_and_initialization(
                        &backend.device,
                        size,
                        render_size,
                        requirements,
                    );
                taa_history_allocation_changed =
                    history.taa_scene_color_current_texture().is_some();
                history_initialization_command_buffer = initialization_command_buffer;
                entry.insert(history)
            }
        };
        history_frame = history.begin_history_frame();
        if history_created {
            history_frame.invalidate_all(SceneHistoryResetReason::NeverProduced);
        } else {
            if let Some(reason) = spatial_history_reset_reason(history_input) {
                history_frame.invalidate_spatial(reason);
            }
            for domain in SceneHistoryDomain::ALL {
                if allocation_changes.changed(domain) {
                    history_frame.invalidate(domain, SceneHistoryResetReason::AllocationChanged);
                }
            }
        }
        invalidate_disabled_domains(
            &mut history_frame,
            runtime_features,
            taa_history_enabled,
            screen_space_reflection_history_enabled,
            hzb_history_enabled,
            exposure_history_enabled,
            volumetric_history_quality.is_some(),
        );
        if exposure_history_enabled
            && !history_created
            && !allocation_changes.changed(SceneHistoryDomain::Exposure)
            && !history_frame
                .availability()
                .is_available(SceneHistoryDomain::Exposure)
        {
            history.request_exposure_history_reset();
        }
        history_textures = Some(history);
    }

    (
        history_textures,
        history_frame,
        taa_history_allocation_changed,
        history_initialization_command_buffer,
    )
}

const fn spatial_history_reset_reason(
    history_input: RenderFrameHistoryInput,
) -> Option<SceneHistoryResetReason> {
    if history_input.previous_available() {
        return None;
    }
    Some(match history_input.invalidation_reason() {
        Some(FrameHistoryInvalidationReason::CameraCut) => SceneHistoryResetReason::CameraCut,
        Some(FrameHistoryInvalidationReason::FrameInputsChanged) => {
            SceneHistoryResetReason::StructuralCompatibilityChanged
        }
        _ => SceneHistoryResetReason::PreviousFrameUnavailable,
    })
}

#[allow(clippy::too_many_arguments)]
fn invalidate_disabled_domains(
    frame: &mut SceneHistoryFrameTransaction,
    runtime_features: SceneRuntimeFeatureFlags,
    taa_history_enabled: bool,
    screen_space_reflection_history_enabled: bool,
    hzb_history_enabled: bool,
    exposure_history_enabled: bool,
    volumetric_history_enabled: bool,
) {
    let enabled = [
        (SceneHistoryDomain::TaaSceneColor, taa_history_enabled),
        (
            SceneHistoryDomain::HybridGlobalIllumination,
            runtime_features.hybrid_global_illumination_enabled,
        ),
        (SceneHistoryDomain::AmbientOcclusion, false),
        (
            SceneHistoryDomain::ScreenSpaceReflection,
            screen_space_reflection_history_enabled,
        ),
        (SceneHistoryDomain::HzbFurthest, hzb_history_enabled),
        (SceneHistoryDomain::Exposure, exposure_history_enabled),
        (
            SceneHistoryDomain::VolumetricScattering,
            volumetric_history_enabled,
        ),
    ];
    for (domain, enabled) in enabled {
        if !enabled {
            frame.invalidate(domain, SceneHistoryResetReason::FeatureDisabled);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{FrameHistoryInvalidationReason, RenderFrameHistoryInput};

    use super::{SceneHistoryResetReason, spatial_history_reset_reason};

    #[test]
    fn recreated_history_returns_initialization_commands_for_the_scene_packet() {
        let source = include_str!("prepare_history_textures.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("new_with_requirements_and_initialization("));
        assert!(production.contains("history_initialization_command_buffer"));
        assert!(!production.contains("record_pre_scene_submission("));
        assert!(!production.contains("RenderFrameSubmissionProducer::HistoryInitialization"));
        assert!(!production.contains("RenderFrameSubmissionTransaction"));
    }

    #[test]
    fn preparation_uses_domain_transactions_instead_of_global_history_validity() {
        let source = include_str!("prepare_history_textures.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("SceneHistoryFrameTransaction"));
        assert!(production.contains("invalidate_spatial"));
        assert!(production.contains("SceneHistoryDomain::Exposure"));
        assert!(!production.contains("fn history_is_available"));
    }

    #[test]
    fn spatial_history_reset_preserves_the_shared_camera_cut_reason() {
        let camera_cut = RenderFrameHistoryInput::new(
            None,
            false,
            Some(FrameHistoryInvalidationReason::CameraCut),
        );
        let structural_change = RenderFrameHistoryInput::new(
            None,
            false,
            Some(FrameHistoryInvalidationReason::FrameInputsChanged),
        );
        let unavailable = RenderFrameHistoryInput::new(None, false, None);
        let available = RenderFrameHistoryInput::new(None, true, None);

        assert_eq!(
            spatial_history_reset_reason(camera_cut),
            Some(SceneHistoryResetReason::CameraCut)
        );
        assert_eq!(
            spatial_history_reset_reason(unavailable),
            Some(SceneHistoryResetReason::PreviousFrameUnavailable)
        );
        assert_eq!(
            spatial_history_reset_reason(structural_change),
            Some(SceneHistoryResetReason::StructuralCompatibilityChanged)
        );
        assert_eq!(spatial_history_reset_reason(available), None);
    }

    #[test]
    fn empty_requirements_release_persistent_physical_history() {
        let source = include_str!("prepare_history_textures.rs");

        assert!(source.contains("requirements.is_empty()"));
        assert!(source.contains("history_targets.remove(&handle)"));
    }

    #[test]
    fn spatial_ambient_occlusion_does_not_activate_shared_frame_history() {
        let source = include_str!("prepare_history_textures.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let requirements = production
            .split("SceneFrameHistoryRequirements::new(")
            .nth(1)
            .and_then(|tail| tail.split(");").next())
            .expect("compiled history requirements");

        assert!(!requirements.contains("ssao_enabled"));
        assert!(production.contains("(SceneHistoryDomain::AmbientOcclusion, false)"));
    }

    #[test]
    fn physical_history_allocation_is_driven_by_compiled_requirements() {
        let source = include_str!("prepare_history_textures.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("SceneFrameHistoryRequirements::new("));
        assert!(production.contains("new_with_requirements_and_initialization("));
        assert!(!production.contains("new_with_volumetric_history_and_submission("));
    }

    #[test]
    fn occupied_history_is_reconciled_without_whole_aggregate_replacement() {
        let source = include_str!("prepare_history_textures.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        let occupied = production
            .split("std::collections::hash_map::Entry::Occupied")
            .nth(1)
            .and_then(|tail| {
                tail.split("std::collections::hash_map::Entry::Vacant")
                    .next()
            })
            .expect("occupied history branch");

        assert!(occupied.contains("reconcile_with_requirements_and_initialization("));
        assert!(!occupied.contains("*history = replacement"));
        assert!(production.contains("allocation_changes.changed(domain)"));
        assert!(production.contains("SceneHistoryResetReason::AllocationChanged"));
    }
}
