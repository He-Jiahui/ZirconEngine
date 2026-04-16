use std::sync::mpsc;

use zircon_math::UVec2;

use crate::types::GraphicsError;

pub(crate) fn read_texture_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: UVec2,
) -> Result<Vec<u8>, GraphicsError> {
    let bytes_per_pixel = 4_u32;
    let unpadded_bytes_per_row = size.x * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer_size = padded_bytes_per_row as u64 * size.y as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-encoder"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(size.y),
            },
        },
        wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);

    let slice = buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;
    receiver
        .recv()
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?
        .map_err(|error| GraphicsError::BufferMap(error.to_string()))?;

    let mapped = slice.get_mapped_range();
    let mut rgba = vec![0_u8; (size.x * size.y * 4) as usize];
    for row in 0..size.y as usize {
        let source_offset = row * padded_bytes_per_row as usize;
        let target_offset = row * unpadded_bytes_per_row as usize;
        rgba[target_offset..target_offset + unpadded_bytes_per_row as usize].copy_from_slice(
            &mapped[source_offset..source_offset + unpadded_bytes_per_row as usize],
        );
    }
    drop(mapped);
    buffer.unmap();

    Ok(rgba)
}
