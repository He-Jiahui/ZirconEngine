use crate::core::framework::render::RenderCameraTargetWritebackReport;
use crate::core::math::UVec2;
use crate::core::resource::ResourceId;
use crate::graphics::debug_markers::{
    insert_marker, RENDERDOC_MARKER_TEXTURE_WRITEBACK,
    RENDERDOC_MARKER_TEXTURE_WRITEBACK_CONVERSION,
};
use crate::graphics::types::{
    GraphicsError, ViewportRenderFrame, ViewportTextureGraphImportStatus,
    ViewportTextureWritebackPlan, ViewportTextureWritebackStatus,
};
use std::sync::Arc;

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn execute_output_target_writeback(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ViewportRenderFrame,
        source_texture: &wgpu::Texture,
        source_view: &wgpu::TextureView,
        source_size: UVec2,
    ) -> Result<(), GraphicsError> {
        self.last_output_target_writeback_report =
            RenderCameraTargetWritebackReport::not_requested(frame.output_target().kind());
        let Some(texture_id) = output_target_texture_id(frame) else {
            return Ok(());
        };
        let prepared_resource = self
            .output_target_textures
            .get(&texture_id)
            .map(|prepared| Arc::clone(prepared.resource()))
            .ok_or_else(|| missing_prepared_output_target(texture_id))?;
        let plan =
            frame.texture_writeback_plan(Some(prepared_resource.descriptor().format.as_str()));
        self.last_output_target_writeback_report = output_target_writeback_report_for_plan(&plan);
        match plan.status() {
            ViewportTextureWritebackStatus::ReadyForSrgbCopy => {
                let extent =
                    output_target_writeback_extent(&plan, source_size, prepared_resource.size())?;
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-output-target-writeback-encoder"),
                });
                insert_marker(&mut encoder, RENDERDOC_MARKER_TEXTURE_WRITEBACK);
                encoder.copy_texture_to_texture(
                    source_texture.as_image_copy(),
                    prepared_resource.texture().as_image_copy(),
                    extent,
                );
                queue.submit([encoder.finish()]);
                self.last_output_target_writeback_report =
                    RenderCameraTargetWritebackReport::copied(prepared_resource.size());
            }
            ViewportTextureWritebackStatus::ReadyForConversion => {
                output_target_writeback_extent(&plan, source_size, prepared_resource.size())?;
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("zircon-output-target-linear-conversion-encoder"),
                });
                insert_marker(&mut encoder, RENDERDOC_MARKER_TEXTURE_WRITEBACK_CONVERSION);
                self.output_target_writeback_converter
                    .encode_linear_rgba_conversion(
                        device,
                        &mut encoder,
                        source_view,
                        prepared_resource.view(),
                    );
                queue.submit([encoder.finish()]);
                self.last_output_target_writeback_report =
                    RenderCameraTargetWritebackReport::converted(prepared_resource.size());
            }
            ViewportTextureWritebackStatus::NotRequested
            | ViewportTextureWritebackStatus::PendingTargetDescriptor
            | ViewportTextureWritebackStatus::BlockedFormatMismatch
            | ViewportTextureWritebackStatus::BlockedPreparedFormatMismatch => {}
        }
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
        let plan = frame
            .output_target()
            .graph_import_plan(Some(prepared_resource.descriptor().format.as_str()));
        if plan.status() == ViewportTextureGraphImportStatus::ReadyForDirectImport {
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
mod tests {
    use crate::core::framework::render::{
        RenderCameraTargetWritebackReport, RenderCameraTargetWritebackStatus,
    };
    use crate::core::math::UVec2;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use crate::graphics::types::{
        ViewportRenderOutputTarget, ViewportTextureWritebackStatus, FRAMEWORK_OUTPUT_FORMAT_LABEL,
        LINEAR_OUTPUT_FORMAT_LABEL,
    };

    use super::{
        output_target_writeback_extent, output_target_writeback_report_for_plan,
        should_execute_output_target_writeback, suppressed_output_target_writeback_report,
    };

    #[test]
    fn output_target_writeback_executes_ready_copy_and_conversion_plans() {
        let texture = texture_handle("tests/writeback/ready");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        };
        let conversion_target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: LINEAR_OUTPUT_FORMAT_LABEL,
        };
        let ready = target.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        let conversion = conversion_target.writeback_plan(Some(LINEAR_OUTPUT_FORMAT_LABEL));
        let blocked = target.writeback_plan(Some("rgba16float"));
        let non_texture = ViewportRenderOutputTarget::PrimarySurface
            .writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));

        assert_eq!(
            ready.status(),
            ViewportTextureWritebackStatus::ReadyForSrgbCopy
        );
        assert!(should_execute_output_target_writeback(&ready));
        assert_eq!(
            conversion.status(),
            ViewportTextureWritebackStatus::ReadyForConversion
        );
        assert!(should_execute_output_target_writeback(&conversion));
        assert!(!should_execute_output_target_writeback(&blocked));
        assert!(!should_execute_output_target_writeback(&non_texture));
    }

    #[test]
    fn output_target_writeback_report_maps_ready_and_blocked_plans() {
        let texture = texture_handle("tests/writeback/report");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        };
        let conversion_target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: LINEAR_OUTPUT_FORMAT_LABEL,
        };
        let ready = output_target_writeback_report_for_plan(
            &target.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)),
        );
        let conversion = output_target_writeback_report_for_plan(
            &conversion_target.writeback_plan(Some(LINEAR_OUTPUT_FORMAT_LABEL)),
        );
        let blocked =
            output_target_writeback_report_for_plan(&target.writeback_plan(Some("rgba16float")));

        assert_eq!(
            ready.status,
            RenderCameraTargetWritebackStatus::ReadyForCopy
        );
        assert_eq!(ready.target_size, UVec2::new(128, 72));
        assert_eq!(ready.copied_count, 0);
        assert!(!ready.debug_marker_emitted);
        assert_eq!(
            conversion.status,
            RenderCameraTargetWritebackStatus::ReadyForConversion
        );
        assert_eq!(conversion.target_size, UVec2::new(128, 72));
        assert_eq!(conversion.converted_count, 0);
        assert!(!conversion.debug_marker_emitted);
        assert_eq!(
            blocked.status,
            RenderCameraTargetWritebackStatus::BlockedFormatMismatch
        );
        assert_eq!(blocked.target_size, UVec2::new(128, 72));
        assert!(!blocked.debug_marker_emitted);
        let copied = RenderCameraTargetWritebackReport::copied(UVec2::new(128, 72));
        let converted = RenderCameraTargetWritebackReport::converted(UVec2::new(128, 72));
        assert!(copied.debug_marker_emitted);
        assert!(!copied.conversion_debug_marker_emitted);
        assert!(!converted.debug_marker_emitted);
        assert!(converted.conversion_debug_marker_emitted);
        assert_eq!(converted.converted_count, 1);
    }

    #[test]
    fn suppressed_output_target_writeback_report_is_texture_only() {
        let texture = ViewportRenderOutputTarget::Texture {
            handle: texture_handle("tests/writeback/suppressed"),
            size: UVec2::new(128, 72),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        };
        let texture_report = suppressed_output_target_writeback_report(texture);
        let primary_report =
            suppressed_output_target_writeback_report(ViewportRenderOutputTarget::PrimarySurface);

        assert_eq!(
            texture_report.status,
            RenderCameraTargetWritebackStatus::SuppressedByCameraStack
        );
        assert_eq!(texture_report.target_size, UVec2::new(128, 72));
        assert_eq!(
            primary_report.status,
            RenderCameraTargetWritebackStatus::NotRequested
        );
    }

    #[test]
    fn output_target_writeback_extent_accepts_matching_source_and_destination() {
        let plan = ready_plan(UVec2::new(64, 32));

        let extent =
            output_target_writeback_extent(&plan, UVec2::new(64, 32), UVec2::new(64, 32)).unwrap();

        assert_eq!(extent.width, 64);
        assert_eq!(extent.height, 32);
        assert_eq!(extent.depth_or_array_layers, 1);
    }

    #[test]
    fn output_target_writeback_extent_rejects_source_size_mismatch() {
        let plan = ready_plan(UVec2::new(64, 32));

        let error = output_target_writeback_extent(&plan, UVec2::new(63, 32), UVec2::new(64, 32))
            .unwrap_err();

        assert!(
            matches!(error, crate::graphics::types::GraphicsError::Asset(message) if message.contains("source extent"))
        );
    }

    #[test]
    fn output_target_writeback_extent_rejects_destination_size_mismatch() {
        let plan = ready_plan(UVec2::new(64, 32));

        let error = output_target_writeback_extent(&plan, UVec2::new(64, 32), UVec2::new(64, 31))
            .unwrap_err();

        assert!(
            matches!(error, crate::graphics::types::GraphicsError::Asset(message) if message.contains("destination extent"))
        );
    }

    fn ready_plan(size: UVec2) -> crate::graphics::types::ViewportTextureWritebackPlan {
        ViewportRenderOutputTarget::Texture {
            handle: texture_handle("tests/writeback/extent"),
            size,
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        }
        .writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL))
    }

    fn texture_handle(label: &str) -> ResourceHandle<TextureMarker> {
        ResourceHandle::new(ResourceId::from_stable_label(label))
    }
}
