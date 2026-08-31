use std::mem::size_of;
use std::sync::Arc;

use super::super::super::primitives::{SceneEnvironmentSh9, SceneUniform};
use super::super::scene_renderer_core::SceneRendererCore;
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::environment::RealtimeIblPreparedFrame;
use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch, WgpuTextureUploadBatch};

impl SceneRendererCore {
    pub(crate) fn prepare_hit_proxy_scene_uniform_upload(
        &self,
        frame: &crate::graphics::types::ViewportRenderFrame,
        pixel: UVec2,
    ) -> Option<WgpuBufferUploadBatch> {
        let mut scene_uniform = SceneUniform::from_hit_proxy_frame(frame, pixel)?;
        scene_uniform.set_global_material_mip_bias(self.global_material_mip_bias);
        let payload: Arc<[u8]> = Arc::from(bytemuck::bytes_of(&scene_uniform));
        let mut batch = WgpuBufferUploadBatch::new();
        batch.push(WgpuBufferUpload::from_bytes(
            self.scene_uniform_buffer.clone(),
            0,
            payload.as_ref(),
        ));
        Some(batch)
    }

    pub(crate) fn write_scene_uniform(
        &mut self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &crate::graphics::types::ViewportRenderFrame,
        realtime_ibl: Option<&RealtimeIblPreparedFrame>,
        reflection_probes_enabled: bool,
        frame_texture_uploads: &mut WgpuTextureUploadBatch,
    ) -> Result<WgpuBufferUploadBatch, GraphicsError> {
        let device = &backend.device;
        let mut batch = WgpuBufferUploadBatch::new();
        self.scene_environment_cubemap.discard_pending_upload();
        if let Some(prepared) = realtime_ibl.filter(|prepared| prepared.uses_realtime_resources()) {
            let slot = prepared.sampling_slot();
            if self.scene_bind_group_realtime_ibl_slot != Some(slot) {
                self.scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("zircon-scene-bind-group-realtime-ibl"),
                    layout: &self.scene_bind_group_layout,
                    entries: &self
                        .scene_environment_cubemap
                        .bind_group_entries_with_environment_views(
                            &self.scene_uniform_buffer,
                            &self.scene_environment_brdf_lut,
                            self.realtime_ibl.source_view(slot),
                            self.realtime_ibl.pmrem_view(slot),
                            self.realtime_ibl.sh9_buffer(slot),
                        ),
                });
                self.scene_bind_group_realtime_ibl_slot = Some(slot);
            }
        } else {
            let mut requires_rebind = self.scene_bind_group_realtime_ibl_slot.take().is_some();
            if let Some(environment) = frame.source_cubemap_environment() {
                requires_rebind |= self.scene_environment_cubemap.ensure_uploaded(
                    device,
                    encoder,
                    environment,
                    &mut batch,
                )?;
            }
            if requires_rebind {
                self.scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("zircon-scene-bind-group"),
                    layout: &self.scene_bind_group_layout,
                    entries: &self.scene_environment_cubemap.bind_group_entries(
                        &self.scene_uniform_buffer,
                        &self.scene_environment_brdf_lut,
                        &self.scene_environment_sh9_buffer,
                    ),
                });
            }
        }
        let _reflection_probe_upload_report = self.mesh_pipelines.reflection_probes.prepare(
            device,
            streamer,
            frame,
            reflection_probes_enabled,
            &mut batch,
            frame_texture_uploads,
        );
        self.deferred
            .set_reflection_probe_bindings(self.mesh_pipelines.reflection_probes.bindings());
        if self
            .mesh_pipelines
            .reflection_probes
            .requires_generic_environment_pbr()
        {
            let environment_only_profile_was_enabled = self
                .mesh_pipelines
                .environment_only_pbr_base_profile_enabled();
            self.mesh_pipelines
                .disable_environment_only_pbr_base_profile();
            if environment_only_profile_was_enabled {
                self.cached_mesh_draw_commands.clear();
            }
        }
        self.mesh_pipelines
            .lightmaps
            .prepare(device, streamer, frame.environment())?;
        self.deferred
            .set_lightmap_bindings(self.mesh_pipelines.lightmaps.bindings());
        let mut scene_uniform = SceneUniform::from_frame(frame);
        scene_uniform.set_global_material_mip_bias(self.global_material_mip_bias);
        if let Some(prepared) = realtime_ibl.filter(|prepared| prepared.uses_realtime_resources()) {
            scene_uniform.use_realtime_ibl(
                prepared.source_face_size(),
                prepared.pmrem_face_size(),
                prepared.pmrem_mip_count(),
            );
        }
        let upload_environment_sh9 =
            !realtime_ibl.is_some_and(RealtimeIblPreparedFrame::uses_realtime_resources);
        let environment_sh9 =
            upload_environment_sh9.then(|| SceneEnvironmentSh9::from_frame(frame));
        let mut payload = Vec::with_capacity(
            size_of::<SceneUniform>()
                + environment_sh9
                    .as_ref()
                    .map_or(0, |_| size_of::<SceneEnvironmentSh9>()),
        );
        let environment_sh9_range = environment_sh9.as_ref().map(|environment_sh9| {
            let start = payload.len();
            payload.extend_from_slice(bytemuck::bytes_of(environment_sh9));
            start..payload.len()
        });
        let scene_uniform_start = payload.len();
        payload.extend_from_slice(bytemuck::bytes_of(&scene_uniform));
        let scene_uniform_range = scene_uniform_start..payload.len();

        let payload: Arc<[u8]> = Arc::from(payload);
        if let Some(source_range) = environment_sh9_range {
            batch.push(
                WgpuBufferUpload::new(
                    self.scene_environment_sh9_buffer.clone(),
                    0,
                    Arc::clone(&payload),
                    source_range,
                )
                .ok_or(GraphicsError::InvalidBufferUploadRange {
                    label: "scene-environment-sh9",
                })?,
            );
        }
        batch.push(
            WgpuBufferUpload::new(
                self.scene_uniform_buffer.clone(),
                0,
                payload,
                scene_uniform_range,
            )
            .ok_or(GraphicsError::InvalidBufferUploadRange {
                label: "scene-uniform",
            })?,
        );
        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn static_cubemap_upload_resets_stale_pending_state_before_recording_the_frame() {
        let uniform_source = include_str!("write_scene_uniform.rs");
        let discard_pending = uniform_source
            .find("self.scene_environment_cubemap.discard_pending_upload();")
            .expect("each scene frame must discard an upload from a dropped prior encoder");
        let upload = uniform_source
            .find("self.scene_environment_cubemap.ensure_uploaded(")
            .expect("source cubemap uploads must be encoded through the scene uniform path");

        assert!(discard_pending < upload);
        assert!(uniform_source[upload..].contains("encoder,"));
        assert!(uniform_source[upload..].contains("&mut batch,"));
    }

    #[test]
    fn provider_upgrade_downgrades_the_environment_variant_before_draw_construction() {
        let uniform_source = include_str!("write_scene_uniform.rs");
        let prepare = uniform_source
            .find(".reflection_probes.prepare(")
            .expect("scene uniform update must prepare reflection providers");
        let provider_fallback = uniform_source
            .find(".requires_generic_environment_pbr()")
            .expect("scene uniform update must observe the provider upgrade");
        let deferred_binding_publication = uniform_source
            .find(".set_reflection_probe_bindings(")
            .expect("deferred lighting must receive the final provider binding lease");
        let previous_profile = uniform_source
            .find("let environment_only_profile_was_enabled = self")
            .expect("provider fallback must detect an actual profile transition");
        let variant_downgrade = uniform_source
            .find(".disable_environment_only_pbr_base_profile()")
            .expect("scene uniform update must select the generic environment variant");
        let command_cache_clear = uniform_source
            .find("self.cached_mesh_draw_commands.clear();")
            .expect("a profile transition must invalidate cached pipeline variants");

        assert!(prepare < deferred_binding_publication);
        assert!(deferred_binding_publication < provider_fallback);
        assert!(provider_fallback < previous_profile);
        assert!(previous_profile < variant_downgrade);
        assert!(variant_downgrade < command_cache_clear);
        assert!(uniform_source.contains("if environment_only_profile_was_enabled"));

        let render_source = include_str!("../scene_renderer_core_render_scene/render_scene.rs");
        let write_uniform = render_source
            .find("self.write_scene_uniform(")
            .expect("direct rendering must update scene state before drawing");
        let build_draws = render_source
            .find(".build_mesh_draws(")
            .expect("direct rendering must build mesh draw commands");

        assert!(
            write_uniform < build_draws,
            "provider fallback must resolve before mesh variants are selected"
        );
    }

    #[test]
    fn incomplete_realtime_ticket_keeps_procedural_environment_bindings() {
        let uniform_source = include_str!("write_scene_uniform.rs");

        assert!(uniform_source.contains("prepared.uses_realtime_resources()"));
        assert!(uniform_source.contains(
            "!realtime_ibl.is_some_and(RealtimeIblPreparedFrame::uses_realtime_resources)"
        ));
    }

    #[test]
    fn scene_constants_prepare_one_packed_frame_upload_batch() {
        let production = include_str!("write_scene_uniform.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("scene uniform test boundary");

        assert!(production.contains("let payload: Arc<[u8]> = Arc::from(payload);"));
        assert!(production.contains("WgpuBufferUploadBatch::new()"));
        assert!(production.contains("Ok(batch)"));
        assert!(production.contains("GraphicsError::InvalidBufferUploadRange"));
        assert!(!production.contains(".expect(\"HitProxy scene uniform upload"));
        assert!(!production.contains(".expect(\"scene environment SH9 upload range"));
        assert!(!production.contains(".expect(\"scene uniform upload range"));
        assert!(!production.contains("enqueue_copy_buffer_upload_batch"));
        assert!(!production.contains("queue.write_buffer("));
    }
}
