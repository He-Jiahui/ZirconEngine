use zircon_math::UVec2;

use super::super::texture_extent::texture_extent;

pub(in crate::scene::scene_renderer::history) fn clear_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: UVec2,
    rgba: &[u8; 4],
) {
    let texel_count = size.x.max(1) as usize * size.y.max(1) as usize;
    let mut data = Vec::with_capacity(texel_count * 4);
    for _ in 0..texel_count {
        data.extend_from_slice(rgba);
    }
    queue.write_texture(
        texture.as_image_copy(),
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size.x.max(1) * 4),
            rows_per_image: Some(size.y.max(1)),
        },
        texture_extent(size),
    );
}
