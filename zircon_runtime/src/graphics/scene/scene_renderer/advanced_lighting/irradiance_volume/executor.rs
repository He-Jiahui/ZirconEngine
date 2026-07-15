use std::sync::Arc;

use crate::core::framework::render::select_irradiance_volume_for_view;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor, RenderPassExecutorRegistration,
};
use crate::render_graph::QueueLane;

use super::IRRADIANCE_VOLUME_BIND_EXECUTOR_ID;

pub(super) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![RenderPassExecutorRegistration::new_executor(
        IRRADIANCE_VOLUME_BIND_EXECUTOR_ID,
        Arc::new(IrradianceVolumeBindExecutor),
    )]
}

struct IrradianceVolumeBindExecutor;

impl RenderPassExecutor for IrradianceVolumeBindExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        if context.pass_name != IRRADIANCE_VOLUME_BIND_EXECUTOR_ID
            || context.executor_id.as_str() != IRRADIANCE_VOLUME_BIND_EXECUTOR_ID
            || context.declared_queue != QueueLane::Graphics
        {
            return Err("irradiance.volume_bind executor contract mismatch".to_string());
        }
        let gpu = context.require_gpu()?;
        let extract = gpu.frame_extract();
        let render_layers = extract.view.selected_camera_layers();
        let visible_world_positions = extract
            .geometry
            .meshes
            .iter()
            .filter(|mesh| mesh.render_layer_mask.intersects(render_layers))
            .map(|mesh| mesh.transform.translation)
            .collect::<Vec<_>>();
        let selected = select_irradiance_volume_for_view(
            &extract.lighting.advanced_lighting.irradiance_volumes,
            render_layers,
            &visible_world_positions,
        )
        .cloned();
        let streamer = gpu.streamer.ok_or_else(|| {
            "irradiance.volume_bind requires resource streamer context".to_string()
        })?;
        let selected = selected.and_then(|volume| {
            streamer
                .irradiance_volume_texture(volume.voxels)
                .map(|texture| (volume, texture))
        });
        let mesh_pipelines = gpu
            .mesh_pipelines
            .as_deref_mut()
            .ok_or_else(|| "irradiance.volume_bind requires mesh pipeline context".to_string())?;
        mesh_pipelines
            .irradiance_volume
            .prepare(gpu.queue, selected);
        Ok(())
    }
}
