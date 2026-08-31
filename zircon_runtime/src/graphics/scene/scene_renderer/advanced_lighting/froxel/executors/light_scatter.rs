use std::sync::Mutex;

use crate::core::framework::render::{
    FroxelGridParams, FroxelGridQuality, PostProcessGraphResourceNames,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassDeviceEpochCache, RenderPassExecutionContext, RenderPassExecutor,
    RenderPassGpuRecordingContext,
};
use crate::graphics::scene::scene_renderer::history::SceneHistoryDomain;
use crate::render_graph::RenderGraphResourceAccessKind;

use super::super::{
    FroxelLightScatterPipeline, FroxelLightScatterRequest, FroxelViewReconstruction,
    GpuFroxelTemporalReprojection, VOLUMETRIC_LIGHT_SCATTER_PIPELINE_LABEL,
    VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE, volumetric_ambient_radiance,
};
use super::{VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID, validate_compute_context};

#[derive(Default)]
pub(super) struct VolumetricLightScatterExecutor {
    pipeline: Mutex<RenderPassDeviceEpochCache<(), FroxelLightScatterPipeline>>,
}

impl RenderPassExecutor for VolumetricLightScatterExecutor {
    fn execute(&self, context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
        validate_compute_context(context, VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID)?;
        let pass_name = context.pass_name.clone();
        let executor_id = context.executor_id.as_str().to_string();
        let gpu = context.require_gpu()?;
        let device_epoch = gpu.device_epoch().ok_or_else(|| {
            "volumetric light scatter requires a materialized device epoch before pipeline recording"
                .to_string()
        })?;
        let extract = gpu.frame_extract();
        let preview_lighting_enabled = gpu.post_process().preview.lighting_enabled;
        let settings = gpu.volumetric_fog();
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
        let light_count = u32::try_from(
            extract.lighting.directional_lights.len()
                + extract.lighting.point_lights.len()
                + extract.lighting.spot_lights.len()
                + extract.lighting.rect_lights.len(),
        )
        .map_err(|_| "volumetric light count exceeds u32".to_string())?;
        let media = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::VOLUMETRIC_MEDIA,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let history = gpu
            .optional_texture_view(
                PostProcessGraphResourceNames::HISTORY_PREVIOUS_VOLUMETRIC_SCATTERING,
                RenderGraphResourceAccessKind::Read,
            )?
            .cloned();
        let previous_camera = gpu.previous_motion_vector_camera().cloned();
        let temporal_enabled = settings.temporal && quality.supports_temporal();
        let temporal = GpuFroxelTemporalReprojection::new(
            &camera,
            previous_camera.as_ref(),
            viewport_size,
            grid,
            temporal_enabled,
            temporal_enabled && history.is_some() && previous_camera.is_some(),
        );
        let output = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING,
                RenderGraphResourceAccessKind::Write,
            )?
            .clone();
        let light_buffer = gpu
            .require_buffer(
                PostProcessGraphResourceNames::SCENE_LIGHT_DATA,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let light_grid_params = gpu.require_buffer_binding(
            PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_zbins = gpu.require_buffer_binding(
            PostProcessGraphResourceNames::LIGHT_ZBINS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_tile_masks = gpu.require_buffer_binding(
            PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let shadow_atlas_view = gpu
            .require_texture_view(
                PostProcessGraphResourceNames::SHADOW_ATLAS,
                RenderGraphResourceAccessKind::Read,
            )?
            .clone();
        let shadow_resources = gpu.shadow_atlas_resources.ok_or_else(|| {
            "volumetric light scatter requires scene shadow atlas resources".to_string()
        })?;
        let shadow_sampler = shadow_resources.compare_sampler().clone();
        let shadow_slots_buffer = shadow_resources.slot_buffer().clone();
        let shadow_globals_buffer = shadow_resources.globals_buffer().clone();
        let ambient_radiance =
            volumetric_ambient_radiance(&extract.lighting.ambient_lights, preview_lighting_enabled);
        let dispatch = {
            let mut native = gpu.native_context();
            let mut pipeline_cache = self
                .pipeline
                .lock()
                .map_err(|_| "volumetric light scatter pipeline cache lock poisoned".to_string())?;
            let pipeline = pipeline_cache.get_or_try_insert_with(device_epoch, (), || {
                Ok(FroxelLightScatterPipeline::new(native.resource_factory()))
            })?;
            pipeline.encode(
                &mut native,
                FroxelLightScatterRequest {
                    grid,
                    view,
                    phase_g: settings.phase_g,
                    ambient_radiance,
                    viewport_size: [viewport_size.x, viewport_size.y],
                    media_view: &media,
                    history_view: history.as_ref().unwrap_or(&media),
                    temporal,
                    light_buffer: &light_buffer,
                    light_count,
                    light_grid_params_buffer: light_grid_params,
                    light_zbins_buffer: light_zbins,
                    light_tile_masks_buffer: light_tile_masks,
                    shadow_atlas_view: &shadow_atlas_view,
                    shadow_sampler: &shadow_sampler,
                    shadow_slots_buffer: &shadow_slots_buffer,
                    shadow_globals_buffer: &shadow_globals_buffer,
                    output_view: &output,
                },
            )
        }?;
        gpu.record_compute_dispatch_with_uploaded_bytes(
            pass_name,
            executor_id,
            VOLUMETRIC_LIGHT_SCATTER_PIPELINE_LABEL,
            VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE,
            dispatch,
            FroxelLightScatterPipeline::UPLOADED_BYTES_PER_DISPATCH,
            vec![PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING.to_string()],
        );
        gpu.record_history_write(SceneHistoryDomain::VolumetricScattering);
        Ok(())
    }
}
