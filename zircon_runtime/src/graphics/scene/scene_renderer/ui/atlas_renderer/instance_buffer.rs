use crate::text::atlas::render_gpu_plan::GlyphAtlasGpuInstance;

use super::state::GlyphAtlasBitmapRendererDrawPass;

const GLYPH_ATLAS_MIN_INSTANCE_BUFFER_CAPACITY_BYTES: u64 = 4 * 1024;

pub(super) fn glyph_atlas_bitmap_renderer_write_instance_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    draw_pass: &mut GlyphAtlasBitmapRendererDrawPass,
    instances: &[GlyphAtlasGpuInstance],
) -> (usize, usize) {
    if instances.is_empty() {
        // Keep capacity across empty active passes; explicit idle releases all retained buffers.
        return (
            glyph_atlas_bitmap_renderer_instance_buffer_capacity_byte_len(
                draw_pass.instance_buffer_capacity_bytes,
            ),
            0,
        );
    }

    let required_byte_len = std::mem::size_of_val(instances);
    let requires_reallocation = draw_pass.instance_buffer.is_none()
        || glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(
            draw_pass.instance_buffer_capacity_bytes,
            required_byte_len,
        );
    if requires_reallocation {
        let capacity_bytes =
            glyph_atlas_bitmap_renderer_instance_buffer_capacity(required_byte_len);
        draw_pass.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-screen-space-ui-glyph-atlas-instances"),
            size: capacity_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        draw_pass.instance_buffer_capacity_bytes = capacity_bytes;
    }
    if let Some(instance_buffer) = draw_pass.instance_buffer.as_ref() {
        // Draw commands bound later cap the instance range, so stale tail bytes stay unreachable.
        queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(instances));
    }
    (
        glyph_atlas_bitmap_renderer_instance_buffer_capacity_byte_len(
            draw_pass.instance_buffer_capacity_bytes,
        ),
        if requires_reallocation { 1 } else { 0 },
    )
}

pub(super) fn glyph_atlas_bitmap_renderer_instance_buffer_capacity(
    required_byte_len: usize,
) -> u64 {
    if required_byte_len == 0 {
        return 0;
    }
    let required_byte_len =
        (required_byte_len as u64).max(GLYPH_ATLAS_MIN_INSTANCE_BUFFER_CAPACITY_BYTES);
    match required_byte_len.checked_next_power_of_two() {
        Some(capacity_bytes) => capacity_bytes,
        None => required_byte_len,
    }
}

pub(super) fn glyph_atlas_bitmap_renderer_instance_buffer_requires_reallocation(
    capacity_bytes: u64,
    required_byte_len: usize,
) -> bool {
    required_byte_len > 0 && capacity_bytes < required_byte_len as u64
}

fn glyph_atlas_bitmap_renderer_instance_buffer_capacity_byte_len(capacity_bytes: u64) -> usize {
    capacity_bytes.min(usize::MAX as u64) as usize
}
