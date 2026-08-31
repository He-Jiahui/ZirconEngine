use crate::core::framework::render::{
    SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT, SkyboxMode,
};
use crate::graphics::backend::{
    begin_source_cubemap_wgpu_readback, request_source_cubemap_wgpu_readback_batch,
};
use crate::graphics::runtime::render_framework::EnvironmentCaptureWorkItem;
use crate::graphics::scene::scene_renderer::environment::{
    EnvironmentCaptureFilterWgpuRecorder, EnvironmentCaptureGpuTarget,
    EnvironmentCaptureLightGridPlan, EnvironmentCaptureLightGridWorkspace,
    EnvironmentCapturePersistenceSubmission, EnvironmentCapturePersistenceSubmissionStatus,
    EnvironmentCaptureProbePublication, EnvironmentCaptureRenderPlan, EnvironmentCaptureSceneBatch,
    EnvironmentCaptureSceneUniformPlan, EnvironmentCaptureSceneUniformWorkspace,
    EnvironmentCaptureSourceSubmission, EnvironmentCaptureSourceSubmissionStatus,
    EnvironmentCaptureWgpuRecorder,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshSceneDataBindHandle;
use crate::graphics::types::GraphicsError;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(in crate::graphics) fn environment_capture_submission_status(
        &self,
        submission: &EnvironmentCaptureSourceSubmission,
    ) -> Result<EnvironmentCaptureSourceSubmissionStatus, GraphicsError> {
        let resource_upload = self
            .backend
            .submission_status(submission.resource_upload_submission())?;
        let capture = self
            .backend
            .submission_status(submission.capture_submission())?;
        Ok(EnvironmentCaptureSourceSubmissionStatus::from_statuses(
            resource_upload,
            capture,
        ))
    }

    pub(in crate::graphics) fn begin_environment_capture_persistence(
        &mut self,
        source: EnvironmentCaptureSourceSubmission,
    ) -> Result<
        EnvironmentCapturePersistenceSubmission,
        (EnvironmentCaptureSourceSubmission, GraphicsError),
    > {
        let plan = source.target_plan();
        let readback =
            match begin_source_cubemap_wgpu_readback(plan.face_size(), plan.source_mip_count()) {
                Ok(readback) => readback,
                Err(error) => return Err((source, error)),
            };
        let mut persistence = EnvironmentCapturePersistenceSubmission::new(source, readback);
        if let Err(error) = self.submit_environment_capture_persistence_batch(&mut persistence) {
            let (source, _) = persistence.into_parts();
            return Err((source, error));
        }
        Ok(persistence)
    }

    pub(in crate::graphics) fn environment_capture_persistence_status(
        &self,
        persistence: &EnvironmentCapturePersistenceSubmission,
    ) -> Result<EnvironmentCapturePersistenceSubmissionStatus, GraphicsError> {
        if let Some(ticket) = persistence.batch_submission() {
            let status = self.backend.submission_status(ticket)?;
            if matches!(
                status,
                zr_rhi::SubmissionStatus::Failed
                    | zr_rhi::SubmissionStatus::Cancelled
                    | zr_rhi::SubmissionStatus::DeviceLost
            ) {
                return Ok(EnvironmentCapturePersistenceSubmissionStatus::Failed {
                    submission: status,
                });
            }
            if !matches!(status, zr_rhi::SubmissionStatus::Completed) {
                return Ok(EnvironmentCapturePersistenceSubmissionStatus::Pending);
            }
        }
        if persistence.readback().batch_in_flight() {
            return Ok(EnvironmentCapturePersistenceSubmissionStatus::Pending);
        }
        if persistence.readback().all_faces_queued() {
            return Ok(if persistence.readback().poll_ready() {
                EnvironmentCapturePersistenceSubmissionStatus::Completed
            } else {
                EnvironmentCapturePersistenceSubmissionStatus::Pending
            });
        }
        Ok(EnvironmentCapturePersistenceSubmissionStatus::ReadyForNextBatch)
    }

    pub(in crate::graphics) fn advance_environment_capture_persistence(
        &mut self,
        persistence: &mut EnvironmentCapturePersistenceSubmission,
    ) -> Result<(), GraphicsError> {
        self.submit_environment_capture_persistence_batch(persistence)
    }

    fn submit_environment_capture_persistence_batch(
        &mut self,
        persistence: &mut EnvironmentCapturePersistenceSubmission,
    ) -> Result<(), GraphicsError> {
        let diagnostic_frame_index = self.core.diagnostic_frame_index.wrapping_add(1);
        let scope = self
            .backend
            .begin_product_diagnostic_readback_scope(diagnostic_frame_index)?;
        let label = format!(
            "zircon-environment-capture-source-readback-{}",
            persistence.submitted_batch_count()
        );
        request_source_cubemap_wgpu_readback_batch(
            &self.backend,
            persistence.source().target().source_texture(),
            persistence.readback(),
        )?;
        let submission = scope.submit(&label)?;
        self.core.diagnostic_frame_index = diagnostic_frame_index;
        persistence.commit_batch_submission(submission);
        Ok(())
    }

    /// Records and submits the source cubemap for one scheduler-owned capture.
    ///
    /// The scene snapshot is prepared once. Six immutable uniform slots change only the
    /// camera binding while the same opaque draw set is replayed into all cube layers.
    pub(in crate::graphics) fn submit_environment_capture_source(
        &mut self,
        work_item: EnvironmentCaptureWorkItem,
    ) -> Result<EnvironmentCaptureSourceSubmission, GraphicsError> {
        let backend = &self.backend;
        let device = &backend.device;
        let core = &mut self.core;
        let streamer = &mut self.streamer;
        let (handle, mut scene_batch) = EnvironmentCaptureSceneBatch::from_work_item(work_item);
        let request = scene_batch.request().clone();
        let probe_target = match request.reflection_probe_target() {
            Some((probe_id, cubemap)) => {
                let revision = streamer.resource_revision(cubemap).map_err(|error| {
                    GraphicsError::Asset(format!(
                        "resolve environment capture reflection-probe target {cubemap}: {error}"
                    ))
                })?;
                Some((probe_id, cubemap, revision))
            }
            None => None,
        };
        let render_plan = EnvironmentCaptureRenderPlan::from_request(&request);
        let target = EnvironmentCaptureGpuTarget::new(device, &request);
        if probe_target.is_some() {
            core.mesh_pipelines
                .reflection_probes
                .ensure_environment_capture_provider(device);
        }

        core.mesh_pipelines
            .collect_terminal_pipeline_submissions(|ticket| backend.submission_status(ticket).ok());
        core.mesh_pipelines.begin_submission_usage_recording();
        core.mesh_pipelines
            .begin_forward_receiver_binding_profile_frame();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-environment-capture-source-encoder"),
        });
        let mut buffer_uploads = zr_rhi_wgpu::WgpuBufferUploadBatch::new();
        let texture_uploads = zr_rhi_wgpu::WgpuTextureUploadBatch::new();

        core.scene_environment_cubemap.discard_pending_upload();
        let realtime_slot = matches!(
            scene_batch.frame().environment().skybox.mode,
            SkyboxMode::ProceduralGradient
        )
        .then_some(core.scene_bind_group_realtime_ibl_slot)
        .flatten();
        if realtime_slot.is_none() {
            if let Some(environment) = scene_batch
                .frame()
                .environment()
                .skybox
                .source_cubemap_environment()
            {
                core.scene_environment_cubemap.ensure_uploaded(
                    device,
                    &mut encoder,
                    environment,
                    &mut buffer_uploads,
                )?;
            }
        }

        let uniform_plan = if realtime_slot.is_some() {
            EnvironmentCaptureSceneUniformPlan::from_scene_batch_with_realtime_ibl(
                &mut scene_batch,
                core.global_material_mip_bias,
                SOURCE_CUBEMAP_PMREM_FACE_SIZE,
                SOURCE_CUBEMAP_PMREM_FACE_SIZE,
                SOURCE_CUBEMAP_PMREM_MIP_COUNT,
            )
        } else {
            EnvironmentCaptureSceneUniformPlan::from_scene_batch(
                &mut scene_batch,
                core.global_material_mip_bias,
            )
        };
        let light_grid_plan = EnvironmentCaptureLightGridPlan::from_scene_batch(&mut scene_batch);
        if light_grid_plan.has_lights() {
            let environment_only_profile_was_enabled = core
                .mesh_pipelines
                .environment_only_pbr_base_profile_enabled();
            core.mesh_pipelines
                .disable_environment_only_pbr_base_profile();
            if environment_only_profile_was_enabled {
                core.cached_mesh_draw_commands.clear();
            }
        }
        core.gpu_scene
            .write_lights(device, light_grid_plan.lights());
        let light_grid_workspace = light_grid_plan
            .has_lights()
            .then(|| EnvironmentCaptureLightGridWorkspace::new(device, &light_grid_plan));
        let uniform_workspace =
            EnvironmentCaptureSceneUniformWorkspace::new(device, |uniform_buffer| {
                let entries = if let Some(slot) = realtime_slot {
                    core.scene_environment_cubemap
                        .bind_group_entries_with_environment_views(
                            uniform_buffer,
                            &core.scene_environment_brdf_lut,
                            core.realtime_ibl.source_view(slot),
                            core.realtime_ibl.pmrem_view(slot),
                            core.realtime_ibl.sh9_buffer(slot),
                        )
                } else {
                    core.scene_environment_cubemap.bind_group_entries(
                        uniform_buffer,
                        &core.scene_environment_brdf_lut,
                        &core.scene_environment_sh9_buffer,
                    )
                };
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("zircon-environment-capture-scene-bind-group"),
                    layout: &core.scene_bind_group_layout,
                    entries: &entries,
                })
            });
        buffer_uploads.append(&mut uniform_plan.prepare_uploads(&uniform_workspace)?);
        if let Some(workspace) = light_grid_workspace.as_ref() {
            buffer_uploads.append(&mut light_grid_plan.prepare_uploads(workspace)?);
        }
        crate::profile_counter!(
            "render",
            "environment_capture_direct_light_count",
            light_grid_plan.light_count()
        );
        crate::profile_counter!(
            "render",
            "environment_capture_light_grid_upload_count",
            light_grid_workspace
                .as_ref()
                .map(|_| light_grid_plan.upload_count())
                .unwrap_or(0)
        );
        crate::profile_counter!(
            "render",
            "environment_capture_light_grid_payload_bytes",
            light_grid_workspace
                .as_ref()
                .map(|_| light_grid_plan.payload_bytes())
                .unwrap_or(0)
        );
        crate::profile_counter!(
            "render",
            "environment_capture_light_grid_gpu_bytes",
            light_grid_workspace
                .as_ref()
                .map(EnvironmentCaptureLightGridWorkspace::allocated_bytes)
                .unwrap_or(0)
        );

        let mut built_mesh_draws = core
            .advanced_plugin_resources
            .build_environment_capture_mesh_draws(
                backend,
                &mut encoder,
                &core.material_texture_bind_group_layout,
                &mut core.gpu_scene,
                streamer,
                &mut core.mesh_pipelines,
                scene_batch.frame(),
            )?;
        let mut gpu_scene_prepared_upload = built_mesh_draws.take_gpu_scene_prepared_upload();
        gpu_scene_prepared_upload.append_to(&core.gpu_scene, &mut buffer_uploads);
        let material_pipeline_requirements = built_mesh_draws.take_material_pipeline_requirements();
        crate::graphics::scene::scene_renderer::mesh::coordinate_material_pipeline_publications(
            device,
            streamer,
            &mut core.mesh_pipelines,
            material_pipeline_requirements,
            false,
            false,
        );
        let mesh_draws = built_mesh_draws.into_draws();
        let gpu_scene_bind_group = core.gpu_scene.scene_bind_group().clone();
        let gpu_scene_bind_handle = MeshSceneDataBindHandle::new(&gpu_scene_bind_group);
        let shader_quality = scene_batch.frame().shader_quality();
        let record_report = EnvironmentCaptureWgpuRecorder::record(
            &mut encoder,
            device,
            &target,
            &render_plan,
            &mut scene_batch,
            &uniform_workspace,
            light_grid_workspace.as_ref(),
            &mesh_draws,
            Some(gpu_scene_bind_handle),
            &mut core.mesh_pipelines,
            streamer,
            &mut core.overlay_renderer,
            shader_quality,
        )
        .map_err(GraphicsError::WgpuValidation)?;
        let filter_report = EnvironmentCaptureFilterWgpuRecorder::record(
            device,
            &mut encoder,
            &request,
            &target,
            &core.environment_capture_mip_pipelines,
            &mut core.ibl_bake_pipeline_cache,
        )
        .map_err(GraphicsError::WgpuValidation)?;
        let probe_publication = match probe_target {
            Some((probe_id, cubemap, revision)) => {
                let reservation = core
                    .mesh_pipelines
                    .reflection_probes
                    .reserve_environment_capture_target(cubemap, revision)
                    .ok_or_else(|| {
                        GraphicsError::Asset(format!(
                            "reserve environment capture reflection-probe target {cubemap}"
                        ))
                    })?;
                Some(EnvironmentCaptureProbePublication::new(
                    probe_id,
                    cubemap,
                    reservation,
                ))
            }
            None => None,
        };
        if let Some(publication) = probe_publication {
            core.mesh_pipelines
                .reflection_probes
                .copy_environment_capture_probe(
                    &mut encoder,
                    target.pmrem_texture(),
                    publication.reservation().slot(),
                );
        }

        // Artifact readback borrows the filtered capture target and shares the capture ticket.
        // The renderer-owned cache key includes this exact capture recipe; the external
        // asset/editor identity is carried only as publication intent. Admission is optional:
        // a runtime-cache miss must never invalidate the visible capture.
        let mut prepared_persistence = None;
        let mut persistence_diagnostic_scope = None;
        if let Some(runtime_cache_request) = request.runtime_cache_artifact_request() {
            let cache_store = streamer
                .asset_manager()
                .ok()
                .and_then(|manager| manager.ibl_bake_artifact_cache_store());
            if let Some(cache_store) = cache_store {
                let diagnostic_frame_index = core.diagnostic_frame_index.wrapping_add(1);
                if let Ok(scope) =
                    backend.begin_product_diagnostic_readback_scope(diagnostic_frame_index)
                {
                    match core
                        .ibl_bake_runtime_writebacks
                        .prepare_from_capture_target(
                            backend,
                            cache_store,
                            runtime_cache_request,
                            &target,
                        ) {
                        Ok(Some(prepared)) => {
                            core.diagnostic_frame_index = diagnostic_frame_index;
                            prepared_persistence = Some(prepared);
                            persistence_diagnostic_scope = Some(scope);
                        }
                        Ok(None) | Err(_) => drop(scope),
                    }
                }
            }
        }

        let resource_upload_submission = match backend.enqueue_copy_resource_upload_batch(
            zr_rhi_wgpu::WgpuResourceUploadBatch::from_batches(buffer_uploads, texture_uploads),
        ) {
            Ok(ticket) => ticket,
            Err(error) => {
                if let Some(publication) = probe_publication {
                    core.mesh_pipelines
                        .reflection_probes
                        .cancel_environment_capture_target(publication.reservation());
                }
                return Err(error);
            }
        };
        let persistence_diagnostic_frame = persistence_diagnostic_scope
            .and_then(|scope| {
                scope
                    .prepare("environment-capture-ibl-artifact", &mut encoder)
                    .ok()
            })
            .flatten();
        if persistence_diagnostic_frame.is_none() {
            prepared_persistence = None;
        }
        let capture_submission = match backend.submit_graphics_command_buffers_with_diagnostics(
            vec![encoder.finish()],
            persistence_diagnostic_frame,
        ) {
            Ok(ticket) => ticket,
            Err(error) => {
                if let Some(publication) = probe_publication {
                    core.mesh_pipelines
                        .reflection_probes
                        .cancel_environment_capture_target(publication.reservation());
                }
                return Err(error);
            }
        };

        core.mesh_pipelines
            .bind_recorded_pipeline_usage_to_submission(capture_submission);
        if let Some(prepared) = prepared_persistence {
            core.ibl_bake_runtime_writebacks.commit_submitted(prepared);
        }
        core.scene_environment_cubemap.commit_pending_upload();
        gpu_scene_prepared_upload.commit(&mut core.gpu_scene);
        core.mesh_pipelines
            .emit_forward_receiver_binding_profile_frame();

        Ok(EnvironmentCaptureSourceSubmission::new(
            handle,
            request,
            target,
            resource_upload_submission,
            capture_submission,
            record_report,
            filter_report,
            probe_publication,
        ))
    }

    pub(in crate::graphics) fn commit_environment_capture_probe(
        &mut self,
        publication: EnvironmentCaptureProbePublication,
    ) {
        debug_assert_eq!(publication.cubemap(), publication.reservation().cubemap());
        self.core
            .mesh_pipelines
            .reflection_probes
            .commit_environment_capture_target(publication.reservation());
    }

    pub(in crate::graphics) fn cancel_environment_capture_probe(
        &mut self,
        publication: EnvironmentCaptureProbePublication,
    ) {
        self.core
            .mesh_pipelines
            .reflection_probes
            .cancel_environment_capture_target(publication.reservation());
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("scene_renderer_environment_capture.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("environment capture submission must retain a test boundary")
    }

    fn source_capture_owner() -> &'static str {
        let source = production_source();
        let start = source
            .find("pub(in crate::graphics) fn submit_environment_capture_source(")
            .expect("source capture owner");
        let end = source[start..]
            .find("pub(in crate::graphics) fn commit_environment_capture_probe(")
            .map(|offset| start + offset)
            .expect("source capture owner boundary");
        &source[start..end]
    }

    fn persistence_submission_owner() -> &'static str {
        let source = production_source();
        let start = source
            .find("fn submit_environment_capture_persistence_batch(")
            .expect("environment capture persistence submission owner");
        let end = source[start..]
            .find("/// Records and submits the source cubemap")
            .map(|offset| start + offset)
            .expect("environment capture persistence submission owner boundary");
        &source[start..end]
    }

    #[test]
    fn source_capture_merges_uploads_and_submits_one_six_face_command_buffer() {
        let source = source_capture_owner();

        assert!(source.contains("fn submit_environment_capture_source("));
        assert!(source.contains("EnvironmentCaptureSceneUniformPlan::"));
        assert!(source.contains("build_environment_capture_mesh_draws("));
        assert!(source.contains("EnvironmentCaptureWgpuRecorder::record("));
        assert!(source.contains("EnvironmentCaptureFilterWgpuRecorder::record("));
        let record = source
            .find("EnvironmentCaptureWgpuRecorder::record(")
            .expect("capture recorder");
        let reject_incomplete = source[record..]
            .find(".map_err(GraphicsError::WgpuValidation)?")
            .map(|offset| record + offset)
            .expect("incomplete capture rejection");
        let filter = source
            .find("EnvironmentCaptureFilterWgpuRecorder::record(")
            .expect("capture filter");
        assert!(record < reject_incomplete);
        assert!(reject_incomplete < filter);
        let provider = source
            .find("ensure_environment_capture_provider(device)")
            .expect("capture must size the probe provider before publication");
        let reservation = source
            .find("reserve_environment_capture_target(cubemap, revision)")
            .expect("capture must reserve the target after provider sizing");
        assert!(provider < reservation);
        assert_eq!(
            source
                .matches("enqueue_copy_resource_upload_batch(")
                .count(),
            1
        );
        assert_eq!(
            source
                .matches("submit_graphics_command_buffers_with_diagnostics(")
                .count(),
            1
        );
        assert_eq!(source.matches("copy_environment_capture_probe(").count(), 1);
    }

    #[test]
    fn source_capture_preserves_existing_submission_transactions() {
        let source = source_capture_owner();

        assert!(source.contains("begin_submission_usage_recording()"));
        assert!(source.contains("take_gpu_scene_prepared_upload()"));
        assert!(source.contains("gpu_scene_prepared_upload.append_to("));
        assert!(source.contains("bind_recorded_pipeline_usage_to_submission("));
        assert!(source.contains("scene_environment_cubemap.commit_pending_upload()"));
        assert!(!source.contains("reflection_probes\n            .commit_pending_uploads()"));
        assert!(source.contains("gpu_scene_prepared_upload.commit("));
        assert!(!source.contains("roll_prev_transforms_after_success"));
    }

    #[test]
    fn source_capture_writes_one_light_list_and_uploads_six_face_grids() {
        let source = source_capture_owner();
        let light_plan = source
            .find("EnvironmentCaptureLightGridPlan::from_scene_batch(")
            .expect("capture light-grid plan");
        let light_write = source
            .find(".write_lights(device, light_grid_plan.lights())")
            .expect("capture GPU light write");
        let mesh_build = source
            .find("build_environment_capture_mesh_draws(")
            .expect("capture mesh build");

        assert!(light_plan < light_write);
        assert!(light_write < mesh_build);
        let generic_profile = source
            .find("disable_environment_only_pbr_base_profile()")
            .expect("lit capture must use a direct-light PBR variant");
        let invalidate_cached_commands = source
            .find("cached_mesh_draw_commands.clear()")
            .expect("the Base-profile transition must invalidate cached viewport commands");
        assert!(light_plan < generic_profile);
        assert!(generic_profile < mesh_build);
        assert!(generic_profile < invalidate_cached_commands);
        assert!(invalidate_cached_commands < mesh_build);
        assert!(source.contains("environment_only_profile_was_enabled"));
        assert!(source.contains("EnvironmentCaptureLightGridWorkspace::new("));
        assert!(source.contains("light_grid_plan.prepare_uploads("));
    }

    #[test]
    fn source_capture_does_not_publish_materials_or_prepare_viewport_sidebands() {
        let source = source_capture_owner();

        assert!(source.contains("coordinate_material_pipeline_publications("));
        assert!(source.contains("false,\n            false,"));
        assert!(!source.contains("prepare_buffers("));
        assert!(!source.contains("build_shadow_frame_plan"));
        assert!(!source.contains("record_overlays("));
        assert!(!source.contains("screen_space_ui"));
    }

    #[test]
    fn completion_check_observes_both_tickets_without_polling_or_waiting() {
        let source = production_source();
        let status_owner = source
            .split("fn environment_capture_submission_status(")
            .nth(1)
            .and_then(|source| source.split("fn submit_environment_capture_source(").next())
            .expect("environment capture submission status owner");

        assert_eq!(status_owner.matches("submission_status(").count(), 2);
        assert!(status_owner.contains("resource_upload_submission()"));
        assert!(status_owner.contains("capture_submission()"));
        assert!(!status_owner.contains("poll_submission_completions"));
        assert!(!status_owner.contains("wait_for_submission"));
    }

    #[test]
    fn probe_array_copy_is_recorded_before_submission_but_commit_is_deferred() {
        let source = source_capture_owner();
        let copy = source
            .find("copy_environment_capture_probe(")
            .expect("capture must record its typed probe-array copy");
        let submit = source
            .find("submit_graphics_command_buffers_with_diagnostics(")
            .expect("capture must submit the completed encoder");
        assert!(copy < submit);
        assert!(source.contains("commit_environment_capture_probe("));
    }

    #[test]
    fn explicit_probe_target_resolution_and_reservation_fail_closed() {
        let source = source_capture_owner();
        let resolution = source
            .find("let probe_target = match request.reflection_probe_target()")
            .expect("typed probe target revision admission");
        let target_allocation = source
            .find("let target = EnvironmentCaptureGpuTarget::new(device, &request)")
            .expect("capture target allocation");
        let provider_expansion = source
            .find("ensure_environment_capture_provider(device)")
            .expect("reflection provider expansion");
        let target = source
            .split("let probe_publication =")
            .nth(1)
            .and_then(|source| source.split("// Artifact readback").next())
            .expect("typed probe target owner");

        assert!(resolution < target_allocation);
        assert!(resolution < provider_expansion);
        assert!(
            source[resolution..target_allocation]
                .contains("streamer.resource_revision(cubemap).map_err(")
        );
        assert!(target.contains("match probe_target"));
        assert!(target.contains("reserve_environment_capture_target(cubemap, revision)"));
        assert!(target.contains(".ok_or_else("));
        assert!(!target.contains(".and_then("));
        assert!(!target.contains(".ok()?"));
    }

    #[test]
    fn renderer_owned_ibl_cache_readback_shares_capture_ticket_and_commits_after_submit() {
        let source = source_capture_owner();
        let request = source
            .find("request.runtime_cache_artifact_request()")
            .expect("capture must derive the renderer-owned runtime-cache request");
        let prepare = source
            .find("prepare_from_capture_target(")
            .expect("capture must prepare readback from the filtered target");
        let diagnostic = source
            .find("begin_product_diagnostic_readback_scope(")
            .expect("artifact readback must reserve a bounded diagnostic frame");
        let submit = source
            .find("submit_graphics_command_buffers_with_diagnostics(")
            .expect("artifact readback must share the capture submission ticket");
        let commit = source
            .find("commit_submitted(prepared)")
            .expect("artifact writeback must commit only after submission");
        assert!(request < diagnostic);
        assert!(diagnostic < prepare);
        assert!(prepare < submit);
        assert!(submit < commit);
        assert!(
            !source
                .contains("if let Some(artifact_request) = request.persistence_artifact_request()")
        );
    }

    #[test]
    fn source_persistence_uses_bounded_diagnostic_batches_without_waiting() {
        let source = production_source();
        let begin = source
            .find("fn begin_environment_capture_persistence(")
            .expect("persistence begin owner");
        let end = source[begin..]
            .find("/// Records and submits the source cubemap")
            .map(|offset| begin + offset)
            .expect("persistence owner boundary");
        let persistence = &source[begin..end];

        assert!(persistence.contains("begin_source_cubemap_wgpu_readback("));
        assert!(persistence.contains("request_source_cubemap_wgpu_readback_batch("));
        assert!(persistence.contains("begin_product_diagnostic_readback_scope("));
        assert!(persistence.contains("scope.submit(&label)?"));
        assert!(persistence.contains("Err((source, error))"));
        assert!(!persistence_submission_owner().contains(".device"));
        assert!(!persistence_submission_owner().contains("create_command_encoder"));
        assert!(!persistence_submission_owner().contains("scope.prepare"));
        assert!(
            !persistence_submission_owner()
                .contains("submit_graphics_command_buffers_with_diagnostics(")
        );
        assert!(!persistence.contains("device.poll("));
        assert!(!persistence.contains("wait_for_submission"));
    }
}
