use crate::core::framework::render::{
    RenderFrameSubmissionProducer, RenderFrameSubmissionTransaction, RenderPipelinePhase,
    SkyboxMode,
};
use crate::core::math::UVec2;
use crate::graphics::backend::{GpuPassTimer, RenderBackend, ViewportSurface};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::render_graph::RenderGraphAttachmentOps;
use crate::rhi::SubmissionTicket;

use super::super::super::super::resources::{OutputTargetFramePlan, ResourceStreamer};
use super::super::scene_renderer_core::SceneRendererCore;

const DIRECT_SCENE_CONTENT_GPU_PASS: &str = "direct_scene_content";
const DIRECT_LIGHT_COOKIE_ATLAS_GPU_PASS: &str = "direct_light_cookie_atlas";
const DIRECT_REALTIME_IBL_GPU_PASS: &str = "direct_realtime_ibl";
const DIRECT_GPU_SCENE_UPLOAD_GPU_PASS: &str = "direct_gpu_scene_upload";
const DIRECT_OUTPUT_TRANSFER_GPU_PASS: &str = "direct_output_transfer";
const DIRECT_OVERLAYS_GPU_PASS: &str = "direct_overlays";
const DIRECT_UI_GPU_PASS: &str = "direct_ui";

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_scene(
        &mut self,
        backend: &RenderBackend,
        streamer: &mut ResourceStreamer,
        frame: &ViewportRenderFrame,
        scene_color_view: &wgpu::TextureView,
        final_color: &wgpu::Texture,
        final_color_view: &wgpu::TextureView,
        final_color_size: UVec2,
        depth_view: &wgpu::TextureView,
        gpu_pass_timer: Option<&mut GpuPassTimer>,
        frame_generation: u64,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
        viewport_product_copy: Option<&zr_rhi_wgpu::WgpuUiExternalImageCopyTarget>,
        surface_frame: Option<(&ViewportSurface, &zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget)>,
        output_target_frame_plan: OutputTargetFramePlan,
    ) -> Result<SubmissionTicket, GraphicsError> {
        let mut environment_frame = self.begin_scene_environment_frame();
        environment_frame.core().render_scene_in_environment_frame(
            backend,
            streamer,
            frame,
            scene_color_view,
            final_color,
            final_color_view,
            final_color_size,
            depth_view,
            gpu_pass_timer,
            frame_generation,
            submission_transaction,
            viewport_product_copy,
            surface_frame,
            output_target_frame_plan,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_scene_in_environment_frame(
        &mut self,
        backend: &RenderBackend,
        streamer: &mut ResourceStreamer,
        frame: &ViewportRenderFrame,
        scene_color_view: &wgpu::TextureView,
        final_color: &wgpu::Texture,
        final_color_view: &wgpu::TextureView,
        final_color_size: UVec2,
        depth_view: &wgpu::TextureView,
        mut gpu_pass_timer: Option<&mut GpuPassTimer>,
        frame_generation: u64,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
        viewport_product_copy: Option<&zr_rhi_wgpu::WgpuUiExternalImageCopyTarget>,
        surface_frame: Option<(&ViewportSurface, &zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget)>,
        output_target_frame_plan: OutputTargetFramePlan,
    ) -> Result<SubmissionTicket, GraphicsError> {
        self.ensure_device_epoch(backend)?;
        let device = &backend.device;
        self.mesh_pipelines
            .collect_terminal_pipeline_submissions(|ticket| backend.submission_status(ticket).ok());
        self.mesh_pipelines.begin_submission_usage_recording();
        self.mesh_pipelines
            .begin_forward_receiver_binding_profile_frame();
        let realtime_ibl_prepared = matches!(
            frame.environment().skybox.mode,
            SkyboxMode::ProceduralGradient
        )
        .then_some(frame.environment().skybox.procedural)
        .filter(|sky| sky.intensity > 0.0)
        .map(|sky| self.realtime_ibl.prepare_frame(device, sky));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-encoder"),
        });
        self.mesh_pipelines.light_cookies.begin_profile_frame();
        let mut frame_texture_uploads = zr_rhi_wgpu::WgpuTextureUploadBatch::new();
        let mut frame_buffer_uploads = self.write_scene_uniform(
            backend,
            &mut encoder,
            streamer,
            frame,
            realtime_ibl_prepared.as_ref(),
            true,
            &mut frame_texture_uploads,
        )?;
        let scene_bind_group = self.frame_scene_bind_group().clone();
        let mut product_diagnostic_query_scope = backend
            .begin_product_diagnostic_query_scope(frame_generation, gpu_pass_timer.is_some(), false)
            .ok()
            .flatten();
        if let Some(scope) = product_diagnostic_query_scope.as_ref() {
            scope.attach_timers(gpu_pass_timer.as_deref_mut(), None);
        } else if let Some(timer) = gpu_pass_timer.as_deref_mut() {
            timer.defer_frame(frame_generation);
        }
        let light_cookies = &frame.extract.lighting.advanced_lighting.cookies;
        let light_cookie_scope = (!light_cookies.is_empty())
            .then(|| {
                gpu_pass_timer.as_deref_mut().and_then(|timer| {
                    timer.begin_pass(&mut encoder, DIRECT_LIGHT_COOKIE_ATLAS_GPU_PASS)
                })
            })
            .flatten();
        if !light_cookies.is_empty() {
            self.mesh_pipelines.light_cookies.rebuild(
                device,
                &mut encoder,
                streamer,
                light_cookies,
            );
        }
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), light_cookie_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        let realtime_ibl_gpu_timing_enabled = self.realtime_ibl.gpu_timestamps_supported();
        let realtime_ibl_scope = realtime_ibl_prepared
            .is_some()
            .then(|| {
                gpu_pass_timer
                    .as_deref_mut()
                    .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_REALTIME_IBL_GPU_PASS))
            })
            .flatten();
        let realtime_ibl_submission_result = realtime_ibl_prepared
            .as_ref()
            .map(|prepared| {
                self.realtime_ibl.record_prepared_frame(
                    device,
                    &mut encoder,
                    realtime_ibl_gpu_timing_enabled,
                    prepared,
                    &mut self.ibl_bake_pipeline_cache,
                )
            })
            .transpose()
            .map_err(GraphicsError::Asset);
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), realtime_ibl_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        let realtime_ibl_submission = match realtime_ibl_submission_result {
            Ok(submission) => submission.flatten(),
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(frame_generation);
                }
                return Err(error);
            }
        };
        let (shadow_frame_plan, mut shadow_atlas_prepared_upload) = if self
            .deferred_lighting_profile
            .uses_full_shadow_atlas_resources()
        {
            let static_caster_revision = streamer
                .with_ready_resource_revisions(|resource_revision| {
                    crate::graphics::scene::scene_renderer::shadow::
                        static_shadow_caster_revision_from_meshes_with_resource_revisions(
                            &frame.extract.geometry.meshes,
                            |resource| resource_revision(resource),
                        )
                })
                .flatten();
            let shadow_frame_plan = crate::graphics::scene::scene_renderer::shadow::
                build_shadow_frame_plan_with_static_caster_revision(
                    &mut self.shadow_atlas_allocator,
                    frame,
                    self.shadow_atlas_resources.config(),
                    static_caster_revision,
                );
            let mut prepared_upload = match self
                .shadow_atlas_resources
                .prepare_frame_upload(shadow_frame_plan.slots(), shadow_frame_plan.globals())
                .map_err(GraphicsError::Asset)
            {
                Ok(prepared_upload) => prepared_upload,
                Err(error) => {
                    if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                        timer.defer_frame(frame_generation);
                    }
                    if let Some(submission) = realtime_ibl_submission {
                        self.realtime_ibl.complete_submission(submission, false);
                    }
                    return Err(error);
                }
            };
            prepared_upload.append_to(&mut frame_buffer_uploads);
            (Some(shadow_frame_plan), Some(prepared_upload))
        } else {
            (None, None)
        };
        let uses_direct_lights = self.deferred_lighting_profile.uses_direct_lights();
        let gpu_scene_upload_scope = gpu_pass_timer
            .as_deref_mut()
            .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_GPU_SCENE_UPLOAD_GPU_PASS));
        let mut built_mesh_draws = match self.advanced_plugin_resources.build_mesh_draws(
            backend,
            &mut encoder,
            &self.material_texture_bind_group_layout,
            &mut self.gpu_scene,
            streamer,
            &mut self.mesh_pipelines,
            frame,
            false,
            uses_direct_lights,
            shadow_frame_plan.as_ref().map(|plan| plan.light_slots()),
        ) {
            Ok(draws) => draws,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(frame_generation);
                }
                if let Some(submission) = realtime_ibl_submission {
                    self.realtime_ibl.complete_submission(submission, false);
                }
                return Err(error);
            }
        };
        let mut gpu_scene_prepared_upload = built_mesh_draws.take_gpu_scene_prepared_upload();
        gpu_scene_prepared_upload.append_to(&self.gpu_scene, &mut frame_buffer_uploads);
        let material_pipeline_requirements = built_mesh_draws.take_material_pipeline_requirements();
        crate::graphics::scene::scene_renderer::mesh::coordinate_material_pipeline_publications(
            device,
            streamer,
            &mut self.mesh_pipelines,
            material_pipeline_requirements,
            frame
                .camera_stack_output_policy()
                .starts_viewport_submission(),
            frame
                .camera_stack_output_policy()
                .owns_viewport_submission(),
        );
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), gpu_scene_upload_scope)
        {
            timer.end_pass(&mut encoder, scope);
        }
        let mesh_draws = built_mesh_draws.into_draws();
        let gpu_scene_bind_group = self.gpu_scene.scene_bind_group().clone();
        let gpu_scene_bind_handle =
            crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshSceneDataBindHandle::new(
                &gpu_scene_bind_group,
            );
        let prepared_overlays = match self.overlay_renderer.prepare_buffers(
            device,
            &self.texture_bind_group_layout,
            streamer,
            frame,
            &mut frame_texture_uploads,
        ) {
            Ok(prepared_overlays) => prepared_overlays,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(frame_generation);
                }
                if let Some(submission) = realtime_ibl_submission {
                    self.realtime_ibl.complete_submission(submission, false);
                }
                return Err(error);
            }
        };
        let scene_content_scope = gpu_pass_timer
            .as_deref_mut()
            .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_SCENE_CONTENT_GPU_PASS));
        self.overlay_renderer.record_scene_content(
            &mut encoder,
            device,
            scene_color_view,
            depth_view,
            &scene_bind_group,
            &mesh_draws,
            Some(gpu_scene_bind_handle),
            &mut self.mesh_pipelines,
            streamer,
            frame,
            Some(&self.shadow_atlas_resources),
        );
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), scene_content_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        let output_transfer_scope = gpu_pass_timer
            .as_deref_mut()
            .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_OUTPUT_TRANSFER_GPU_PASS));
        let output_region = frame
            .render_region_for_phase(RenderPipelinePhase::OutputTransform)
            .filter(|region| {
                frame.view_family_pipeline().resolution().display_extent() == final_color_size
            })
            .unwrap_or_else(|| frame.render_region());
        self.post_process.execute_output_transfer(
            device,
            &mut encoder,
            scene_color_view,
            final_color_view,
            RenderGraphAttachmentOps::clear_store(),
            output_region,
        );
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), output_transfer_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        let overlays_scope = gpu_pass_timer
            .as_deref_mut()
            .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_OVERLAYS_GPU_PASS));
        self.overlay_renderer.record_overlays(
            &mut encoder,
            final_color_view,
            depth_view,
            &scene_bind_group,
            frame,
            &prepared_overlays,
            output_region,
        );
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), overlays_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        let mut screen_space_ui_prepared_upload = None;
        if let Some(screen_space_ui_renderer) = self.screen_space_ui_renderer.as_mut() {
            let ui_scope = gpu_pass_timer
                .as_deref_mut()
                .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_UI_GPU_PASS));
            let mut prepared_upload = match screen_space_ui_renderer.record(
                device,
                &mut encoder,
                final_color_view,
                frame,
                RenderGraphAttachmentOps::load_store(),
                Some(streamer),
            ) {
                Ok(prepared_upload) => prepared_upload,
                Err(error) => {
                    if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                        timer.defer_frame(frame_generation);
                    }
                    if let Some(submission) = realtime_ibl_submission {
                        self.realtime_ibl.complete_submission(submission, false);
                    }
                    return Err(error);
                }
            };
            if !prepared_upload.append_to(
                screen_space_ui_renderer,
                &mut frame_buffer_uploads,
                &mut frame_texture_uploads,
            ) {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(frame_generation);
                }
                if let Some(submission) = realtime_ibl_submission {
                    self.realtime_ibl.complete_submission(submission, false);
                }
                return Err(GraphicsError::Asset(
                    "direct screen-space UI could not attach its resource upload transaction"
                        .to_string(),
                ));
            }
            screen_space_ui_prepared_upload = Some(prepared_upload);
            if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), ui_scope) {
                timer.end_pass(&mut encoder, scope);
            }
        }
        let frame_resource_upload_submission = match backend.enqueue_copy_resource_upload_batch(
            zr_rhi_wgpu::WgpuResourceUploadBatch::from_batches(
                frame_buffer_uploads,
                frame_texture_uploads,
            ),
        ) {
            Ok(submission) => submission,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(frame_generation);
                }
                if let Some(submission) = realtime_ibl_submission {
                    self.realtime_ibl.complete_submission(submission, false);
                }
                return Err(error);
            }
        };
        if let Err(error) = backend.record_pre_scene_submission(
            submission_transaction,
            RenderFrameSubmissionProducer::FrameResourceUpload,
            frame_resource_upload_submission,
        ) {
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.defer_frame(frame_generation);
            }
            if let Some(submission) = realtime_ibl_submission {
                self.realtime_ibl.complete_submission(submission, false);
            }
            return Err(error);
        }
        let product_diagnostic_frame_opened =
            if realtime_ibl_submission.is_some() && self.realtime_ibl.gpu_timestamps_supported() {
                match backend.begin_product_diagnostic_readback_frame(frame_generation) {
                    Ok(()) => {
                        let admitted = realtime_ibl_submission.as_ref().is_some_and(|submission| {
                            self.realtime_ibl.request_product_gpu_timestamp_readback(
                                submission,
                                backend.render_device.timestamp_period_ns(),
                                backend,
                            )
                        });
                        if !admitted {
                            backend.abort_product_diagnostic_readback_frame();
                        }
                        admitted
                    }
                    Err(_) => false,
                }
            } else {
                false
            };
        if let Err(error) = streamer.encode_output_target_writeback_with_frame_plan(
            device,
            &mut encoder,
            output_target_frame_plan,
            final_color,
            final_color_view,
            final_color_size,
        ) {
            if product_diagnostic_frame_opened {
                backend.abort_product_diagnostic_readback_frame();
            }
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.defer_frame(frame_generation);
            }
            if let Some(submission) = realtime_ibl_submission {
                self.realtime_ibl.complete_submission(submission, false);
            }
            return Err(error);
        }
        if let Some(viewport_product_copy) = viewport_product_copy {
            viewport_product_copy.encode_copy(&mut encoder, final_color);
        }
        if let Some((surface, surface_target)) = surface_frame {
            if let Err(error) =
                surface.record_frame_target_blit(&mut encoder, final_color_view, surface_target)
            {
                if product_diagnostic_frame_opened {
                    backend.abort_product_diagnostic_readback_frame();
                }
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(frame_generation);
                }
                if let Some(submission) = realtime_ibl_submission {
                    self.realtime_ibl.complete_submission(submission, false);
                }
                return Err(error);
            }
        }
        let product_diagnostic_frame = if product_diagnostic_frame_opened {
            match backend.prepare_product_diagnostic_readback_frame(
                "product-diagnostic-readback",
                &mut encoder,
            ) {
                Ok(frame) => frame,
                Err(_) => {
                    backend.abort_product_diagnostic_readback_frame();
                    None
                }
            }
        } else {
            None
        };
        let product_diagnostic_query_frame =
            product_diagnostic_query_scope.take().and_then(|scope| {
                scope
                    .finish_and_prepare(&mut encoder, gpu_pass_timer.as_deref_mut(), None)
                    .ok()
                    .flatten()
            });
        let scene_submission = match backend
            .submit_graphics_command_buffers_with_frame_diagnostics_and_surface(
                vec![encoder.finish()],
                product_diagnostic_frame,
                product_diagnostic_query_frame,
                surface_frame.map(|(_, target)| target),
            ) {
            Ok(submission) => submission,
            Err(error) => {
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    timer.defer_frame(frame_generation);
                }
                if let Some(submission) = realtime_ibl_submission {
                    self.realtime_ibl.complete_submission(submission, false);
                }
                return Err(error);
            }
        };
        self.mesh_pipelines
            .bind_recorded_pipeline_usage_to_submission(scene_submission);
        self.commit_scene_environment_frame();
        self.mesh_pipelines
            .reflection_probes
            .commit_pending_uploads();
        if let Some(submission) = realtime_ibl_submission {
            self.realtime_ibl.complete_submission(submission, true);
        }
        if let Err(source) = submission_transaction.validate_scene_submission(scene_submission) {
            return Err(GraphicsError::FrameFailedAfterSceneSubmission {
                scene_submission,
                source: Box::new(source.into()),
            });
        }
        self.overlay_renderer.commit_pending_icon_uploads();
        let _shadow_atlas_upload_report = shadow_atlas_prepared_upload
            .take()
            .map(|prepared| prepared.commit(&mut self.shadow_atlas_resources));
        gpu_scene_prepared_upload.commit(&mut self.gpu_scene);
        if let Some(prepared_upload) = screen_space_ui_prepared_upload {
            let committed = self
                .screen_space_ui_renderer
                .as_mut()
                .is_some_and(|renderer| renderer.commit_prepared_upload(prepared_upload));
            debug_assert!(committed);
        }
        let _prev_transform_roll_report = self.gpu_scene.roll_prev_transforms_after_success();
        let _prev_skinned_palette_roll_report =
            self.gpu_scene.roll_prev_skinned_palettes_after_success();
        let _prev_skinned_source_roll_report =
            self.gpu_scene.roll_prev_skinned_gpu_sources_after_success();
        let _prev_morph_weights_roll_report =
            self.gpu_scene.roll_prev_morph_weights_after_success();
        self.mesh_pipelines
            .emit_forward_receiver_binding_profile_frame();
        self.mesh_pipelines.light_cookies.emit_profile_frame();
        Ok(scene_submission)
    }
}

#[cfg(test)]
#[path = "render_scene/tests/light_cookie_tests.rs"]
mod light_cookie_tests;

#[cfg(test)]
#[path = "tests/render_scene.rs"]
mod tests;
