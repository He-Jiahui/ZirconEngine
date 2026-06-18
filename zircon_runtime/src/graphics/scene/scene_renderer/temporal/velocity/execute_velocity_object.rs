use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassGpuExecutionContext;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommandReplayer;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

impl RenderPassGpuExecutionContext<'_> {
    pub(in crate::graphics::scene::scene_renderer) fn record_velocity_object_to_resource(
        &mut self,
        pass_name: &str,
        velocity_resource_name: &str,
        depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver();
        let velocity_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            velocity_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "mesh object velocity graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        let stream = mesh_draw_lists.velocity_stream();

        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MeshObjectVelocityPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: velocity_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(depth_attachment_operations(
                    RenderGraphAttachmentOps::load_store(),
                    1.0,
                )),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if stream.is_empty() {
            return Ok(());
        }

        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!(
                "mesh object velocity graph executor for pass `{pass_name}` requires mesh pipeline context"
            )
        })?;
        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(
                self.device,
                self.shadow_atlas_resources,
                None,
                None,
                None,
            );
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
            if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id) {
                let pipeline = mesh_pipelines
                    .ensure_velocity_pipeline_for_variant(self.device, command.pipeline_variant_id)
                    .expect("velocity mesh command must resolve a cache-backed pipeline variant");
                pass.set_pipeline(pipeline);
            }
            replayer.bind_gpu_scene_if_needed(pass, command, mesh_draw_lists.gpu_scene_bind_group);
            replayer.bind_standard_material_if_needed(pass, command);
            replayer.bind_geometry_if_needed(pass, command);
            true
        });
        mesh_draw_lists.replay_stats.record(replayer.stats());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::PostProcessGraphResourceNames;

    #[test]
    fn object_velocity_writes_graph_scene_velocity_resource() {
        assert_eq!(
            PostProcessGraphResourceNames::SCENE_VELOCITY,
            "scene-velocity"
        );
    }

    #[test]
    fn object_velocity_binds_forward_shadow_receiver_group() {
        let source = include_str!("execute_velocity_object.rs");

        assert!(source.contains(
            "create_forward_shadow_receiver_bind_group(\n                self.device,\n                self.shadow_atlas_resources,\n                None,\n                None,\n                None,\n            )"
        ));
        assert!(source.contains("pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[])"));
    }
}
