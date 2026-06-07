use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
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
        let draws = mesh_draw_lists
            .non_transparent
            .iter()
            .copied()
            .filter(|draw| {
                draw.queue_profile().motion_vector_history_eligible()
                    && draw.has_previous_motion_vector_transform()
            })
            .collect::<Vec<_>>();
        if draws.is_empty() {
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
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        for draw in draws {
            let pipeline =
                mesh_pipelines.ensure_motion_vector_pipeline(self.device, draw.pipeline_key());
            pass.set_pipeline(pipeline);
            draw.bind_model(&mut pass);
            draw.bind_texture(&mut pass);
            draw.bind_material(&mut pass);
            draw.bind_geometry_buffers(&mut pass);
            draw.record_indexed_draw(&mut pass);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::PostProcessGraphResourceNames;

    #[test]
    fn object_motion_vectors_write_graph_raw_motion_resource() {
        assert_eq!(
            PostProcessGraphResourceNames::SCENE_MOTION_VECTOR,
            "scene-motion-vector"
        );
    }
}
