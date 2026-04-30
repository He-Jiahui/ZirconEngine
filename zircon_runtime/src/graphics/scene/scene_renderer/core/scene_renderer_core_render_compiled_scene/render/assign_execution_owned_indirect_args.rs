use std::sync::Arc;

use crate::graphics::scene::scene_renderer::mesh::MeshDraw;

const INDIRECT_ARGS_WORD_COUNT: u64 = 5;
const INDIRECT_ARGS_STRIDE_BYTES: u64 =
    (std::mem::size_of::<u32>() as u64) * INDIRECT_ARGS_WORD_COUNT;

pub(super) fn assign_execution_owned_indirect_args(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    mesh_draws: &mut [MeshDraw],
    deferred_lighting_enabled: bool,
) -> Option<Arc<wgpu::Buffer>> {
    let mut execution_draw_indices = mesh_draws
        .iter()
        .enumerate()
        .filter_map(|(draw_index, draw)| {
            (!deferred_lighting_enabled || !draw.is_transparent()).then_some(draw_index)
        })
        .collect::<Vec<_>>();
    if deferred_lighting_enabled {
        execution_draw_indices.extend(
            mesh_draws
                .iter()
                .enumerate()
                .filter_map(|(draw_index, draw)| draw.is_transparent().then_some(draw_index)),
        );
    }

    let indirect_execution_draw_indices = execution_draw_indices
        .into_iter()
        .filter(|draw_index| mesh_draws[*draw_index].uses_indirect_draw())
        .collect::<Vec<_>>();
    if indirect_execution_draw_indices.is_empty() {
        return None;
    }

    let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-execution-args"),
        size: (indirect_execution_draw_indices.len() as u64) * INDIRECT_ARGS_STRIDE_BYTES,
        usage: wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    }));

    for (execution_index, draw_index) in indirect_execution_draw_indices.iter().copied().enumerate()
    {
        let draw = &mesh_draws[draw_index];
        let Some(source_buffer) = draw.indirect_args_buffer() else {
            continue;
        };
        encoder.copy_buffer_to_buffer(
            source_buffer,
            draw.indirect_args_offset(),
            &buffer,
            (execution_index as u64) * INDIRECT_ARGS_STRIDE_BYTES,
            INDIRECT_ARGS_STRIDE_BYTES,
        );
    }

    for (execution_index, draw_index) in indirect_execution_draw_indices.into_iter().enumerate() {
        let draw = &mut mesh_draws[draw_index];
        draw.assign_execution_owned_indirect_args(
            Arc::clone(&buffer),
            (execution_index as u64) * INDIRECT_ARGS_STRIDE_BYTES,
        );
    }

    Some(buffer)
}
