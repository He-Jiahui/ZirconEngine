use crate::graphics::pipeline::PipelineAdmission;
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassGpuExecutionContext;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommandReplayer;
use crate::render_graph::{
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphAttachmentStoreOp,
    RenderGraphResourceAccessKind,
};

const VELOCITY_OBJECT_PIPELINE_CONSUMER: &str = "velocity_object";

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
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "mesh object velocity graph executor for pass `{pass_name}` requires resource streamer context"
            )
        })?;
        // 只有空绘制的 Load+Store 可跳过；Clear 或 Discard 仍须录制 pass，以兑现图声明的附件操作。
        if stream.is_empty()
            && attachment_ops.load == RenderGraphAttachmentLoadOp::Load
            && attachment_ops.store == RenderGraphAttachmentStoreOp::Store
        {
            return Ok(());
        }

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
                match mesh_pipelines.ensure_velocity_pipeline_admission_for_variant(
                    self.device,
                    streamer,
                    command.pipeline_variant_id,
                ) {
                    PipelineAdmission::Ready(()) => {
                        mesh_pipelines.record_bound_mesh_pass_pipeline(
                            command.pipeline_kind,
                            command.pipeline_variant_id,
                        );
                        pass.set_pipeline(
                            mesh_pipelines
                                .velocity_pipeline_for_ready_variant(command.pipeline_variant_id),
                        );
                    }
                    PipelineAdmission::Deferred(unavailable)
                    | PipelineAdmission::Failed(unavailable) => {
                        mesh_pipelines.record_pipeline_fallback_for_command_variant(
                            command,
                            command.pipeline_variant_id,
                            VELOCITY_OBJECT_PIPELINE_CONSUMER,
                            unavailable,
                        );
                        replayer.invalidate_state_after_external_pipeline();
                        return false;
                    }
                }
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
#[path = "tests/execute_velocity_object.rs"]
mod tests;
