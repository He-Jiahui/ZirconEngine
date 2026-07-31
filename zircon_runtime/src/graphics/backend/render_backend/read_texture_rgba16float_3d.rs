use std::sync::mpsc;

use crate::graphics::debug_markers::{RENDERDOC_MARKER_READBACK, insert_marker};
use crate::graphics::types::GraphicsError;

pub(crate) fn read_texture_rgba16float_3d(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: [u32; 3],
) -> Result<Vec<u8>, GraphicsError> {
    let bytes_per_pixel = 8_u32;
    let unpadded_bytes_per_row = size[0] * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let rows_per_image = size[1].max(1);
    let buffer_size = padded_bytes_per_row as u64 * rows_per_image as u64 * size[2].max(1) as u64;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-rgba16float-3d"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-rgba16float-3d-encoder"),
    });
    insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(rows_per_image),
            },
        },
        wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: size[2],
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
    let rgba = strip_padded_rgba16float_3d_rows(
        &mapped,
        size,
        unpadded_bytes_per_row as usize,
        padded_bytes_per_row as usize,
    );
    drop(mapped);
    buffer.unmap();

    Ok(rgba)
}

fn strip_padded_rgba16float_3d_rows(
    mapped: &[u8],
    size: [u32; 3],
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
) -> Vec<u8> {
    let row_count = size[1] as usize;
    let slice_count = size[2] as usize;
    let mut rgba = vec![0_u8; size[0] as usize * row_count * slice_count * 8];
    for slice in 0..slice_count {
        for row in 0..row_count {
            let source_offset =
                slice * row_count * padded_bytes_per_row + row * padded_bytes_per_row;
            let target_offset =
                slice * row_count * unpadded_bytes_per_row + row * unpadded_bytes_per_row;
            rgba[target_offset..target_offset + unpadded_bytes_per_row]
                .copy_from_slice(&mapped[source_offset..source_offset + unpadded_bytes_per_row]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::strip_padded_rgba16float_3d_rows;

    #[test]
    fn rgba16float_3d_readback_strips_row_padding_per_slice() {
        let size = [2, 2, 2];
        let unpadded = 16;
        let padded = 32;
        let mut mapped = vec![0_u8; padded * size[1] as usize * size[2] as usize];
        for (index, row) in mapped.chunks_exact_mut(padded).enumerate() {
            row[..unpadded].fill(index as u8 + 1);
            row[unpadded..].fill(255);
        }

        let stripped = strip_padded_rgba16float_3d_rows(&mapped, size, unpadded, padded);

        assert_eq!(stripped.len(), 64);
        assert_eq!(&stripped[0..16], &[1_u8; 16]);
        assert_eq!(&stripped[16..32], &[2_u8; 16]);
        assert_eq!(&stripped[32..48], &[3_u8; 16]);
        assert_eq!(&stripped[48..64], &[4_u8; 16]);
        assert!(!stripped.contains(&255));
    }
}
