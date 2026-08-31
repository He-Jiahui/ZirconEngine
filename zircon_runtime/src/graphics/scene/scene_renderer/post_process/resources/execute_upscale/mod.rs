use super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::core::framework::render::{RenderPipelinePhase, RenderViewFamilyPhaseTargets};
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::post_process::params::upscale_params::UpscaleParams;
use crate::graphics::scene::scene_renderer::post_process::resources::render_region::apply_local_render_region_to_pass;
use crate::render_graph::RenderGraphAttachmentOps;
use std::fmt;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UpscaleParamsBufferSlot {
    Primary,
    Secondary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) enum UpscaleExecutionError {
    InvalidPhase(RenderPipelinePhase),
    MissingInputTarget(RenderPipelinePhase),
}

impl fmt::Display for UpscaleExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPhase(phase) => {
                write!(formatter, "phase {phase:?} is not a spatial upscale phase")
            }
            Self::MissingInputTarget(phase) => {
                write!(
                    formatter,
                    "phase {phase:?} has no spatial upscale input target"
                )
            }
        }
    }
}

impl std::error::Error for UpscaleExecutionError {}

fn prepare_upscale(
    phase: RenderPipelinePhase,
    phase_targets: RenderViewFamilyPhaseTargets,
) -> Result<(UpscaleParamsBufferSlot, UpscaleParams), UpscaleExecutionError> {
    let buffer_slot = match phase {
        RenderPipelinePhase::PrimarySpatialUpscale => UpscaleParamsBufferSlot::Primary,
        RenderPipelinePhase::SecondarySpatialUpscale => UpscaleParamsBufferSlot::Secondary,
        _ => return Err(UpscaleExecutionError::InvalidPhase(phase)),
    };
    let input = phase_targets
        .input()
        .ok_or(UpscaleExecutionError::MissingInputTarget(phase))?
        .viewport()
        .physical_size;
    let output = phase_targets.output().viewport().physical_size;
    Ok((
        buffer_slot,
        UpscaleParams::from_logical_sizes(input, output),
    ))
}

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) fn execute_upscale(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        phase: RenderPipelinePhase,
        phase_targets: RenderViewFamilyPhaseTargets,
        source_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<WgpuBufferUploadBatch, UpscaleExecutionError> {
        let (buffer_slot, params) = prepare_upscale(phase, phase_targets)?;
        let params_buffer = match buffer_slot {
            UpscaleParamsBufferSlot::Primary => &self.primary_upscale_params_buffer,
            UpscaleParamsBufferSlot::Secondary => &self.secondary_upscale_params_buffer,
        };
        let params_uploads = WgpuBufferUploadBatch::from(WgpuBufferUpload::from_bytes(
            params_buffer.clone(),
            0,
            bytemuck::bytes_of(&params),
        ));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-upscale-bind-group"),
            layout: &self.upscale_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.upscale_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UpscalePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.upscale_pipeline);
        let output_size = phase_targets.output().viewport().physical_size;
        let region = crate::graphics::types::ViewportRenderRegion::full_target(output_size);
        if !apply_local_render_region_to_pass(&mut pass, region) {
            return Ok(params_uploads);
        }
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        Ok(params_uploads)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderPipelinePhase, RenderResolutionPolicy, RenderUpscalerKind, RenderViewFamilyPipeline,
    };
    use crate::core::math::UVec2;

    use super::{UpscaleExecutionError, UpscaleParamsBufferSlot, prepare_upscale};

    #[test]
    fn upscale_preparation_selects_distinct_phase_local_parameter_slots() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(1920, 1080),
            RenderResolutionPolicy::with_scales(0.5, 0.75),
            RenderUpscalerKind::Spatial,
        );

        let (primary_slot, primary_params) = prepare_upscale(
            RenderPipelinePhase::PrimarySpatialUpscale,
            pipeline
                .phase_targets(RenderPipelinePhase::PrimarySpatialUpscale)
                .expect("dual spatial pipeline must include the primary upscale phase"),
        )
        .expect("primary upscale preparation");
        let (secondary_slot, secondary_params) = prepare_upscale(
            RenderPipelinePhase::SecondarySpatialUpscale,
            pipeline
                .phase_targets(RenderPipelinePhase::SecondarySpatialUpscale)
                .expect("dual spatial pipeline must include the secondary upscale phase"),
        )
        .expect("secondary upscale preparation");

        assert_eq!(primary_slot, UpscaleParamsBufferSlot::Primary);
        assert_eq!(primary_params.input_output_size, [960, 540, 1440, 810]);
        assert_eq!(secondary_slot, UpscaleParamsBufferSlot::Secondary);
        assert_eq!(secondary_params.input_output_size, [1440, 810, 1920, 1080]);
    }

    #[test]
    fn upscale_preparation_rejects_non_spatial_phase_without_panicking() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(640, 360),
            RenderResolutionPolicy::default(),
            RenderUpscalerKind::Spatial,
        );
        let targets = pipeline
            .phase_targets(RenderPipelinePhase::SceneLinear)
            .expect("scene-linear phase targets");

        assert_eq!(
            prepare_upscale(RenderPipelinePhase::SceneLinear, targets),
            Err(UpscaleExecutionError::InvalidPhase(
                RenderPipelinePhase::SceneLinear
            ))
        );
    }

    #[test]
    fn upscale_preparation_rejects_missing_input_target_without_panicking() {
        let pipeline = RenderViewFamilyPipeline::resolve(
            UVec2::new(640, 360),
            RenderResolutionPolicy::default(),
            RenderUpscalerKind::Spatial,
        );
        let scene_targets = pipeline
            .phase_targets(RenderPipelinePhase::SceneLinear)
            .expect("scene-linear phase targets");

        assert_eq!(
            prepare_upscale(RenderPipelinePhase::PrimarySpatialUpscale, scene_targets,),
            Err(UpscaleExecutionError::MissingInputTarget(
                RenderPipelinePhase::PrimarySpatialUpscale
            ))
        );
    }
}
