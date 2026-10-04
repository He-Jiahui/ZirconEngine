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
        if let Some(prepared) = realtime_ibl.filter(|prepared| prepared.uses_realtime_resources()) {
            let slot = prepared.sampling_slot();
            if self.scene_bind_group_realtime_ibl_slot != Some(slot) {
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                self.prepare_realtime_environment_bindings(bind_group, slot);
            }
        } else {
            let mut requires_rebind = self.scene_bind_group_realtime_ibl_slot.is_some();
            if let Some(environment) = frame.source_cubemap_environment() {
                requires_rebind |= self.scene_environment_cubemap.ensure_uploaded(
                    device,
                    encoder,
                    environment,
                    &mut batch,
                )?;
            }
            if requires_rebind {
                self.prepare_static_environment_bindings(device);
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
            self.mesh_pipelines
                .disable_environment_only_pbr_base_profile();
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
            self.encode_environment_sh9_upload(
                device,
                encoder,
                Arc::clone(&payload),
                source_range,
                &mut batch,
            )?;
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
#[path = "tests/write_scene_uniform.rs"]
mod tests;
