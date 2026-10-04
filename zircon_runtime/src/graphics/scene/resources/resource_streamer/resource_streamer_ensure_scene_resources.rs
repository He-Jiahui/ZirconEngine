use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetWritebackReport,
    RenderColorLookupTextureLayout, RenderFrameSubmissionTransaction, RenderImageDescriptor,
};
use crate::core::math::UVec2;
use crate::core::resource::ResourceId;
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::{
    GraphicsError, ViewportRenderFrame, ViewportTextureGraphImportPlan,
    ViewportTextureGraphImportStatus,
};

use super::super::OutputTargetFramePlan;
use super::ResourceStreamer;

impl ResourceStreamer {
    /// 在绘制前准备本帧资源并把上传登记到调用者的提交事务；mesh/model/material 等需求各自去重，诊断计数仍按实例累计。
    /// 材质准备完成后再收集可见性并安排 mip 变更，保证计划使用本帧可绘制资源。
    pub(crate) fn ensure_scene_resources(
        &mut self,
        backend: &RenderBackend,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        frame: &ViewportRenderFrame,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        self.last_material_count = 0;
        self.last_material_ready_count = 0;
        self.last_material_fallback_count = 0;
        self.last_material_validation_error_count = 0;
        self.last_material_diagnostic_count = 0;
        self.last_sprite_count = 0;
        self.last_sprite_ready_count = 0;
        self.last_sprite_texture_fallback_count = 0;
        self.last_post_process_lut_request_count = 0;
        self.last_post_process_lut_ready_count = 0;
        self.last_post_process_lut_fallback_count = 0;
        self.last_post_process_lut_2d_strip_ready_count = 0;
        self.last_post_process_lut_3d_request_count = 0;
        self.last_post_process_lut_unsupported_shape_count = 0;
        self.set_output_target_frame_plan(OutputTargetFramePlan::not_requested(
            frame.output_target(),
        ));
        let mut direct_mesh_readiness = HashMap::new();
        let mut ensured_models = HashSet::new();
        let mut ensured_materials = HashSet::new();
        for mesh in frame.meshes() {
            let direct_mesh_ready = if let Some(mesh_handle) = mesh.mesh {
                let mesh_id = mesh_handle.id();
                if let Some(ready) = direct_mesh_readiness.get(&mesh_id) {
                    *ready
                } else {
                    let ready = self.ensure_mesh(device, mesh_handle).is_ok();
                    direct_mesh_readiness.insert(mesh_id, ready);
                    ready
                }
            } else {
                false
            };
            if !direct_mesh_ready && ensured_models.insert(mesh.model.id()) {
                self.ensure_model(device, mesh.model)?;
            }
            if ensured_materials.insert(mesh.material.id()) {
                self.ensure_material_for_frame(
                    backend,
                    device,
                    texture_layout,
                    mesh.material,
                    submission_transaction,
                )?;
            }
            self.record_material_summary(mesh.material.id());
        }
        if let Some(lightmaps) = frame.environment().baked_lighting() {
            self.ensure_texture_for_frame(
                backend,
                texture_layout,
                lightmaps.atlas,
                submission_transaction,
            )?;
        }
        let mut ensured_cookie_textures = HashSet::new();
        for cookie in &frame.extract.lighting.advanced_lighting.cookies {
            if ensured_cookie_textures.insert(cookie.texture) {
                let _ = self.ensure_texture_for_frame(
                    backend,
                    texture_layout,
                    cookie.texture,
                    submission_transaction,
                );
            }
        }
        let mut ensured_irradiance_textures = HashSet::new();
        for volume in &frame.extract.lighting.advanced_lighting.irradiance_volumes {
            if ensured_irradiance_textures.insert(volume.voxels) {
                let _ = self.ensure_irradiance_volume_texture(
                    backend,
                    volume.voxels,
                    submission_transaction,
                );
            }
        }
        let mut sprite_texture_readiness = HashMap::new();
        for sprite in frame.sprites() {
            self.last_sprite_count += 1;
            let texture_id = sprite.image.id();
            let ready = if let Some(ready) = sprite_texture_readiness.get(&texture_id) {
                *ready
            } else {
                let ready = self
                    .ensure_sprite_texture_for_frame(
                        backend,
                        texture_layout,
                        texture_id,
                        submission_transaction,
                    )
                    .is_ok();
                sprite_texture_readiness.insert(texture_id, ready);
                ready
            };
            if ready {
                self.last_sprite_ready_count += 1;
            } else {
                self.last_sprite_texture_fallback_count += 1;
            }
        }
        if let Some(ui) = frame.ui.as_ref() {
            let dependencies = self.ui_texture_dependencies.prepare(ui);
            self.prepare_ui_textures_for_frame(
                backend,
                texture_layout,
                dependencies.as_ref(),
                submission_transaction,
            )?;
        } else {
            self.ui_texture_dependencies.clear();
            self.last_ui_texture_prepare_receipt = None;
        }
        if let Some(request) = effect_stack_lut_texture_request(frame) {
            self.last_post_process_lut_request_count += 1;
            if matches!(
                request.texture_layout,
                RenderColorLookupTextureLayout::Texture3d { .. }
            ) {
                self.last_post_process_lut_3d_request_count += 1;
            }
            self.record_effect_stack_lut_texture_readiness(
                backend,
                texture_layout,
                request,
                submission_transaction,
            );
        }
        self.ensure_output_target_texture(device, frame)?;
        self.resolve_output_target_frame_plan(frame);
        self.collect_texture_mip_streaming_visibility(frame);
        self.apply_texture_mip_streaming(
            backend,
            texture_layout,
            frame.texture_mip_bias(),
            submission_transaction,
        );
        Ok(())
    }

    fn record_material_summary(&mut self, material_id: crate::core::resource::ResourceId) {
        self.last_material_count += 1;
        if let Some(summary) = self.material_readiness_summary(&material_id) {
            if summary.is_ready {
                self.last_material_ready_count += 1;
            }
            if summary.uses_fallback {
                self.last_material_fallback_count += 1;
            }
            self.last_material_validation_error_count += summary.validation_error_count;
            self.last_material_diagnostic_count += summary.diagnostic_count;
        }
    }

    fn record_effect_stack_lut_texture_readiness(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        request: EffectStackLutTextureRequest,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) {
        let Some(texture_id) = request.texture_id else {
            self.last_post_process_lut_fallback_count += 1;
            return;
        };

        let Ok(asset_manager) = self.asset_manager() else {
            self.last_post_process_lut_fallback_count += 1;
            return;
        };
        let Ok(texture) = asset_manager.load_texture_asset_snapshot(texture_id) else {
            self.last_post_process_lut_fallback_count += 1;
            return;
        };
        let status = effect_stack_lut_texture_status(
            request.texture_layout,
            &texture.render_image_descriptor(),
        );

        match status {
            EffectStackLutTextureStatus::Ready2d | EffectStackLutTextureStatus::Ready2dStrip => {
                if self
                    .ensure_texture_for_frame(
                        backend,
                        texture_layout,
                        texture_id,
                        submission_transaction,
                    )
                    .is_ok()
                {
                    self.last_post_process_lut_ready_count += 1;
                    if status == EffectStackLutTextureStatus::Ready2dStrip {
                        self.last_post_process_lut_2d_strip_ready_count += 1;
                    }
                } else {
                    self.last_post_process_lut_fallback_count += 1;
                }
            }
            EffectStackLutTextureStatus::Ready3d => {
                if !matches!(
                    request.texture_layout,
                    RenderColorLookupTextureLayout::Texture3d { .. }
                ) {
                    self.last_post_process_lut_3d_request_count += 1;
                }
                if self
                    .ensure_post_process_lut_texture_snapshot(
                        backend,
                        texture_id,
                        texture,
                        submission_transaction,
                    )
                    .is_ok()
                {
                    self.last_post_process_lut_ready_count += 1;
                } else {
                    self.last_post_process_lut_fallback_count += 1;
                }
            }
            EffectStackLutTextureStatus::UnsupportedShape => {
                self.last_post_process_lut_unsupported_shape_count += 1;
                self.last_post_process_lut_fallback_count += 1;
            }
        }
    }

    fn resolve_output_target_frame_plan(&mut self, frame: &ViewportRenderFrame) {
        if !frame
            .camera_stack_output_policy()
            .owns_final_target_output()
        {
            let Some(size) = frame
                .output_target()
                .size()
                .filter(|_| frame.output_target().texture_handle().is_some())
            else {
                self.set_output_target_frame_plan(OutputTargetFramePlan::not_requested(
                    frame.output_target(),
                ));
                return;
            };
            self.set_output_target_frame_plan(OutputTargetFramePlan::new(
                frame.output_target(),
                RenderCameraTargetGraphImportReport::suppressed_by_camera_stack(size),
                RenderCameraTargetWritebackReport::suppressed_by_camera_stack(size),
                RenderCameraTargetWritebackReport::suppressed_by_camera_stack(size),
            ));
            return;
        }
        let target_format = frame
            .output_target()
            .texture_handle()
            .and_then(|texture| self.output_target_textures.get(&texture.id()))
            .map(|prepared| prepared.resource().descriptor().format.as_str());
        let plan = frame.output_target().graph_import_plan(target_format);
        let graph_import_report = output_target_graph_import_report(&plan);
        self.set_output_target_frame_plan(OutputTargetFramePlan::new(
            frame.output_target(),
            graph_import_report,
            output_target_writeback_plan(graph_import_report),
            direct_submission_output_target_writeback_plan(graph_import_report),
        ));
    }

    fn set_output_target_frame_plan(&mut self, plan: OutputTargetFramePlan) {
        self.last_output_target_graph_import_report = plan.graph_import_report();
        self.last_output_target_frame_plan = plan;
    }
}

fn direct_submission_output_target_writeback_plan(
    graph_import: RenderCameraTargetGraphImportReport,
) -> RenderCameraTargetWritebackReport {
    use crate::core::framework::render::RenderCameraTargetGraphImportStatus;

    match graph_import.status {
        RenderCameraTargetGraphImportStatus::ReadyForDirectImport
        | RenderCameraTargetGraphImportStatus::DirectImported => {
            RenderCameraTargetWritebackReport::ready_for_copy(graph_import.target_size)
        }
        _ => output_target_writeback_plan(graph_import),
    }
}

fn output_target_writeback_plan(
    graph_import: RenderCameraTargetGraphImportReport,
) -> RenderCameraTargetWritebackReport {
    use crate::core::framework::render::RenderCameraTargetGraphImportStatus;

    match graph_import.status {
        RenderCameraTargetGraphImportStatus::NotRequested => {
            RenderCameraTargetWritebackReport::not_requested(graph_import.target_kind)
        }
        RenderCameraTargetGraphImportStatus::PendingTargetDescriptor => {
            RenderCameraTargetWritebackReport::pending_target_descriptor(graph_import.target_size)
        }
        RenderCameraTargetGraphImportStatus::ReadyForDirectImport
        | RenderCameraTargetGraphImportStatus::DirectImported => {
            RenderCameraTargetWritebackReport::skipped_direct_import(graph_import.target_size)
        }
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback => {
            RenderCameraTargetWritebackReport::ready_for_conversion(graph_import.target_size)
        }
        RenderCameraTargetGraphImportStatus::SuppressedByCameraStack => {
            RenderCameraTargetWritebackReport::suppressed_by_camera_stack(graph_import.target_size)
        }
        RenderCameraTargetGraphImportStatus::BlockedFormatMismatch => {
            RenderCameraTargetWritebackReport::blocked_format_mismatch(graph_import.target_size)
        }
    }
}

fn output_target_graph_import_report(
    plan: &ViewportTextureGraphImportPlan,
) -> RenderCameraTargetGraphImportReport {
    let size = plan.size().unwrap_or_else(|| UVec2::new(0, 0));
    match plan.status() {
        ViewportTextureGraphImportStatus::NotRequested => {
            RenderCameraTargetGraphImportReport::not_requested(plan.target_kind())
        }
        ViewportTextureGraphImportStatus::PendingTargetDescriptor => {
            RenderCameraTargetGraphImportReport::pending_target_descriptor(size)
        }
        ViewportTextureGraphImportStatus::ReadyForDirectImport => {
            RenderCameraTargetGraphImportReport::ready_for_direct_import(size)
        }
        ViewportTextureGraphImportStatus::RequiresConversionWriteback => {
            RenderCameraTargetGraphImportReport::requires_conversion_writeback(size)
        }
        ViewportTextureGraphImportStatus::BlockedFormatMismatch => {
            RenderCameraTargetGraphImportReport::blocked_format_mismatch(size)
        }
        ViewportTextureGraphImportStatus::BlockedPreparedFormatMismatch => {
            RenderCameraTargetGraphImportReport::blocked_format_mismatch(size)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EffectStackLutTextureRequest {
    texture_id: Option<ResourceId>,
    texture_layout: RenderColorLookupTextureLayout,
}

fn effect_stack_lut_texture_request(
    frame: &ViewportRenderFrame,
) -> Option<EffectStackLutTextureRequest> {
    let settings = frame.post_process().effect_stack.color_lookup;
    settings.is_enabled().then(|| EffectStackLutTextureRequest {
        texture_id: settings.texture.map(|texture| texture.id()),
        texture_layout: settings.texture_layout,
    })
}

#[cfg(test)]
fn effect_stack_lut_texture_id(frame: &ViewportRenderFrame) -> Option<ResourceId> {
    effect_stack_lut_texture_request(frame).and_then(|request| request.texture_id)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectStackLutTextureStatus {
    Ready2d,
    Ready2dStrip,
    Ready3d,
    UnsupportedShape,
}

fn effect_stack_lut_texture_status(
    layout: RenderColorLookupTextureLayout,
    descriptor: &RenderImageDescriptor,
) -> EffectStackLutTextureStatus {
    if layout.matches_texture_3d(descriptor) {
        return EffectStackLutTextureStatus::Ready3d;
    }
    if layout.matches_texture_2d_strip(descriptor) {
        return EffectStackLutTextureStatus::Ready2dStrip;
    }
    if layout.accepts_current_post_process_binding(descriptor) {
        return EffectStackLutTextureStatus::Ready2d;
    }
    EffectStackLutTextureStatus::UnsupportedShape
}

#[cfg(test)]
#[path = "tests/resource_streamer_ensure_scene_resources.rs"]
mod tests;
