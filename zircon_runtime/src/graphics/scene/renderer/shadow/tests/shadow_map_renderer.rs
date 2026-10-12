use std::collections::BTreeSet;

use bytemuck::bytes_of;
use wgpu::util::DeviceExt;

use crate::core::framework::render::RenderPhase;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle, MeshPassPipelineKind,
    MeshPipelineVariantId,
};
use crate::graphics::scene::scene_renderer::primitives::SceneEnvironmentSh9;

#[test]
fn shadow_atlas_view_filter_keeps_only_visible_source_entities() {
    let commands = vec![test_command(11), test_command(22), test_command(33)];
    let visible_entities = [22, 33].into_iter().collect::<BTreeSet<_>>();

    let filtered = super::filter_shadow_commands_for_visible_entities(&commands, &visible_entities);

    assert_eq!(
        filtered
            .iter()
            .map(|command| command.source_entity)
            .collect::<Vec<_>>(),
        vec![22, 33]
    );
}

#[test]
fn shadow_view_filter_disables_global_indirect_batches_before_replay() {
    let source = include_str!("../shadow_map_renderer.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source section should exist");

    assert!(source.contains("if visible_entities.is_some()"));
    assert!(source.contains("mesh_draw_commands.without_indirect()"));
}

#[test]
fn shadow_atlas_binds_forward_shadow_receiver_layout_slot() {
    let source = include_str!("../shadow_map_renderer.rs");

    assert!(source.contains("create_forward_shadow_receiver_bind_group"));
    assert!(source.contains("pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[])"));
    assert!(source.contains("replayer.bind_standard_material_if_needed(pass, command);"));
}

#[test]
fn shadow_atlas_uses_one_persistent_aligned_scene_uniform_workspace() {
    let production = include_str!("../shadow_map_renderer.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source section should exist");
    let record = production
        .split("fn record_atlas_commands_with_attachment_ops")
        .nth(1)
        .and_then(|source| source.split("fn replay_shadow_command_stream").next())
        .expect("shadow atlas recording must remain bounded");
    let prepare = production
        .split("fn prepare_slot_scene_uploads")
        .nth(1)
        .and_then(|source| source.split("fn record_atlas_commands").next())
        .expect("shadow slot preparation must remain bounded");

    assert!(!record.contains("queue:"));
    assert!(!record.contains("create_buffer_init"));
    assert!(!record.contains("create_bind_group"));
    assert!(prepare.contains("checked_next_power_of_two()"));
    assert!(prepare.contains("min_uniform_buffer_offset_alignment"));
    assert!(prepare.contains("WgpuBufferUpload::new(buffer, 0, payload, 0..payload_len)"));
    assert!(prepare.contains("wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST"));
}

#[test]
fn shadow_environment_bindings_are_leased_without_private_resource_creation() {
    let production = include_str!("../shadow_map_renderer.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source section should exist");
    let constructor = production
        .split("impl ShadowMapRenderer")
        .nth(1)
        .and_then(|source| source.split("fn new(").nth(1))
        .and_then(|source| source.split("fn prepare_slot_scene_uploads").next())
        .expect("shadow constructor must remain bounded");
    let slot_bindings = production
        .split("fn create_slot_scene_bind_group")
        .nth(1)
        .and_then(|source| source.split("fn align_up_u64").next())
        .expect("shadow scene binding must remain bounded");

    assert!(!constructor.contains("device:"));
    assert!(!constructor.contains("create_texture"));
    assert!(!constructor.contains("create_sampler"));
    assert!(!constructor.contains("create_buffer"));
    assert_eq!(
        slot_bindings
            .matches("&environment.black_cube_view")
            .count(),
        3
    );
    assert!(slot_bindings.contains("&environment.brdf_lut_view"));
    assert!(slot_bindings.contains("&environment.sampler"));
    assert!(slot_bindings.contains("environment.sh9_buffer.as_entire_binding()"));
}

#[test]
fn shadow_map_scene_bind_group_matches_environment_scene_layout() {
    let Ok(backend) = RenderBackend::new_offscreen() else {
        return;
    };
    let scene_layout_entries = scene_bind_group_layout_entries();
    let scene_layout = backend
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-shadow-map-scene-layout"),
            entries: &scene_layout_entries,
        });

    let error_scope = backend
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let mut renderer = super::ShadowMapRenderer::new(
        &scene_layout,
        test_shadow_environment_binding_lease(&backend.device),
    );
    renderer
        .ensure_slot_scene_capacity(&backend.device, 1)
        .expect("shadow slot workspace should support one scene uniform");
    let _scene_bind_group = renderer
        .slot_scene_workspace
        .bind_groups
        .first()
        .expect("prepared shadow slot workspace should publish one bind group");
    let error = pollster::block_on(error_scope.pop());

    assert!(
        error.is_none(),
        "shadow-map scene bind group should match scene environment layout: {error:?}"
    );
}

#[test]
fn shadow_scene_uniform_stride_alignment_is_checked() {
    assert_eq!(super::align_up_u64(432, 256), Some(512));
    assert_eq!(super::align_up_u64(512, 256), Some(512));
    assert_eq!(super::align_up_u64(u64::MAX, 256), None);
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

fn test_shadow_environment_binding_lease(
    device: &wgpu::Device,
) -> super::ShadowSceneEnvironmentBindingLease {
    let black_cube_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-test-shadow-black-cube"),
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
    let black_cube_view = black_cube_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-test-shadow-black-cube-view"),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
    let brdf_lut_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-test-shadow-brdf-lut"),
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
    let brdf_lut_view = brdf_lut_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sh9_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-test-shadow-environment-sh9"),
        contents: bytes_of(&SceneEnvironmentSh9::default()),
        usage: wgpu::BufferUsages::UNIFORM,
    });

    super::ShadowSceneEnvironmentBindingLease::new(
        black_cube_texture,
        black_cube_view,
        sampler,
        brdf_lut_texture,
        brdf_lut_view,
        sh9_buffer,
    )
}
