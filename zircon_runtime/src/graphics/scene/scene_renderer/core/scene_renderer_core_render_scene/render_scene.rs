use crate::core::framework::render::SkyboxMode;
use crate::graphics::backend::GpuPassTimer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::super::resources::ResourceStreamer;
use super::super::scene_renderer_core::SceneRendererCore;

const DIRECT_SCENE_CONTENT_GPU_PASS: &str = "direct_scene_content";
const DIRECT_REALTIME_IBL_GPU_PASS: &str = "direct_realtime_ibl";
const DIRECT_GPU_SCENE_UPLOAD_GPU_PASS: &str = "direct_gpu_scene_upload";
const DIRECT_OUTPUT_TRANSFER_GPU_PASS: &str = "direct_output_transfer";
const DIRECT_OVERLAYS_GPU_PASS: &str = "direct_overlays";
const DIRECT_UI_GPU_PASS: &str = "direct_ui";

impl SceneRendererCore {
    pub(crate) fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        scene_color_view: &wgpu::TextureView,
        final_color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        mut gpu_pass_timer: Option<&mut GpuPassTimer>,
        frame_generation: u64,
    ) -> Result<(), GraphicsError> {
        let realtime_ibl_prepared = matches!(
            frame.environment().skybox.mode,
            SkyboxMode::ProceduralGradient
        )
        .then_some(frame.environment().skybox.procedural)
        .filter(|sky| sky.intensity > 0.0)
        .map(|sky| self.realtime_ibl.prepare_frame(device, sky));
        self.write_scene_uniform(
            device,
            queue,
            streamer,
            frame,
            realtime_ibl_prepared.as_ref(),
            true,
        )?;
        self.readback_frame_index = self.readback_frame_index.wrapping_add(1);
        let readback_frame_index = self.readback_frame_index;
        let readback_ready = self
            .readback_queue
            .prepare_frame(device, readback_frame_index)
            .is_ok();
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-encoder"),
        });
        if readback_ready {
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                timer.begin_frame(frame_generation);
            }
        } else if let Some(timer) = gpu_pass_timer.as_deref_mut() {
            let _ = timer.defer_frame(frame_generation);
        }
        let realtime_ibl_gpu_timing_enabled = readback_ready && gpu_pass_timer.is_some();
        let realtime_ibl_scope = (readback_ready && realtime_ibl_prepared.is_some())
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
                if readback_ready {
                    self.readback_queue.abort_frame(readback_frame_index);
                    if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                        let _ = timer.defer_frame(frame_generation);
                    }
                }
                return Err(error);
            }
        };
        let shadow_frame_plan = if self
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
            if let Err(error) = self
                .shadow_atlas_resources
                .upload_frame(
                    queue,
                    shadow_frame_plan.slots(),
                    shadow_frame_plan.globals(),
                )
                .map_err(GraphicsError::Asset)
            {
                if readback_ready {
                    self.readback_queue.abort_frame(readback_frame_index);
                    if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                        let _ = timer.defer_frame(frame_generation);
                    }
                }
                return Err(error);
            }
            Some(shadow_frame_plan)
        } else {
            None
        };
        let uses_direct_lights = self.deferred_lighting_profile.uses_direct_lights();
        let gpu_scene_upload_scope = readback_ready
            .then(|| {
                gpu_pass_timer.as_deref_mut().and_then(|timer| {
                    timer.begin_pass(&mut encoder, DIRECT_GPU_SCENE_UPLOAD_GPU_PASS)
                })
            })
            .flatten();
        let built_mesh_draws = self.advanced_plugin_resources.build_mesh_draws(
            device,
            queue,
            &mut encoder,
            &self.material_texture_bind_group_layout,
            &mut self.gpu_scene,
            streamer,
            frame,
            false,
            uses_direct_lights,
            shadow_frame_plan.as_ref().map(|plan| plan.light_slots()),
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
            queue,
            &self.texture_bind_group_layout,
            streamer,
            frame,
        ) {
            Ok(prepared_overlays) => prepared_overlays,
            Err(error) => {
                if readback_ready {
                    self.readback_queue.abort_frame(readback_frame_index);
                    if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                        let _ = timer.defer_frame(frame_generation);
                    }
                }
                return Err(error);
            }
        };
        let scene_content_scope = readback_ready
            .then(|| {
                gpu_pass_timer
                    .as_deref_mut()
                    .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_SCENE_CONTENT_GPU_PASS))
            })
            .flatten();
        self.overlay_renderer.record_scene_content(
            &mut encoder,
            device,
            scene_color_view,
            depth_view,
            &self.scene_bind_group,
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
        let output_transfer_scope = readback_ready
            .then(|| {
                gpu_pass_timer.as_deref_mut().and_then(|timer| {
                    timer.begin_pass(&mut encoder, DIRECT_OUTPUT_TRANSFER_GPU_PASS)
                })
            })
            .flatten();
        self.post_process.execute_output_transfer(
            device,
            &mut encoder,
            scene_color_view,
            final_color_view,
            RenderGraphAttachmentOps::clear_store(),
            frame.render_region(),
        );
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), output_transfer_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        let overlays_scope = readback_ready
            .then(|| {
                gpu_pass_timer
                    .as_deref_mut()
                    .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_OVERLAYS_GPU_PASS))
            })
            .flatten();
        self.overlay_renderer.record_overlays(
            &mut encoder,
            final_color_view,
            depth_view,
            &self.scene_bind_group,
            frame,
            &prepared_overlays,
            frame.render_region(),
        );
        if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), overlays_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        if let Some(screen_space_ui_renderer) = self.screen_space_ui_renderer.as_mut() {
            let ui_scope = readback_ready
                .then(|| {
                    gpu_pass_timer
                        .as_deref_mut()
                        .and_then(|timer| timer.begin_pass(&mut encoder, DIRECT_UI_GPU_PASS))
                })
                .flatten();
            if let Err(error) = screen_space_ui_renderer.record(
                device,
                queue,
                &mut encoder,
                final_color_view,
                frame,
                RenderGraphAttachmentOps::load_store(),
                Some(streamer),
            ) {
                if readback_ready {
                    self.readback_queue.abort_frame(readback_frame_index);
                    if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                        let _ = timer.defer_frame(frame_generation);
                    }
                }
                return Err(error);
            }
            if let (Some(timer), Some(scope)) = (gpu_pass_timer.as_deref_mut(), ui_scope) {
                timer.end_pass(&mut encoder, scope);
            }
        }
        if readback_ready {
            if let Some(submission) = realtime_ibl_submission.as_ref() {
                self.realtime_ibl.request_gpu_timestamp_readback(
                    submission,
                    queue.get_timestamp_period(),
                    &mut self.readback_queue,
                );
            }
            if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                let _ = timer.resolve_and_request(&mut encoder, &mut self.readback_queue);
            }
            if let Err(error) = self
                .readback_queue
                .encode_copies(&mut encoder, readback_frame_index)
            {
                self.readback_queue.abort_frame(readback_frame_index);
                if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                    let _ = timer.defer_frame(frame_generation);
                }
                return Err(GraphicsError::BufferMap(error.to_string()));
            }
        }
        queue.submit([encoder.finish()]);
        let readback_map_error = if readback_ready {
            match self.readback_queue.begin_map(readback_frame_index) {
                Ok(()) => None,
                Err(error) => {
                    self.readback_queue.abort_frame(readback_frame_index);
                    if let Some(timer) = gpu_pass_timer.as_deref_mut() {
                        let _ = timer.defer_frame(frame_generation);
                    }
                    Some(error)
                }
            }
        } else {
            None
        };
        if let Some(submission) = realtime_ibl_submission {
            self.realtime_ibl.complete_submission(submission, true);
        }
        let _prev_transform_roll_report = self.gpu_scene.roll_prev_transforms_after_success();
        let _prev_skinned_palette_roll_report =
            self.gpu_scene.roll_prev_skinned_palettes_after_success();
        let _prev_skinned_source_roll_report =
            self.gpu_scene.roll_prev_skinned_gpu_sources_after_success();
        let _prev_morph_weights_roll_report =
            self.gpu_scene.roll_prev_morph_weights_after_success();
        if let Some(error) = readback_map_error {
            return Err(GraphicsError::BufferMap(error.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("render_scene.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("render scene source should retain a test-module boundary")
    }

    #[test]
    fn environment_only_direct_render_skips_shadow_frame_work_before_resource_scanning() {
        let source = production_source();
        let profile_gate = source
            .find(".uses_full_shadow_atlas_resources()")
            .expect("direct render should gate shadow-frame work by its profile");
        let static_caster_scan = source
            .find("static_shadow_caster_revision_from_meshes_with_resource_revisions")
            .expect("full-scene path should retain static caster revision tracking");

        assert!(
            profile_gate < static_caster_scan,
            "EnvironmentOnly must skip resource revision scans before they can iterate mesh assets"
        );
        assert!(
            source.contains("shadow_frame_plan.as_ref()"),
            "only a full-scene shadow plan may supply light-slot assignments to mesh draws"
        );
    }

    #[test]
    fn environment_only_direct_render_forwards_its_direct_light_policy_to_mesh_preparation() {
        let source = production_source();
        let profile_policy = source
            .find("let uses_direct_lights = self.deferred_lighting_profile.uses_direct_lights();")
            .expect("direct rendering must derive the light-buffer policy from its profile");
        let mesh_preparation = source
            .find("self.advanced_plugin_resources.build_mesh_draws(")
            .expect("direct rendering must prepare mesh draws");
        let preparation_call = &source[mesh_preparation..];

        assert!(
            profile_policy < mesh_preparation,
            "the profile policy must be decided before mesh preparation begins"
        );
        assert!(
            preparation_call.contains("uses_direct_lights,"),
            "the direct renderer must forward the profile policy to mesh preparation"
        );
    }

    #[test]
    fn direct_render_records_its_real_encoder_stages_with_the_shared_gpu_timer() {
        let source = production_source();

        assert!(source.contains("gpu_pass_timer: Option<&mut GpuPassTimer>"));
        assert!(source.contains("timer.begin_frame(frame_generation)"));
        assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_REALTIME_IBL_GPU_PASS)"));
        assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_GPU_SCENE_UPLOAD_GPU_PASS)"));
        assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_SCENE_CONTENT_GPU_PASS)"));
        assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_OUTPUT_TRANSFER_GPU_PASS)"));
        assert!(source.contains("timer.begin_pass(&mut encoder, DIRECT_OVERLAYS_GPU_PASS)"));
        assert!(source.contains("if let Some(screen_space_ui_renderer)"));
        assert!(
            source.contains("timer.resolve_and_request(&mut encoder, &mut self.readback_queue)")
        );

        let prepare_frame = source
            .find(".prepare_frame(device, readback_frame_index)")
            .expect("direct rendering must admit the shared readback frame");
        let begin_frame = source
            .find("timer.begin_frame(frame_generation)")
            .expect("direct timing must begin after readback admission");
        let scene_content = source
            .find("timer.begin_pass(&mut encoder, DIRECT_SCENE_CONTENT_GPU_PASS)")
            .expect("direct scene content must have a named timestamp scope");
        let realtime_ibl = source
            .find("timer.begin_pass(&mut encoder, DIRECT_REALTIME_IBL_GPU_PASS)")
            .expect("direct realtime IBL must have a named timestamp scope");
        let gpu_scene_upload = source
            .find("timer.begin_pass(&mut encoder, DIRECT_GPU_SCENE_UPLOAD_GPU_PASS)")
            .expect("direct GPU scene upload must have a named timestamp scope");
        let resolve = source
            .find("timer.resolve_and_request(&mut encoder, &mut self.readback_queue)")
            .expect("direct timing must resolve through the shared readback queue");
        let encode_copies = source
            .find(".encode_copies(&mut encoder, readback_frame_index)")
            .expect("direct rendering must encode shared readback copies after timestamps");

        assert!(prepare_frame < begin_frame);
        assert!(begin_frame < realtime_ibl);
        assert!(realtime_ibl < gpu_scene_upload);
        assert!(gpu_scene_upload < scene_content);
        assert!(begin_frame < scene_content);
        assert!(scene_content < resolve);
        assert!(resolve < encode_copies);
    }

    #[test]
    fn environment_only_timestamp_evidence_has_required_core_scopes_and_optional_realtime_work() {
        let source = production_source();

        for scope in [
            "DIRECT_GPU_SCENE_UPLOAD_GPU_PASS",
            "DIRECT_SCENE_CONTENT_GPU_PASS",
            "DIRECT_OUTPUT_TRANSFER_GPU_PASS",
            "DIRECT_OVERLAYS_GPU_PASS",
        ] {
            assert!(source.contains(scope), "HDRI evidence must retain {scope}");
        }
        assert!(source.contains("realtime_ibl_prepared.is_some()"));
        assert!(source.contains("if let Some(screen_space_ui_renderer)"));
    }

    #[test]
    fn direct_render_defers_gpu_timing_when_shared_readback_cannot_complete() {
        let source = production_source();
        assert!(source.contains("timer.defer_frame(frame_generation)"));
        assert!(source.contains("self.readback_queue.abort_frame(readback_frame_index);"));
        assert!(source.contains(
            "let realtime_ibl_gpu_timing_enabled = readback_ready && gpu_pass_timer.is_some();"
        ));
        assert!(source.contains(
            "let realtime_ibl_scope = (readback_ready && realtime_ibl_prepared.is_some())"
        ));
    }
}
