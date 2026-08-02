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
mod tests {
    #[derive(Clone, Copy)]
    struct TestDraw {
        transparent: bool,
        indirect: bool,
    }

    #[test]
    fn execution_indirect_indices_keep_mesh_order_without_deferred_lighting() {
        let draws = test_draws();

        let indices = super::collect_execution_indirect_draw_indices(
            &draws,
            false,
            |draw| draw.transparent,
            |draw| draw.indirect,
        );

        assert_eq!(indices, vec![0, 1, 4]);
    }

    #[test]
    fn execution_indirect_indices_submit_opaque_before_transparent_with_deferred_lighting() {
        let draws = test_draws();

        let indices = super::collect_execution_indirect_draw_indices(
            &draws,
            true,
            |draw| draw.transparent,
            |draw| draw.indirect,
        );

        assert_eq!(indices, vec![1, 4, 0]);
    }

    fn test_draws() -> [TestDraw; 5] {
        [
            TestDraw {
                transparent: true,
                indirect: true,
            },
            TestDraw {
                transparent: false,
                indirect: true,
            },
            TestDraw {
                transparent: false,
                indirect: false,
            },
            TestDraw {
                transparent: true,
                indirect: false,
            },
            TestDraw {
                transparent: false,
                indirect: true,
            },
        ]
    }
}
