use std::collections::BTreeSet;

use bytemuck::bytes_of;
use wgpu::util::DeviceExt;

use crate::core::framework::scene::EntityId;
use crate::core::math::{is_finite_mat4, Mat4};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshDrawCommandStream, MeshDrawReplayStats, MeshPassPipelineKind,
    MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::visibility::VisibilityViewKey;
use crate::render_graph::RenderGraphAttachmentOps;

use super::plan::ShadowAtlasSlotPass;

pub(crate) struct ShadowMapRenderer {
    scene_layout: wgpu::BindGroupLayout,
    _environment_cube_texture: wgpu::Texture,
    environment_cube_view: wgpu::TextureView,
    environment_cube_sampler: wgpu::Sampler,
    _environment_brdf_lut_texture: wgpu::Texture,
    environment_brdf_lut_view: wgpu::TextureView,
    _environment_specular_cube_texture: wgpu::Texture,
    environment_specular_cube_view: wgpu::TextureView,
    _environment_irradiance_cube_texture: wgpu::Texture,
    environment_irradiance_cube_view: wgpu::TextureView,
}

impl ShadowMapRenderer {
    pub(crate) fn new(device: &wgpu::Device, scene_layout: &wgpu::BindGroupLayout) -> Self {
        let (environment_cube_texture, environment_cube_view) = create_shadow_environment_cube(
            device,
            "zircon-shadow-map-scene-environment-cube",
            "zircon-shadow-map-scene-environment-cube-view",
        );
        let environment_cube_sampler = create_shadow_environment_sampler(device);
        let (environment_brdf_lut_texture, environment_brdf_lut_view) =
            create_shadow_environment_brdf_lut(device);
        let (environment_specular_cube_texture, environment_specular_cube_view) =
            create_shadow_environment_cube(
                device,
                "zircon-shadow-map-scene-environment-specular-pmrem-cube",
                "zircon-shadow-map-scene-environment-specular-pmrem-cube-view",
            );
        let (environment_irradiance_cube_texture, environment_irradiance_cube_view) =
            create_shadow_environment_cube(
                device,
                "zircon-shadow-map-scene-environment-irradiance-cube",
                "zircon-shadow-map-scene-environment-irradiance-cube-view",
            );
        Self {
            scene_layout: scene_layout.clone(),
            _environment_cube_texture: environment_cube_texture,
            environment_cube_view,
            environment_cube_sampler,
            _environment_brdf_lut_texture: environment_brdf_lut_texture,
            environment_brdf_lut_view,
            _environment_specular_cube_texture: environment_specular_cube_texture,
            environment_specular_cube_view,
            _environment_irradiance_cube_texture: environment_irradiance_cube_texture,
            environment_irradiance_cube_view,
        }
    }

    pub(crate) fn record_atlas_commands_with_attachment_ops<'a>(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
        atlas_view: &wgpu::TextureView,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        slot_passes: &[ShadowAtlasSlotPass],
        frame: &ViewportRenderFrame,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
        mesh_draw_commands: MeshDrawCommandStream<'a>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats {
        if slot_passes.is_empty() || mesh_draw_commands.is_empty() {
            record_depth_only_pass(encoder, pass_name, atlas_view, attachment_ops);
            return MeshDrawReplayStats::default();
        }

        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(device, None, None, None, None);
        let mut wrote_first_slot = false;
        let mut combined = MeshDrawReplayStats::default();
        for slot_pass in slot_passes {
            if slot_pass.rect.width == 0 || slot_pass.rect.height == 0 {
                continue;
            }
            let scene_uniform = scene_uniform_for_view_projection(slot_pass.view_proj);
            let scene_uniform_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zircon-shadow-map-slot-scene-uniform"),
                    contents: bytes_of(&scene_uniform),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            let scene_bind_group = self.create_slot_scene_bind_group(device, &scene_uniform_buffer);

            let atlas_attachment_ops = if wrote_first_slot {
                RenderGraphAttachmentOps::load_store()
            } else {
                attachment_ops
            };
            wrote_first_slot = true;
            let slot_pass_name = format!("{pass_name}.atlas-slot-{}", slot_pass.slot_index);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&slot_pass_name),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: atlas_view,
                    depth_ops: Some(depth_attachment_operations(atlas_attachment_ops, 1.0)),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_viewport(
                slot_pass.rect.x as f32,
                slot_pass.rect.y as f32,
                slot_pass.rect.width as f32,
                slot_pass.rect.height as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(
                slot_pass.rect.x,
                slot_pass.rect.y,
                slot_pass.rect.width,
                slot_pass.rect.height,
            );
            pass.set_bind_group(0, &scene_bind_group, &[]);
            pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);
            let visible_entities = slot_pass
                .view_key
                .and_then(|view_key| visible_shadow_entities_for_view(frame, &view_key));
            add_replay_stats(
                &mut combined,
                self.replay_shadow_command_stream(
                    device,
                    mesh_pipelines,
                    streamer,
                    &mut pass,
                    gpu_scene_bind_group,
                    mesh_draw_commands,
                    visible_entities.as_ref(),
                ),
            );
        }

        if !wrote_first_slot {
            record_depth_only_pass(encoder, pass_name, atlas_view, attachment_ops);
        }
        combined
    }

    fn create_slot_scene_bind_group(
        &self,
        device: &wgpu::Device,
        scene_uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-shadow-map-slot-scene-bind-group"),
            layout: &self.scene_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: scene_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.environment_cube_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.environment_cube_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.environment_brdf_lut_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(
                        &self.environment_specular_cube_view,
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(
                        &self.environment_irradiance_cube_view,
                    ),
                },
            ],
        })
    }

    fn replay_shadow_command_stream<'pass>(
        &self,
        device: &wgpu::Device,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        pass: &mut wgpu::RenderPass<'pass>,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'pass>>,
        mesh_draw_commands: MeshDrawCommandStream<'pass>,
        visible_entities: Option<&BTreeSet<EntityId>>,
    ) -> MeshDrawReplayStats {
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(pass, mesh_draw_commands, |replayer, pass, command| {
            if visible_entities.is_some_and(|entities| !entities.contains(&command.source_entity)) {
                return false;
            }
            match command.pipeline_kind {
                MeshPassPipelineKind::ShadowDepthAlphaMask => {
                    if replayer.should_set_pipeline(
                        command.pipeline_kind,
                        command.pipeline_variant_id,
                    ) {
                        let pipeline = mesh_pipelines
                            .ensure_shadow_pipeline_for_variant(
                                device,
                                streamer,
                                command.pipeline_variant_id,
                            )
                            .expect(
                                "shadow alpha mask command must resolve a cache-backed pipeline variant",
                            );
                        pass.set_pipeline(pipeline);
                    }
                }
                MeshPassPipelineKind::ShadowDepth => {
                    if replayer.should_set_pipeline(
                        command.pipeline_kind,
                        command.pipeline_variant_id,
                    ) {
                        let pipeline = mesh_pipelines
                            .ensure_shadow_pipeline_for_variant(
                                device,
                                streamer,
                                command.pipeline_variant_id,
                            )
                            .expect(
                                "shadow depth command must resolve a cache-backed pipeline variant",
                            );
                        pass.set_pipeline(pipeline);
                    }
                }
                _ => return false,
            }
            replayer.bind_standard_material_if_needed(pass, command);
            replayer.bind_gpu_scene_if_needed(pass, command, gpu_scene_bind_group);
            replayer.bind_geometry_if_needed(pass, command);
            true
        });
        replayer.stats()
    }
}

fn create_shadow_environment_cube(
    device: &wgpu::Device,
    texture_label: &'static str,
    view_label: &'static str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(texture_label),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(view_label),
        format: Some(wgpu::TextureFormat::Rgba16Float),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(6),
    });
    (texture, view)
}

fn create_shadow_environment_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("zircon-shadow-map-scene-environment-cube-sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    })
}

fn create_shadow_environment_brdf_lut(device: &wgpu::Device) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-shadow-map-scene-environment-brdf-lut"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rg16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-shadow-map-scene-environment-brdf-lut-view"),
        format: Some(wgpu::TextureFormat::Rg16Float),
        dimension: Some(wgpu::TextureViewDimension::D2),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(1),
    });
    (texture, view)
}

fn visible_shadow_entities_for_view(
    frame: &ViewportRenderFrame,
    view_key: &VisibilityViewKey,
) -> Option<BTreeSet<EntityId>> {
    let frame_visibility = frame.frame_visibility()?;
    frame_visibility.view(view_key)?;
    Some(frame_visibility.visible_entity_set_for_view(view_key))
}

#[cfg(test)]
fn filter_shadow_commands_for_visible_entities(
    commands: &[crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommand],
    visible_entities: &BTreeSet<EntityId>,
) -> Vec<crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommand> {
    commands
        .iter()
        .filter(|command| visible_entities.contains(&command.source_entity))
        .cloned()
        .collect()
}

fn scene_uniform_for_view_projection(view_proj: Mat4) -> SceneUniform {
    let view_proj = finite_mat4_or_identity(view_proj);
    let view_proj_cols = view_proj.to_cols_array_2d();
    SceneUniform {
        view_proj: view_proj_cols,
        view_proj_unjittered: view_proj_cols,
        inverse_view_proj: finite_mat4_or_identity(view_proj.inverse()).to_cols_array_2d(),
        ambient_color: [0.0, 0.0, 0.0, 1.0],
        previous_view_proj_unjittered: view_proj_cols,
        motion_params: [0.0, 0.0, 0.0, 0.0],
        jitter_params: [0.0, 0.0, 0.0, 0.0],
        camera_world_position: [0.0, 0.0, 0.0, 1.0],
        camera_view_direction: [0.0, 0.0, 1.0, 0.0],
        sky_horizon_color: [0.0, 0.0, 0.0, 1.0],
        sky_zenith_color: [0.0, 0.0, 0.0, 1.0],
        sky_ground_color: [0.0, 0.0, 0.0, 1.0],
        sky_sun_direction: [0.0, 0.0, 0.0, 0.0],
        sky_sun_color_radius: [0.0, 0.0, 0.0, 0.0],
        sky_sun_params: [0.0, 0.0, 0.0, 0.0],
        environment_params: [0.0, 0.0, 0.0, 0.0],
        environment_sample_params: [0.0, 0.0, 0.0, 0.0],
    }
}

fn record_depth_only_pass(
    encoder: &mut wgpu::CommandEncoder,
    pass_name: &str,
    depth_view: &wgpu::TextureView,
    attachment_ops: RenderGraphAttachmentOps,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(pass_name),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(depth_attachment_operations(attachment_ops, 1.0)),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

fn add_replay_stats(total: &mut MeshDrawReplayStats, next: MeshDrawReplayStats) {
    total.draw_call_count = total.draw_call_count.saturating_add(next.draw_call_count);
    total.state_change_count = total
        .state_change_count
        .saturating_add(next.state_change_count);
    total.bind_skip_count = total.bind_skip_count.saturating_add(next.bind_skip_count);
}

fn finite_mat4_or_identity(value: Mat4) -> Mat4 {
    if is_finite_mat4(value) {
        value
    } else {
        Mat4::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::core::framework::render::RenderPhase;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    #[test]
    fn shadow_atlas_view_filter_keeps_only_visible_source_entities() {
        let commands = vec![test_command(11), test_command(22), test_command(33)];
        let visible_entities = [22, 33].into_iter().collect::<BTreeSet<_>>();

        let filtered =
            super::filter_shadow_commands_for_visible_entities(&commands, &visible_entities);

        assert_eq!(
            filtered
                .iter()
                .map(|command| command.source_entity)
                .collect::<Vec<_>>(),
            vec![22, 33]
        );
    }

    #[test]
    fn shadow_atlas_binds_forward_shadow_receiver_layout_slot() {
        let source = include_str!("shadow_map_renderer.rs");

        assert!(source.contains("create_forward_shadow_receiver_bind_group"));
        assert!(source.contains("pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[])"));
        assert!(source.contains("replayer.bind_standard_material_if_needed(pass, command);"));
    }

    #[test]
    fn shadow_atlas_allocates_immutable_scene_uniform_per_slot() {
        let source = include_str!("shadow_map_renderer.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");

        assert!(source.contains("create_buffer_init"));
        assert!(source.contains("create_slot_scene_bind_group"));
        assert!(!source.contains("queue.write_buffer(&self.scene_uniform_buffer"));
    }

    #[test]
    fn shadow_map_scene_bind_group_matches_environment_scene_layout() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let scene_layout_entries = scene_bind_group_layout_entries();
        let scene_layout =
            backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("zircon-test-shadow-map-scene-layout"),
                    entries: &scene_layout_entries,
                });

        let error_scope = backend
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let _renderer = super::ShadowMapRenderer::new(&backend.device, &scene_layout);
        let error = pollster::block_on(error_scope.pop());

        assert!(
            error.is_none(),
            "shadow-map scene bind group should match scene environment layout: {error:?}"
        );
    }

    fn test_command(source_entity: u64) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Shadow,
            MeshPassPipelineKind::ShadowDepth,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
            source_entity,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 0,
                instance_count: 1,
            },
            MeshGeometryHandle::test(source_entity),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_source_entity(source_entity)
    }
}
