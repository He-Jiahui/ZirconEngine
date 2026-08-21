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
            device,
            queue,
            streamer,
            frame,
            reflection_probes_enabled,
        );
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
        if !realtime_ibl.is_some_and(RealtimeIblPreparedFrame::uses_realtime_resources) {
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

#[cfg(test)]
mod tests {
    #[test]
    fn provider_upgrade_downgrades_the_environment_variant_before_draw_construction() {
        let uniform_source = include_str!("write_scene_uniform.rs");
        let prepare = uniform_source
            .find(".reflection_probes.prepare(")
            .expect("scene uniform update must prepare reflection providers");
        let provider_fallback = uniform_source
            .find(".requires_generic_environment_pbr()")
            .expect("scene uniform update must observe the provider upgrade");
        let variant_downgrade = uniform_source
            .find(".disable_environment_only_pbr_base_profile()")
            .expect("scene uniform update must select the generic environment variant");

        assert!(prepare < provider_fallback);
        assert!(provider_fallback < variant_downgrade);

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
}
