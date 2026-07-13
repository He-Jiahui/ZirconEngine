use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::core::framework::render::{DisplayMode, OitBufferPlan, OitSettings};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::GpuTextureResource;
use crate::graphics::scene::scene_renderer::advanced_lighting::oit_buffers::OitFragmentStorePipeline;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshPassPipelineKind,
};
use crate::graphics::scene::scene_renderer::sprite::{build_sprite_vertices, SpriteVertex};
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OitGpuSettings {
    viewport_width: u32,
    viewport_height: u32,
    viewport_origin_x: u32,
    viewport_origin_y: u32,
    fragments_per_pixel: u32,
    sorted_fragment_max_count: u32,
    alpha_threshold: f32,
    _padding: u32,
}

struct PreparedOitSpriteDraw {
    _texture: Arc<GpuTextureResource>,
    texture_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl RenderPassGpuExecutionContext<'_> {
    pub(in crate::graphics::scene::scene_renderer) fn record_oit_fragment_store_to_resources(
        &mut self,
        pipeline: &OitFragmentStorePipeline,
        depth_resource_name: &str,
        layers: &wgpu::Buffer,
        counts: &wgpu::Buffer,
        settings: OitSettings,
    ) -> Result<(), String> {
        let depth_view = Self::require_texture_view_by_name(
            &*self.resources,
            self.resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let mesh_draw_lists = self
            .mesh_draw_lists
            .ok_or_else(|| "OIT fragment store requires mesh draw context".to_string())?;
        let streamer = self
            .streamer
            .ok_or_else(|| "OIT fragment store requires resource streamer context".to_string())?;
        let render_region = self.render_region().local_render_region();
        let light_grid_params_buffer = Self::optional_buffer_by_name(
            &*self.resources,
            self.resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_zbins_buffer = Self::optional_buffer_by_name(
            &*self.resources,
            self.resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_ZBINS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_tile_masks_buffer = Self::optional_buffer_by_name(
            &*self.resources,
            self.resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let integrated_volumetric_view = Self::optional_texture_view_by_name(
            &*self.resources,
            self.resource_resolver,
            crate::core::framework::render::PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
            RenderGraphResourceAccessKind::Read,
        )?;
        let viewport_size = render_region.physical_size();
        let plan = OitBufferPlan::for_view([viewport_size.x, viewport_size.y], settings);
        let gpu_settings = OitGpuSettings {
            viewport_width: viewport_size.x.max(1),
            viewport_height: viewport_size.y.max(1),
            viewport_origin_x: render_region.physical_position().x,
            viewport_origin_y: render_region.physical_position().y,
            fragments_per_pixel: plan.fragments_per_pixel_capacity,
            sorted_fragment_max_count: settings.sorted_fragment_max_count.clamp(1, 32),
            alpha_threshold: settings.alpha_threshold.clamp(0.0, 1.0),
            _padding: 0,
        };
        let uniform = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-oit-fragment-store-settings"),
                contents: bytemuck::bytes_of(&gpu_settings),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mesh_pipelines = self
            .mesh_pipelines
            .as_deref_mut()
            .ok_or_else(|| "OIT fragment store requires mesh pipeline context".to_string())?;
        let oit_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-oit-fragment-store-bind-group"),
            layout: mesh_pipelines.oit_fragment_store_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: layers.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });
        let sprite_draws = prepare_oit_sprite_draws(self.device, streamer, self.frame, pipeline);
        let forward_bind_group = mesh_pipelines.create_forward_shading_bind_group(
            self.device,
            self.frame,
            render_region,
            self.shadow_atlas_resources,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            integrated_volumetric_view,
        );
        let mut pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("oit.fragment_store"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
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
        if !render_region.apply_physical_to_render_pass(&mut pass)
            || self.frame.overlays().display_mode == DisplayMode::WireOnly
        {
            return Ok(());
        }
        pass.set_bind_group(0, self.scene_bind_group, &[]);
        pass.set_bind_group(1, &forward_bind_group, &[]);
        pass.set_bind_group(4, &oit_bind_group, &[]);
        let mut unsupported_shader = None;
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(
            &mut pass,
            mesh_draw_lists.transparent_stream(),
            |replayer, pass, command| {
                debug_assert_eq!(command.pipeline_kind, MeshPassPipelineKind::Base);
                if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id)
                {
                    let Some(oit_pipeline) = mesh_pipelines.ensure_oit_pipeline_for_base_variant(
                        self.device,
                        streamer,
                        command.pipeline_variant_id,
                    ) else {
                        unsupported_shader = Some(command.pipeline_key().shader_id.clone());
                        return false;
                    };
                    pass.set_pipeline(oit_pipeline);
                }
                replayer.bind_gpu_scene_if_needed(
                    pass,
                    command,
                    mesh_draw_lists.gpu_scene_bind_group,
                );
                if mesh_pipelines
                    .pipeline_uses_builtin_fallback_shader(streamer, command.pipeline_key())
                {
                    replayer.bind_standard_material_if_needed(pass, command);
                } else {
                    replayer.bind_material_if_needed(pass, command);
                }
                replayer.bind_geometry_if_needed(pass, command);
                true
            },
        );
        let replay_stats = replayer.stats();
        for draw in &sprite_draws {
            pass.set_pipeline(pipeline.sprite_pipeline());
            pass.set_bind_group(0, self.scene_bind_group, &[]);
            pass.set_bind_group(1, &draw.texture_bind_group, &[]);
            pass.set_bind_group(4, &oit_bind_group, &[]);
            pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
            pass.draw(0..draw.vertex_count, 0..1);
        }
        drop(pass);
        mesh_draw_lists.replay_stats.record(replay_stats);
        if let Some(shader_id) = unsupported_shader {
            return Err(format!(
                "transparent shader `{shader_id}` does not expose the OIT `fs_oit` contract"
            ));
        }
        Ok(())
    }
}

fn prepare_oit_sprite_draws(
    device: &wgpu::Device,
    streamer: &crate::graphics::scene::resources::ResourceStreamer,
    frame: &crate::graphics::types::ViewportRenderFrame,
    pipeline: &OitFragmentStorePipeline,
) -> Vec<PreparedOitSpriteDraw> {
    build_sprite_vertices(frame, RenderPassStage::Transparent3d)
        .into_iter()
        .filter_map(|(sprite_index, vertices)| {
            let vertex_count = u32::try_from(vertices.len()).ok()?;
            let sprite = frame.sprites().get(sprite_index)?;
            let texture = streamer.texture(Some(sprite.image.id()));
            let texture_bind_group = pipeline.create_sprite_texture_bind_group(
                device,
                texture.view(),
                texture.sampler(),
            );
            let vertex_buffer = create_sprite_vertex_buffer(device, &vertices);
            Some(PreparedOitSpriteDraw {
                _texture: texture,
                texture_bind_group,
                vertex_buffer,
                vertex_count,
            })
        })
        .collect()
}

fn create_sprite_vertex_buffer(device: &wgpu::Device, vertices: &[SpriteVertex]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-oit-sprite-vertices"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}
