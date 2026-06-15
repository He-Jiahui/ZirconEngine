use crate::core::framework::render::FrameHistoryHandle;
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
) -> (Option<&'a mut SceneFrameHistoryTextures>, bool) {
    let mut history_available = false;
    let mut history_textures = None;

    if runtime_features.temporal_history_enabled
        || runtime_features.ssao_enabled
        || runtime_features.hybrid_global_illumination_enabled
        || screen_space_reflection_history_enabled
        || hzb_history_enabled
        || exposure_history_enabled
    {
        if let Some(handle) = history_handle {
            let hzb_plan = HzbBuilder::new(render_size).build_plan();
            let history = history_targets.entry(handle).or_insert_with(|| {
                SceneFrameHistoryTextures::new(device, queue, size, render_size)
            });
            let history_matches_target = history.size == size
                && history.hzb_furthest_size == hzb_plan.hzb_size
                && history.hzb_furthest_mip_count == hzb_plan.mip_count
                && history.taa_scene_color_history_matches(TemporalHistoryKey::new(
                    size,
                    TAA_SCENE_COLOR_HISTORY_FORMAT,
                ));
            history_available = previous_history_available && history_matches_target;
            if !history_matches_target {
                *history = SceneFrameHistoryTextures::new(device, queue, size, render_size);
                history_available = false;
            }
            if !history_available {
                history.invalidate_taa_scene_color_history();
                if exposure_history_enabled {
                    history.invalidate_exposure_history(queue);
                }
            }
            history_textures = Some(history);
        }
    }

    (history_textures, history_available)
}
