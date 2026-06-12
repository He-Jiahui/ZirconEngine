use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommandReplayer;
use crate::render_graph::RenderGraphAttachmentOps;

use super::RenderPassGpuExecutionContext;

impl RenderPassGpuExecutionContext<'_> {
    pub(in crate::graphics::scene::scene_renderer) fn record_mesh_motion_vectors_to_resource(
        &mut self,
        pass_name: &str,
        motion_vector_resource_name: &str,
        depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let motion_vector_view = self
            .resources
            .require_texture_view(motion_vector_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "mesh object motion-vector graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!(
                "mesh object motion-vector graph executor for pass `{pass_name}` requires mesh pipeline context"
            )
        })?;
        let stream = mesh_draw_lists.velocity_stream();
        if stream.is_empty() {
            return Ok(());
        }

        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MeshObjectMotionVectorPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: motion_vector_view,
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
        let forward_shadow_receiver_bind_group =
            mesh_pipelines.create_forward_shadow_receiver_bind_group(self.device, None);
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
            if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id) {
                let pipeline = mesh_pipelines
                    .ensure_motion_vector_pipeline_for_variant(
                        self.device,
                        command.pipeline_variant_id,
                    )
                    .expect(
                        "motion-vector mesh command must resolve a cache-backed pipeline variant",
                    );
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
    fn object_motion_vectors_write_graph_raw_motion_resource() {
        assert_eq!(
            PostProcessGraphResourceNames::SCENE_MOTION_VECTOR,
            "scene-motion-vector"
        );
    }

    #[test]
    fn object_motion_vectors_bind_forward_shadow_receiver_group() {
        let source = include_str!("mesh_motion_vector.rs");

        assert!(source.contains("create_forward_shadow_receiver_bind_group(self.device, None)"));
        assert!(source.contains("pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[])"));
    }
}
