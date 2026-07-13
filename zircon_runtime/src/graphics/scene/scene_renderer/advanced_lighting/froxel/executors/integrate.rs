use std::sync::Mutex;

use crate::core::framework::render::{
    FroxelGridParams, FroxelGridQuality, PostProcessGraphResourceNames,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassExecutionContext, RenderPassExecutor,
};
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::{
    resolved_volumetric_fog_settings, FroxelIntegratePipeline, FroxelIntegrateRequest,
    FroxelViewReconstruction, VOLUMETRIC_INTEGRATE_PIPELINE_LABEL,
    VOLUMETRIC_INTEGRATE_WORKGROUP_SIZE,
};
use super::{validate_compute_context, VOLUMETRIC_INTEGRATE_EXECUTOR_ID};

#[derive(Default)]
pub(super) struct VolumetricIntegrateExecutor {
    pipeline: Mutex<Option<FroxelIntegratePipeline>>,
}

impl RenderPassExecutor for VolumetricIntegrateExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_compute_context(context, VOLUMETRIC_INTEGRATE_EXECUTOR_ID)?;
        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let extract = gpu.frame_extract();
        let settings = resolved_volumetric_fog_settings(extract)?;
        let camera = extract.view.selected_effective_camera();
        let viewport_size = gpu.viewport_size();
        let grid = FroxelGridParams::for_quality(
            FroxelGridQuality::from_shader_quality(gpu.shader_quality()),
            camera.z_near,
            camera.z_far,
            settings.depth_distribution_exp,
        );
        let view = FroxelViewReconstruction::from_camera(&camera, viewport_size);
        let scattering = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let output = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let mut pipeline = self
            .pipeline
            .lock()
            .map_err(|_| "volumetric integrate pipeline cache lock poisoned".to_string())?;
        if pipeline.is_none() {
            *pipeline = Some(FroxelIntegratePipeline::new(gpu.device));
        }
        let dispatch = pipeline.as_ref().unwrap().encode(
            gpu.device,
            gpu.encoder,
            FroxelIntegrateRequest {
                grid,
                view,
                scattering_view: &scattering,
                output_view: &output,
            },
        )?;
        gpu.record_compute_dispatch_with_uploaded_bytes(
            pass_name,
            executor_id,
            VOLUMETRIC_INTEGRATE_PIPELINE_LABEL,
            [
                VOLUMETRIC_INTEGRATE_WORKGROUP_SIZE[0],
                VOLUMETRIC_INTEGRATE_WORKGROUP_SIZE[1],
                1,
            ],
            [dispatch[0], dispatch[1], 1],
            FroxelIntegratePipeline::UPLOADED_BYTES_PER_DISPATCH,
            vec![PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED.to_string()],
        );
        Ok(())
    }
}
