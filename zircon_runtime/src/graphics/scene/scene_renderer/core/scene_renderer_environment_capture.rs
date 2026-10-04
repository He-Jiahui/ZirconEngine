use std::sync::Arc;

use crate::core::framework::render::{
    RenderEnvironmentCaptureHandle, SkyboxMode, SOURCE_CUBEMAP_PMREM_FACE_SIZE,
    SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};
use crate::graphics::backend::{
    begin_source_cubemap_wgpu_readback, request_source_cubemap_wgpu_readback_batch,
};
use crate::graphics::runtime::EnvironmentCaptureWorkItem;
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
        let (handle, scene_batch) = EnvironmentCaptureSceneBatch::from_work_item(work_item);
        self.submit_environment_capture_scene_batch(handle, scene_batch)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn submit_environment_capture_scene_batch(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
        mut scene_batch: EnvironmentCaptureSceneBatch,
    ) -> Result<EnvironmentCaptureSourceSubmission, GraphicsError> {
        let backend = &self.backend;
        let device = &backend.device;
        let mut environment_frame = self.core.begin_scene_environment_frame();
        let core = environment_frame.core();
        let streamer = &mut self.streamer;
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
                let requires_rebind = core.scene_environment_cubemap.ensure_uploaded(
                    device,
                    &mut encoder,
                    environment,
                    &mut buffer_uploads,
                )?;
                if requires_rebind || core.scene_bind_group_realtime_ibl_slot.is_some() {
                    core.prepare_static_environment_bindings(device);
                }
                let environment_sh9 = crate::graphics::scene::scene_renderer::primitives::SceneEnvironmentSh9::from_frame(scene_batch.frame());
                let payload: Arc<[u8]> = Arc::from(bytemuck::bytes_of(&environment_sh9));
                let source_range = 0..payload.len();
                core.encode_environment_sh9_upload(
                    device,
                    &mut encoder,
                    payload,
                    source_range,
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
            core.mesh_pipelines
                .disable_environment_only_pbr_base_profile();
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
                        core.frame_environment_sh9_buffer(),
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
                return Err(
                    match backend.settle_abandoned_submissions(&[resource_upload_submission]) {
                        Ok(_) => error,
                        Err(settlement) => GraphicsError::FrameSubmissionSettlement {
                            settlement: settlement.to_string(),
                            source: Box::new(error),
                        },
                    },
                );
            }
        };

        core.mesh_pipelines
            .bind_recorded_pipeline_usage_to_submission(capture_submission);
        if let Some(prepared) = prepared_persistence {
            core.ibl_bake_runtime_writebacks.commit_submitted(prepared);
        }
        core.commit_scene_environment_frame();
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
#[path = "tests/scene_renderer_environment_capture.rs"]
mod tests;
