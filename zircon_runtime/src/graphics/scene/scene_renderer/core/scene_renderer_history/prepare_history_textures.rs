use crate::core::framework::render::{FrameHistoryHandle, FroxelGridQuality};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::temporal::taa::{
    TemporalHistoryKey, TAA_SCENE_COLOR_HISTORY_FORMAT,
};
use crate::graphics::visibility::HzbBuilder;

use super::super::super::history::SceneFrameHistoryTextures;
use super::super::super::post_process::SceneRuntimeFeatureFlags;

pub(crate) fn prepare_history_textures<'a>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    history_targets: &'a mut std::collections::HashMap<
        FrameHistoryHandle,
        SceneFrameHistoryTextures,
    >,
    history_handle: Option<FrameHistoryHandle>,
    previous_history_available: bool,
    size: UVec2,
    render_size: UVec2,
    runtime_features: SceneRuntimeFeatureFlags,
    screen_space_reflection_history_enabled: bool,
    hzb_history_enabled: bool,
    exposure_history_enabled: bool,
    volumetric_history_quality: Option<FroxelGridQuality>,
) -> (Option<&'a mut SceneFrameHistoryTextures>, bool, bool) {
    let mut history_available = false;
    let mut history_textures = None;
    let mut history_recreated = false;

    if runtime_features.temporal_history_enabled
        || runtime_features.ssao_enabled
        || runtime_features.hybrid_global_illumination_enabled
        || screen_space_reflection_history_enabled
        || hzb_history_enabled
        || exposure_history_enabled
        || volumetric_history_quality.is_some()
    {
        if let Some(handle) = history_handle {
            history_recreated = !history_targets.contains_key(&handle);
            let hzb_plan = HzbBuilder::new(render_size).build_plan();
            let history = history_targets.entry(handle).or_insert_with(|| {
                SceneFrameHistoryTextures::new_with_volumetric_history(
                    device,
                    queue,
                    size,
                    render_size,
                    volumetric_history_quality,
                )
            });
            let history_matches_target = history.size == size
                && history.hzb_furthest_size == hzb_plan.hzb_size
                && history.hzb_furthest_mip_count == hzb_plan.mip_count
                && history.taa_scene_color_history_matches(TemporalHistoryKey::new(
                    size,
                    TAA_SCENE_COLOR_HISTORY_FORMAT,
                ))
                && history.volumetric_history_quality() == volumetric_history_quality;
            history_available = history_is_available(
                previous_history_available,
                history_matches_target,
                history_recreated,
            );
            if !history_matches_target {
                *history = SceneFrameHistoryTextures::new_with_volumetric_history(
                    device,
                    queue,
                    size,
                    render_size,
                    volumetric_history_quality,
                );
                history_available = false;
                history_recreated = true;
            }
            if !history_available {
                history.invalidate_taa_scene_color_history();
                history.set_volumetric_history_valid(false);
                if exposure_history_enabled {
                    history.invalidate_exposure_history(queue);
                }
            }
            history_textures = Some(history);
        }
    }

    (history_textures, history_available, history_recreated)
}

fn history_is_available(
    previous_history_available: bool,
    history_matches_target: bool,
    history_recreated: bool,
) -> bool {
    previous_history_available && history_matches_target && !history_recreated
}

#[cfg(test)]
mod tests {
    use super::history_is_available;

    #[test]
    fn a_new_history_handle_cannot_reuse_a_previous_frame_validity_signal() {
        assert!(!history_is_available(true, true, true));
    }

    #[test]
    fn a_matching_retained_history_keeps_previous_frame_validity() {
        assert!(history_is_available(true, true, false));
    }

    #[test]
    fn a_rebuilt_or_mismatched_history_is_invalid() {
        assert!(!history_is_available(true, false, false));
        assert!(!history_is_available(false, true, false));
    }
}
