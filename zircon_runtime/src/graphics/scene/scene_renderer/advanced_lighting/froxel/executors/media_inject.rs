use std::sync::Mutex;

use crate::core::framework::render::{
    FroxelGridParams, FroxelGridQuality, PostProcessGraphResourceNames, RenderLayerSet,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassDeviceEpochCache, RenderPassExecutionContext, RenderPassExecutor,
    RenderPassGpuRecordingContext,
};
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::{
    FroxelMediaInjectPipeline, FroxelMediaInjectRequest, FroxelViewReconstruction,
    VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL, VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE,
};
use super::{VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID, validate_compute_context};

#[derive(Default)]
pub(super) struct VolumetricMediaInjectExecutor {
    pipeline: Mutex<RenderPassDeviceEpochCache<(), FroxelMediaInjectPipeline>>,
}

impl RenderPassExecutor for VolumetricMediaInjectExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_compute_context(context, VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID)?;
        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "volumetric media inject requires a materialized device epoch before pipeline recording"
                .to_string()
        })?;
        let extract = gpu.frame_extract();
        let advanced = &extract.lighting.advanced_lighting;
        let settings = gpu.volumetric_fog();
        let default_render_layers = RenderLayerSet::default();
        let render_layers = extract
            .view
            .selected_camera_descriptor()
            .map(|camera| &camera.culling_mask)
            .unwrap_or(&default_render_layers);
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
        let prepared = FroxelMediaInjectPipeline::prepare_for_layers(
            FroxelMediaInjectRequest {
                settings,
                grid,
                view,
                local_volumes: &advanced.fog_volumes,
                include_local_volumes,
            },
            render_layers,
        )?;
        let output = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::VOLUMETRIC_MEDIA,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let outcome = {
            let mut native = gpu.native_context();
            let mut pipeline_cache = self
                .pipeline
                .lock()
                .map_err(|_| "volumetric media inject pipeline cache lock poisoned".to_string())?;
            let pipeline = pipeline_cache.get_or_try_insert_with(device_epoch, (), || {
                Ok(FroxelMediaInjectPipeline::new(native.resource_factory()))
            })?;
            pipeline.encode_prepared(&mut native, &output, prepared)
        }?;
        gpu.record_compute_dispatch_with_uploaded_bytes(
            pass_name,
            executor_id,
            VOLUMETRIC_MEDIA_INJECT_PIPELINE_LABEL,
            VOLUMETRIC_MEDIA_INJECT_WORKGROUP_SIZE,
            outcome.dispatch,
            outcome.uploaded_bytes,
            vec![PostProcessGraphResourceNames::VOLUMETRIC_MEDIA.to_string()],
        );
        Ok(())
    }
}
