use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshDrawCommandStream, MeshPassPipelineKind,
};
use crate::graphics::scene::scene_renderer::overlay::BaseScenePass;
use crate::graphics::scene::scene_renderer::transparent::has_transparent_sprite_submissions;
use crate::render_graph::{
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphResourceAccessKind,
};

use super::surface::record_depth_clear_pass;
use super::{RenderPassGpuExecutionContext, RenderPassMeshCommandLists};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeshStageCommandSource {
    Standard(RenderPassStage),
    HalfResolutionTransparent,
    AdvancedPbrOpaque,
    TransmissionStep(usize),
}

impl MeshStageCommandSource {
    fn stage(self) -> RenderPassStage {
        match self {
            Self::Standard(stage) => stage,
            Self::HalfResolutionTransparent => RenderPassStage::Transparent3d,
            Self::AdvancedPbrOpaque => RenderPassStage::Transparent3d,
            Self::TransmissionStep(_) => RenderPassStage::Transparent3d,
        }
    }

    fn stream<'a>(
        self,
        mesh_draw_lists: &RenderPassMeshCommandLists<'a>,
    ) -> MeshDrawCommandStream<'a> {
        match self {
            Self::Standard(stage) => mesh_draw_lists.stream_for_stage(stage),
            Self::HalfResolutionTransparent => mesh_draw_lists.half_resolution_transparent_stream(),
            Self::AdvancedPbrOpaque => mesh_draw_lists.advanced_pbr_opaque_stream(),
            Self::TransmissionStep(step_index) => {
                mesh_draw_lists.transmission_step_stream(step_index)
            }
        }
    }

    fn mixes_transparent_sprites(self) -> bool {
        matches!(self, Self::Standard(RenderPassStage::Transparent3d))
    }
}

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_depth_prepass_to_resources(
        &mut self,
        pass_name: &str,
        depth_resource_name: &str,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        if mesh_draw_lists.depth_prepass_commands.is_empty() {
            return Ok(());
        }
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires resource streamer context"
            )
        })?;
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!(
                "depth prepass graph executor for pass `{pass_name}` requires mesh pipeline context"
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
        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("DepthPrepass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(depth_attachment_operations(depth_attachment_ops, 1.0)),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !self
            .frame
            .render_region()
            .local_render_region()
            .apply_physical_to_render_pass(&mut pass)
        {
            return Ok(());
        }
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(
            &mut pass,
            mesh_draw_lists.depth_prepass_stream(),
            |replayer, pass, command| {
                debug_assert_eq!(command.pipeline_kind, MeshPassPipelineKind::DepthPrepass);
                if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id)
                {
                    let pipeline = mesh_pipelines
                        .ensure_depth_prepass_pipeline_for_variant(
                            self.device,
                            streamer,
                            command.pipeline_variant_id,
                        )
                        .expect(
                            "depth prepass command must resolve a cache-backed pipeline variant",
                        );
                    pass.set_pipeline(pipeline);
                }
                replayer.bind_forward_shadow_receiver_if_needed(
                    pass,
                    &forward_shadow_receiver_bind_group,
                );
                replayer.bind_gpu_scene_if_needed(
                    pass,
                    command,
                    mesh_draw_lists.gpu_scene_bind_group,
                );
                replayer.bind_standard_material_if_needed(pass, command);
                replayer.bind_geometry_if_needed(pass, command);
                true
            },
        );
        let replay_stats = replayer.stats();
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_shadow_atlas_to_resources(
        &mut self,
        pass_name: &str,
        shadow_atlas_resource_name: &str,
        shadow_atlas_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let shadow_atlas_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            shadow_atlas_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        if let Some(shadow_map_renderer) = self.shadow_map_renderer {
            let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
                format!(
                    "shadow atlas graph executor for pass `{pass_name}` requires mesh draw context"
                )
            })?;
            if let Some(shadow_frame_plan) = self.shadow_frame_plan {
                let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
                    format!(
                        "shadow atlas graph executor for pass `{pass_name}` requires mesh pipeline context"
                    )
                })?;
                let streamer = self.streamer.ok_or_else(|| {
                    format!(
                        "shadow atlas graph executor for pass `{pass_name}` requires resource streamer context"
                    )
                })?;
                let replay_stats = shadow_map_renderer.record_atlas_commands_with_attachment_ops(
                    self.device,
                    self.queue,
                    self.encoder,
                    pass_name,
                    shadow_atlas_view,
                    mesh_pipelines,
                    streamer,
                    shadow_frame_plan.atlas_passes(),
                    self.frame,
                    mesh_draw_lists.gpu_scene_bind_group,
                    mesh_draw_lists.shadow_stream(),
                    shadow_atlas_attachment_ops,
                );
                mesh_draw_lists.replay_stats.record(replay_stats);
            } else {
                record_depth_clear_pass(
                    self.encoder,
                    pass_name,
                    shadow_atlas_view,
                    shadow_atlas_attachment_ops,
                );
            }
            return Ok(());
        }
        record_depth_clear_pass(
            self.encoder,
            pass_name,
            shadow_atlas_view,
            shadow_atlas_attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_mesh_stage_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        stage: RenderPassStage,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        self.record_mesh_stage_selection_to_resources(
            color_resource_name,
            depth_resource_name,
            attachment_ops,
            depth_attachment_ops,
            MeshStageCommandSource::Standard(stage),
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_advanced_pbr_opaque_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        self.record_mesh_stage_selection_to_resources(
            color_resource_name,
            depth_resource_name,
            attachment_ops,
            depth_attachment_ops,
            MeshStageCommandSource::AdvancedPbrOpaque,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_half_resolution_transparent_mesh_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        self.record_mesh_stage_selection_to_resources(
            color_resource_name,
            depth_resource_name,
            attachment_ops,
            depth_attachment_ops,
            MeshStageCommandSource::HalfResolutionTransparent,
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn transmission_step_has_commands(
        &self,
        step_index: usize,
    ) -> Result<bool, String> {
        let mesh_draw_lists = self
            .mesh_draw_lists
            .ok_or_else(|| "transmission graph executor requires mesh draw context".to_string())?;
        Ok(!mesh_draw_lists
            .transmission_step_stream(step_index)
            .is_empty())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_transmission_step_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        step_index: usize,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        self.record_mesh_stage_selection_to_resources(
            color_resource_name,
            depth_resource_name,
            attachment_ops,
            depth_attachment_ops,
            MeshStageCommandSource::TransmissionStep(step_index),
        )
    }

    fn record_mesh_stage_selection_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
        command_source: MeshStageCommandSource,
    ) -> Result<(), String> {
        let stage = command_source.stage();
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires mesh draw context")
        })?;
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires mesh pipeline context")
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!("mesh graph executor for stage `{stage:?}` requires resource streamer context")
        })?;
        let mixes_transparent_sprites = command_source.mixes_transparent_sprites()
            && self.sprite_renderer.is_some()
            && has_transparent_sprite_submissions(&self.frame.extract.sprites.phase_queue);
        let stream = command_source.stream(&mesh_draw_lists);
        if stream.is_empty() && !mixes_transparent_sprites {
            return Ok(());
        }
        let light_grid_params_buffer = Self::optional_buffer_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_zbins_buffer = Self::optional_buffer_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_ZBINS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_tile_masks_buffer = Self::optional_buffer_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let integrated_volumetric_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
            RenderGraphResourceAccessKind::Read,
        )?;
        let transmission_scene_color_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
            RenderGraphResourceAccessKind::Read,
        )?;
        let replay_stats = if mixes_transparent_sprites {
            let sprite_renderer = self.sprite_renderer.ok_or_else(|| {
                format!(
                    "transparent mixed graph executor for stage `{stage:?}` requires sprite renderer context"
                )
            })?;
            BaseScenePass.record_transparent_mixed_with_attachment_ops(
                self.encoder,
                self.device,
                color_view,
                depth_view,
                self.scene_bind_group,
                mesh_draw_lists.gpu_scene_bind_group,
                stream.commands(),
                mesh_pipelines,
                streamer,
                sprite_renderer,
                self.frame,
                self.shadow_atlas_resources,
                render_region,
                light_grid_params_buffer,
                light_zbins_buffer,
                light_tile_masks_buffer,
                integrated_volumetric_view,
                transmission_scene_color_view,
                mesh_stage_attachment_ops(stage, attachment_ops),
                mesh_stage_attachment_ops(stage, depth_attachment_ops),
            )
        } else {
            BaseScenePass.record_commands_with_attachment_ops(
                self.encoder,
                self.device,
                color_view,
                depth_view,
                self.scene_bind_group,
                mesh_draw_lists.gpu_scene_bind_group,
                [stream],
                mesh_pipelines,
                streamer,
                self.frame,
                self.shadow_atlas_resources,
                render_region,
                light_grid_params_buffer,
                light_zbins_buffer,
                light_tile_masks_buffer,
                integrated_volumetric_view,
                transmission_scene_color_view,
                mesh_stage_attachment_ops(stage, attachment_ops),
                mesh_stage_attachment_ops(stage, depth_attachment_ops),
            )
        };
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_taa_reactive_mask_mesh_to_resource(
        &mut self,
        pass_name: &str,
        taa_reactive_mask_resource_name: &str,
        scene_depth_resource_name: &str,
    ) -> Result<(), String> {
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!(
                "TAA reactive mask mesh graph executor for pass `{pass_name}` requires mesh draw context"
            )
        })?;
        let stream = mesh_draw_lists.taa_reactive_mask_stream();
        if stream.is_empty() {
            return Ok(());
        }
        let render_region = self
            .render_region_for_write_resource(taa_reactive_mask_resource_name)
            .local_render_region();
        if render_region.is_empty() {
            return Ok(());
        }
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!(
                "TAA reactive mask mesh graph executor for pass `{pass_name}` requires mesh pipeline context"
            )
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "TAA reactive mask mesh graph executor for pass `{pass_name}` requires resource streamer context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let taa_reactive_mask_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            taa_reactive_mask_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let device = self.device;
        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(
                device,
                self.shadow_atlas_resources,
                None,
                None,
                None,
            );
        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass_name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: taa_reactive_mask_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(
                    RenderGraphAttachmentOps::clear_store(),
                    wgpu::Color::BLACK,
                ),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: scene_depth_view,
                depth_ops: Some(depth_attachment_operations(
                    RenderGraphAttachmentOps::load_store(),
                    1.0,
                )),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return Ok(());
        }
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);

        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
            if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id) {
                let pipeline = mesh_pipelines
                    .ensure_taa_reactive_mask_pipeline_for_variant(
                        device,
                        streamer,
                        command.pipeline_variant_id,
                    )
                    .expect(
                        "TAA reactive mask command must resolve a cache-backed pipeline variant",
                    );
                pass.set_pipeline(pipeline);
            }
            replayer.bind_gpu_scene_if_needed(pass, command, mesh_draw_lists.gpu_scene_bind_group);
            replayer.bind_standard_material_if_needed(pass, command);
            replayer.bind_geometry_if_needed(pass, command);
            true
        });
        mesh_draw_lists.replay_stats.record(replayer.stats());
        drop(pass);
        self.record_taa_reactive_mask_encoding(render_region.local_size());
        Ok(())
    }
}

fn mesh_stage_attachment_ops(
    stage: RenderPassStage,
    attachment_ops: RenderGraphAttachmentOps,
) -> RenderGraphAttachmentOps {
    if matches!(stage, RenderPassStage::Opaque3d) {
        return RenderGraphAttachmentOps {
            load: RenderGraphAttachmentLoadOp::Load,
            store: attachment_ops.store,
        };
    }
    attachment_ops
}
