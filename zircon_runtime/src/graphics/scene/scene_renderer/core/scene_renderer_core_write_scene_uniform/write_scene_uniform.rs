use super::super::super::primitives::{SceneEnvironmentSh9, SceneUniform};
use super::super::scene_renderer_core::SceneRendererCore;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::environment::RealtimeIblPreparedFrame;
use crate::graphics::types::GraphicsError;

impl SceneRendererCore {
    pub(crate) fn write_scene_uniform(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &crate::graphics::types::ViewportRenderFrame,
        realtime_ibl: Option<&RealtimeIblPreparedFrame>,
        reflection_probes_enabled: bool,
    ) -> Result<(), GraphicsError> {
        if let Some(prepared) = realtime_ibl {
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
            if let Some(environment) = frame.environment().skybox.source_cubemap_environment() {
                requires_rebind |=
                    self.scene_environment_cubemap
                        .ensure_uploaded(device, queue, environment);
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
            queue,
            streamer,
            frame,
            reflection_probes_enabled,
        );
        self.mesh_pipelines
            .lightmaps
            .prepare(device, streamer, frame.environment())?;
        self.deferred
            .set_lightmap_bindings(self.mesh_pipelines.lightmaps.bindings());
        let mut scene_uniform = SceneUniform::from_frame(frame);
        if let Some(prepared) = realtime_ibl {
            scene_uniform.use_realtime_ibl(
                prepared.source_face_size(),
                prepared.pmrem_face_size(),
                prepared.pmrem_mip_count(),
            );
        }
        if realtime_ibl.is_none() {
            queue.write_buffer(
                &self.scene_environment_sh9_buffer,
                0,
                bytemuck::bytes_of(&SceneEnvironmentSh9::from_frame(frame)),
            );
        }
        queue.write_buffer(
            &self.scene_uniform_buffer,
            0,
            bytemuck::bytes_of(&scene_uniform),
        );
        Ok(())
    }
}
