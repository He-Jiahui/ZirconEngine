use std::sync::Mutex;

use crate::core::framework::render::{PostProcessGraphResourceNames, RenderCameraTarget};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassDeviceEpochCache, RenderPassExecutionContext, RenderPassExecutor,
    RenderPassGpuRecordingContext,
};
use crate::render_graph::{QueueLane, RenderGraphResourceAccessKind};

use super::{
    PlanarReflectionFilterPipeline, PLANAR_FILTER_EXECUTOR_ID, PLANAR_FILTER_PIPELINE_LABEL,
    PLANAR_FILTER_WORKGROUP_SIZE, PLANAR_REFLECTION_TEXTURE_RESOURCE,
};

#[derive(Default)]
pub(super) struct PlanarReflectionFilterExecutor {
    pipeline: Mutex<RenderPassDeviceEpochCache<(), PlanarReflectionFilterPipeline>>,
}

impl RenderPassExecutor for PlanarReflectionFilterExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_context(context)?;
        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "planar filter requires a materialized device epoch before pipeline recording"
                .to_string()
        })?;
        let target = match gpu.frame_extract().view.selected_camera_target() {
            RenderCameraTarget::Texture(target) => *target,
            _ => return Err("planar.filter requires a texture capture camera".to_string()),
        };
        let probe = gpu
            .frame_extract()
            .lighting
            .advanced_lighting
            .planar_probes
            .iter()
            .find(|probe| probe.capture_target() == Some(target))
            .cloned()
            .ok_or_else(|| {
                "planar.filter selected target is not owned by an extracted planar probe"
                    .to_string()
            })?;
        let resolution = probe.resolution.clamp(
            1,
            crate::graphics::scene::scene_renderer::environment::PLANAR_REFLECTION_TEXTURE_SIZE,
        );
        let mip_count = u32::BITS - resolution.leading_zeros();
        let source = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let output = {
            let mesh_pipelines = gpu.mesh_pipelines.as_deref_mut().ok_or_else(|| {
                "planar.filter requires mesh pipeline resources for its persistent mip chain"
                    .to_string()
            })?;
            mesh_pipelines.reflection_probes.planar_texture()
        };
        let report = {
            let mut native = gpu.native_context();
            let mut pipeline_cache = self
                .pipeline
                .lock()
                .map_err(|_| "planar filter pipeline cache lock poisoned".to_string())?;
            let pipeline = pipeline_cache.get_or_try_insert_with(device_epoch, (), || {
                Ok(PlanarReflectionFilterPipeline::new(
                    native.resource_factory(),
                ))
            })?;
            pipeline.encode(
                &mut native,
                &source,
                &output,
                wgpu::Extent3d {
                    width: resolution,
                    height: resolution,
                    depth_or_array_layers: 1,
                },
                mip_count,
            )
        }?;
        for (index, dispatch) in report.dispatches.into_iter().enumerate() {
            gpu.record_compute_dispatch_with_uploaded_bytes(
                pass_name.clone(),
                executor_id.clone(),
                format!("{PLANAR_FILTER_PIPELINE_LABEL}.mip{index}"),
                PLANAR_FILTER_WORKGROUP_SIZE,
                [dispatch[0], dispatch[1], 1],
                if index == 0 { report.uploaded_bytes } else { 0 },
                vec![PLANAR_REFLECTION_TEXTURE_RESOURCE.to_string()],
            );
        }
        Ok(())
    }
}

fn validate_context(context: &RenderPassExecutionContext<'_>) -> Result<(), String> {
    if context.pass_name != PLANAR_FILTER_EXECUTOR_ID
        || context.executor_id.as_str() != PLANAR_FILTER_EXECUTOR_ID
    {
        return Err(format!(
            "planar filter executor contract mismatch: pass `{}` executor `{}`",
            context.pass_name, context.executor_id
        ));
    }
    if context.declared_queue != QueueLane::AsyncCompute {
        return Err(format!(
            "planar.filter requires AsyncCompute declaration, got `{:?}`",
            context.declared_queue
        ));
    }
    Ok(())
}
