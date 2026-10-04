use crate::core::framework::render::{
    RenderCameraTargetWritebackReport, RenderCameraTargetWritebackStatus,
};
use crate::core::math::UVec2;
use crate::core::resource::ResourceId;
use crate::graphics::debug_markers::{
    insert_marker, RENDERDOC_MARKER_TEXTURE_WRITEBACK,
    RENDERDOC_MARKER_TEXTURE_WRITEBACK_CONVERSION,
};
use crate::graphics::types::{
    GraphicsError, ViewportRenderFrame, ViewportTextureWritebackPlan,
    ViewportTextureWritebackStatus,
};
use std::sync::Arc;

use super::super::OutputTargetFramePlan;
use super::ResourceStreamer;

impl ResourceStreamer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn encode_planned_output_target_writeback(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        plan: RenderCameraTargetWritebackReport,
        source_texture: &wgpu::Texture,
        source_view: &wgpu::TextureView,
        source_size: UVec2,
        destination_texture: &wgpu::Texture,
        destination_view: &wgpu::TextureView,
        destination_size: UVec2,
    ) -> Result<RenderCameraTargetWritebackReport, GraphicsError> {
        match plan.status {
            RenderCameraTargetWritebackStatus::ReadyForCopy => {
                let extent = output_target_writeback_extent_for_size(
                    plan.target_size,
                    source_size,
                    destination_size,
                )?;
                insert_marker(encoder, RENDERDOC_MARKER_TEXTURE_WRITEBACK);
                encoder.copy_texture_to_texture(
                    source_texture.as_image_copy(),
                    destination_texture.as_image_copy(),
                    extent,
                );
                Ok(RenderCameraTargetWritebackReport::copied(destination_size))
            }
            RenderCameraTargetWritebackStatus::ReadyForConversion => {
                output_target_writeback_extent_for_size(
                    plan.target_size,
                    source_size,
                    destination_size,
                )?;
                insert_marker(encoder, RENDERDOC_MARKER_TEXTURE_WRITEBACK_CONVERSION);
                self.output_target_writeback_converter
                    .encode_linear_rgba_conversion(device, encoder, source_view, destination_view);
                Ok(RenderCameraTargetWritebackReport::converted(
                    destination_size,
                ))
            }
            _ => Ok(plan),
        }
    }

    pub(crate) fn set_output_target_writeback_report(
        &mut self,
        report: RenderCameraTargetWritebackReport,
    ) {
        self.last_output_target_writeback_report = report;
    }

    /// Encodes output-target work into the caller-owned frame submission.
    ///
    /// The scene core retains sole submission ownership so this transfer
    /// remains ordered with final-color readback and does not add a queue
    /// submit after the frame packet.
    pub(crate) fn encode_output_target_writeback(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &ViewportRenderFrame,
        source_texture: &wgpu::Texture,
        source_view: &wgpu::TextureView,
        source_size: UVec2,
    ) -> Result<(), GraphicsError> {
        let plan = self.output_target_frame_plan();
        debug_assert_eq!(plan.target(), frame.output_target());
        self.encode_output_target_writeback_with_frame_plan(
            device,
            encoder,
            plan,
            source_texture,
            source_view,
            source_size,
        )
    }

    /// Encodes the resolved frame plan into the caller-owned command packet.
    ///
    /// The caller must pass the plan captured immediately after
    /// `ensure_scene_resources`. This keeps direct rendering on the same
    /// immutable target decision as the compiled graph path.
    pub(crate) fn encode_output_target_writeback_with_frame_plan(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        plan: OutputTargetFramePlan,
        source_texture: &wgpu::Texture,
        source_view: &wgpu::TextureView,
        source_size: UVec2,
    ) -> Result<(), GraphicsError> {
        self.last_output_target_writeback_report =
            RenderCameraTargetWritebackReport::not_requested(plan.target().kind());
        let Some(texture_id) = plan.target().texture_handle().map(|texture| texture.id()) else {
            return Ok(());
        };
        let prepared_resource = self
            .output_target_textures
            .get(&texture_id)
            .map(|prepared| Arc::clone(prepared.resource()))
            .ok_or_else(|| missing_prepared_output_target(texture_id))?;
        let report = plan.direct_submission_writeback_plan();
        self.last_output_target_writeback_report = report;
        self.last_output_target_writeback_report = self.encode_planned_output_target_writeback(
            device,
            encoder,
            report,
            source_texture,
            source_view,
            source_size,
            prepared_resource.texture(),
            prepared_resource.view(),
            prepared_resource.size(),
        )?;
        Ok(())
    }

    pub(crate) fn skip_output_target_writeback_after_direct_import(
        &mut self,
        frame: &ViewportRenderFrame,
    ) {
        let Some(texture_id) = output_target_texture_id(frame) else {
            self.last_output_target_writeback_report =
                RenderCameraTargetWritebackReport::not_requested(frame.output_target().kind());
            return;
        };
        let Some(prepared_resource) = self
            .output_target_textures
            .get(&texture_id)
            .map(|prepared| Arc::clone(prepared.resource()))
        else {
            self.last_output_target_writeback_report =
                RenderCameraTargetWritebackReport::not_requested(frame.output_target().kind());
            return;
        };
        let plan = self.output_target_frame_plan();
        if plan.graph_import_report().status
            == crate::core::framework::render::RenderCameraTargetGraphImportStatus::ReadyForDirectImport
        {
            self.last_output_target_writeback_report =
                RenderCameraTargetWritebackReport::skipped_direct_import(prepared_resource.size());
        } else {
            self.last_output_target_writeback_report =
                RenderCameraTargetWritebackReport::not_requested(frame.output_target().kind());
        }
    }

    pub(crate) fn suppress_output_target_writeback(&mut self, frame: &ViewportRenderFrame) {
        self.last_output_target_writeback_report =
            suppressed_output_target_writeback_report(frame.output_target());
    }
}

fn output_target_texture_id(frame: &ViewportRenderFrame) -> Option<ResourceId> {
    frame
        .output_target()
        .texture_handle()
        .map(|texture| texture.id())
}

fn suppressed_output_target_writeback_report(
    target: crate::graphics::types::ViewportRenderOutputTarget,
) -> RenderCameraTargetWritebackReport {
    match target.size() {
        Some(size) if target.texture_handle().is_some() => {
            RenderCameraTargetWritebackReport::suppressed_by_camera_stack(size)
        }
        _ => RenderCameraTargetWritebackReport::not_requested(target.kind()),
    }
}

fn should_execute_output_target_writeback(plan: &ViewportTextureWritebackPlan) -> bool {
    matches!(
        plan.status(),
        ViewportTextureWritebackStatus::ReadyForSrgbCopy
            | ViewportTextureWritebackStatus::ReadyForConversion
    )
}

fn output_target_writeback_report_for_plan(
    plan: &ViewportTextureWritebackPlan,
) -> RenderCameraTargetWritebackReport {
    let size = plan.size().unwrap_or_else(|| UVec2::new(0, 0));
    match plan.status() {
        ViewportTextureWritebackStatus::NotRequested => {
            RenderCameraTargetWritebackReport::not_requested(plan.target_kind())
        }
        ViewportTextureWritebackStatus::PendingTargetDescriptor => {
            RenderCameraTargetWritebackReport::pending_target_descriptor(size)
        }
        ViewportTextureWritebackStatus::ReadyForSrgbCopy => {
            RenderCameraTargetWritebackReport::ready_for_copy(size)
        }
        ViewportTextureWritebackStatus::ReadyForConversion => {
            RenderCameraTargetWritebackReport::ready_for_conversion(size)
        }
        ViewportTextureWritebackStatus::BlockedFormatMismatch => {
            RenderCameraTargetWritebackReport::blocked_format_mismatch(size)
        }
        ViewportTextureWritebackStatus::BlockedPreparedFormatMismatch => {
            RenderCameraTargetWritebackReport::blocked_format_mismatch(size)
        }
    }
}

fn output_target_writeback_extent(
    plan: &ViewportTextureWritebackPlan,
    source_size: UVec2,
    destination_size: UVec2,
) -> Result<wgpu::Extent3d, GraphicsError> {
    let Some(plan_size) = plan.size() else {
        return Err(GraphicsError::Asset(
            "output target writeback copy requires a resolved target extent".to_string(),
        ));
    };
    output_target_writeback_extent_for_size(plan_size, source_size, destination_size)
}

fn output_target_writeback_extent_for_size(
    plan_size: UVec2,
    source_size: UVec2,
    destination_size: UVec2,
) -> Result<wgpu::Extent3d, GraphicsError> {
    if plan_size != source_size {
        return Err(GraphicsError::Asset(format!(
            "output target writeback source extent {}x{} does not match target extent {}x{}",
            source_size.x, source_size.y, plan_size.x, plan_size.y
        )));
    }
    if plan_size != destination_size {
        return Err(GraphicsError::Asset(format!(
            "output target writeback destination extent {}x{} does not match target extent {}x{}",
            destination_size.x, destination_size.y, plan_size.x, plan_size.y
        )));
    }
    Ok(wgpu::Extent3d {
        width: plan_size.x,
        height: plan_size.y,
        depth_or_array_layers: 1,
    })
}

fn missing_prepared_output_target(texture_id: ResourceId) -> GraphicsError {
    GraphicsError::Asset(format!(
        "missing prepared output target texture {texture_id}"
    ))
}

#[cfg(test)]
#[path = "tests/resource_streamer_execute_output_target_writeback.rs"]
mod tests;
