use crate::types::EditorOrRuntimeFrame;

use super::super::super::mesh_draw::MeshDraw;
use super::super::super::virtual_geometry_indirect_args_gpu_resources::VirtualGeometryIndirectArgsGpuResources;
use super::super::create_mesh_draw::create_mesh_draw;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::build_mesh_draw_build_context::build_mesh_draw_build_context;
use super::build_shared_indirect_args_buffer::build_shared_indirect_args_buffer;
use super::extend_pending_draws_for_mesh_instance::extend_pending_draws_for_mesh_instance;

pub(crate) struct BuiltMeshDraws {
    pub(crate) draws: Vec<MeshDraw>,
    pub(crate) indirect_segment_count: u32,
    pub(crate) indirect_draw_ref_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pub(crate) indirect_segment_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
}

pub(crate) fn build_mesh_draws(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    virtual_geometry_indirect_args: &VirtualGeometryIndirectArgsGpuResources,
    model_layout: &wgpu::BindGroupLayout,
    streamer: &crate::scene::resources::ResourceStreamer,
    frame: &EditorOrRuntimeFrame,
    virtual_geometry_enabled: bool,
) -> BuiltMeshDraws {
    let build_context = build_mesh_draw_build_context(frame, virtual_geometry_enabled);
    let mut pending_draws = Vec::new();
    for mesh_instance in &frame.scene.scene.meshes {
        extend_pending_draws_for_mesh_instance(
            &mut pending_draws,
            streamer,
            frame,
            &build_context,
            mesh_instance,
        );
    }

    let shared_indirect_args_buffer = virtual_geometry_enabled
        .then(|| {
            build_shared_indirect_args_buffer(
                device,
                encoder,
                virtual_geometry_indirect_args,
                &pending_draws,
            )
        })
        .flatten();
    let indirect_segment_count = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| shared.segment_count)
        .unwrap_or(0);
    let indirect_draw_ref_buffer = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.draw_ref_buffer));
    let indirect_segment_buffer = shared_indirect_args_buffer
        .as_ref()
        .map(|shared| std::sync::Arc::clone(&shared.segment_buffer));
    let shared_indirect_args_buffer = shared_indirect_args_buffer.map(|shared| shared.buffer);
    let indirect_args_stride = std::mem::size_of::<IndexedIndirectArgs>() as u64;

    BuiltMeshDraws {
        draws: pending_draws
            .into_iter()
            .enumerate()
            .map(|(index, pending_draw)| {
                create_mesh_draw(
                    device,
                    model_layout,
                    pending_draw.mesh,
                    pending_draw.texture,
                    pending_draw.pipeline_key,
                    pending_draw.model_matrix,
                    pending_draw.draw_tint,
                    pending_draw.first_index,
                    pending_draw.draw_index_count,
                    shared_indirect_args_buffer.clone(),
                    (index as u64) * indirect_args_stride,
                )
            })
            .collect(),
        indirect_segment_count,
        indirect_draw_ref_buffer,
        indirect_segment_buffer,
    }
}
