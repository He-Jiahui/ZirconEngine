use std::sync::Arc;

use crate::graphics::scene::scene_renderer::mesh::MeshDraw;

const INDIRECT_ARGS_WORD_COUNT: u64 = 5;
const INDIRECT_ARGS_STRIDE_BYTES: u64 =
    (std::mem::size_of::<u32>() as u64) * INDIRECT_ARGS_WORD_COUNT;

/// 为本次执行复制间接绘制参数，隔离来源缓冲区的后续更新；调用方须保持返回缓冲区活到提交。
/// deferred 模式按不透明再透明的 scene-pass 顺序分配偏移。
pub(super) fn assign_execution_owned_indirect_args(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    mesh_draws: &mut [MeshDraw],
    deferred_lighting_enabled: bool,
) -> Option<Arc<wgpu::Buffer>> {
    let indirect_execution_draw_indices = collect_execution_indirect_draw_indices(
        mesh_draws,
        deferred_lighting_enabled,
        |draw| draw.is_transparent(),
        |draw| draw.uses_indirect_draw(),
    );
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

    for (execution_index, draw_index) in indirect_execution_draw_indices.into_iter().enumerate() {
        let execution_offset = (execution_index as u64) * INDIRECT_ARGS_STRIDE_BYTES;
        {
            let draw = &mesh_draws[draw_index];
            if let Some(source_buffer) = draw.indirect_args_buffer() {
                encoder.copy_buffer_to_buffer(
                    source_buffer,
                    draw.indirect_args_offset(),
                    &buffer,
                    execution_offset,
                    INDIRECT_ARGS_STRIDE_BYTES,
                );
            }
        }
        mesh_draws[draw_index]
            .assign_execution_owned_indirect_args(Arc::clone(&buffer), execution_offset);
    }

    Some(buffer)
}

/// Preserves scene-pass execution order while filtering direct draws before
/// materializing indices, avoiding a second full-frame index vector.
fn collect_execution_indirect_draw_indices<T>(
    draws: &[T],
    deferred_lighting_enabled: bool,
    is_transparent: impl Fn(&T) -> bool,
    uses_indirect_draw: impl Fn(&T) -> bool,
) -> Vec<usize> {
    let mut indices = Vec::with_capacity(draws.len());
    for (draw_index, draw) in draws.iter().enumerate() {
        if (!deferred_lighting_enabled || !is_transparent(draw)) && uses_indirect_draw(draw) {
            indices.push(draw_index);
        }
    }
    if deferred_lighting_enabled {
        for (draw_index, draw) in draws.iter().enumerate() {
            if uses_indirect_draw(draw) && is_transparent(draw) {
                indices.push(draw_index);
            }
        }
    }
    indices
}

#[cfg(test)]
#[path = "tests/assign_execution_owned_indirect_args.rs"]
mod tests;
