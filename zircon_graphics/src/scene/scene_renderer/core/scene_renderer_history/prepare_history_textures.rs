use zircon_framework::render::FrameHistoryHandle;

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
    size: zircon_math::UVec2,
    runtime_features: SceneRuntimeFeatureFlags,
) -> (Option<&'a mut SceneFrameHistoryTextures>, bool) {
    let mut history_available = false;
    let mut history_textures = None;

    if runtime_features.history_resolve_enabled || runtime_features.ssao_enabled {
        if let Some(handle) = history_handle {
            if history_targets
                .get(&handle)
                .is_some_and(|history| history.size == size)
            {
                history_available = true;
            }
            let history = history_targets
                .entry(handle)
                .or_insert_with(|| SceneFrameHistoryTextures::new(device, queue, size));
            if history.size != size {
                *history = SceneFrameHistoryTextures::new(device, queue, size);
                history_available = false;
            }
            history_textures = Some(history);
        }
    }

    (history_textures, history_available)
}
