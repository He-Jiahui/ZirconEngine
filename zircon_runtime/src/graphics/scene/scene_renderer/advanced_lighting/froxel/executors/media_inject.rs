use std::sync::Mutex;

use crate::core::framework::render::{
    FroxelGridParams, FroxelGridQuality, PostProcessGraphResourceNames,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor,
};
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::{
    resolved_volumetric_fog_settings, FroxelMediaInjectPipeline, FroxelMediaInjectRequest,
    FroxelViewReconstruction, VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL,
    VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE,
};
use super::{validate_compute_context, VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID};

#[derive(Default)]
pub(super) struct VolumetricMediaInjectExecutor {
    pipeline: Mutex<Option<FroxelMediaInjectPipeline>>,
}

impl RenderPassExecutor for VolumetricMediaInjectExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_compute_context(context, VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID)?;
        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let extract = gpu.frame_extract();
        let advanced = &extract.lighting.advanced_lighting;
        let settings = resolved_volumetric_fog_settings(extract)?;
        let render_layers = extract
            .view
            .selected_camera_descriptor()
            .map(|camera| camera.culling_mask.clone())
            .unwrap_or_default();
        let local_volumes = advanced.fog_volumes_for_layers(&render_layers);
        let camera = extract.view.selected_effective_camera();
        let viewport_size = gpu.viewport_size();
        let quality = FroxelGridQuality::from_shader_quality(gpu.shader_quality());
        let grid = FroxelGridParams::for_quality(
            quality,
            camera.z_near,
            camera.z_far,
            settings.depth_distribution_exp,
        );
        let view = FroxelViewReconstruction::from_camera(&camera, viewport_size);
        let include_local_volumes = quality.supports_local_volumes();
        let uploaded_bytes =
            FroxelMediaInjectPipeline::uploaded_bytes(local_volumes.len(), include_local_volumes);
        let output = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::VOLUMETRIC_MEDIA,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let mut pipeline = self
            .pipeline
            .lock()
            .map_err(|_| "volumetric media inject pipeline cache lock poisoned".to_string())?;
        if pipeline.is_none() {
            *pipeline = Some(FroxelMediaInjectPipeline::new(gpu.device));
        }
        let dispatch = pipeline.as_ref().unwrap().encode(
            gpu.device,
            gpu.encoder,
            &output,
            FroxelMediaInjectRequest {
                settings,
                grid,
                view,
                local_volumes: &local_volumes,
                include_local_volumes,
            },
        )?;
        gpu.record_compute_dispatch_with_uploaded_bytes(
            pass_name,
            executor_id,
            VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL,
            VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE,
            dispatch,
            uploaded_bytes,
            vec![PostProcessGraphResourceNames::VOLUMETRIC_MEDIA.to_string()],
        );
        Ok(())
    }
}
