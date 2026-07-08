// Staged Plan 11 acquisition helpers land before the runtime bake scheduler consumes them.

use std::sync::mpsc;

use crate::core::framework::render::{source_cubemap_mip_size, SOURCE_CUBEMAP_FACE_COUNT};
use crate::graphics::debug_markers::{insert_marker, RENDERDOC_MARKER_READBACK};
use crate::graphics::types::GraphicsError;

const RGBA16FLOAT_BYTES_PER_TEXEL: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rgba16FloatTextureRegionReadback {
    pub mip_level: u32,
    pub origin: wgpu::Origin3d,
    pub size: wgpu::Extent3d,
    pub label: &'static str,
}

pub(crate) fn read_texture_rgba16float_cube_mip_chain(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    face_size: u32,
    mip_count: u32,
) -> Result<Vec<u8>, GraphicsError> {
    let face_size = face_size.max(1);
    let mip_count = mip_count.max(1);
    let mut bytes = Vec::with_capacity(rgba16float_cube_mip_chain_size_bytes(face_size, mip_count));
    for face in 0..SOURCE_CUBEMAP_FACE_COUNT as u32 {
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            bytes.extend_from_slice(&read_texture_rgba16float_region(
                device,
                queue,
                texture,
                Rgba16FloatTextureRegionReadback {
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face,
                    },
                    size: wgpu::Extent3d {
                        width: mip_size,
                        height: mip_size,
                        depth_or_array_layers: 1,
                    },
                    label: "zircon-readback-rgba16float-cube-mip",
                },
            )?);
        }
    }
    Ok(bytes)
}

pub(crate) fn read_texture_rgba16float_region(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    region: Rgba16FloatTextureRegionReadback,
) -> Result<Vec<u8>, GraphicsError> {
    let width = region.size.width.max(1);
    let height = region.size.height.max(1);
    let depth_or_array_layers = region.size.depth_or_array_layers.max(1);
    let unpadded_bytes_per_row = width * RGBA16FLOAT_BYTES_PER_TEXEL;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let rows_per_image = height;
    let buffer_size =
        padded_bytes_per_row as u64 * rows_per_image as u64 * depth_or_array_layers as u64;
    let buffer_label = format!("{}-buffer", region.label);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&buffer_label),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let encoder_label = format!("{}-encoder", region.label);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(&encoder_label),
    });
    insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: region.mip_level,
            origin: region.origin,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(rows_per_image),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
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
    let rgba = strip_padded_rgba16float_region_rows(
        &mapped,
        [width, height, depth_or_array_layers],
        unpadded_bytes_per_row as usize,
        padded_bytes_per_row as usize,
    );
    drop(mapped);
    buffer.unmap();

    Ok(rgba)
}

fn rgba16float_cube_mip_chain_size_bytes(face_size: u32, mip_count: u32) -> usize {
    let mut total = 0;
    for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip_level in 0..mip_count.max(1) {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            total += mip_size as usize * mip_size as usize * RGBA16FLOAT_BYTES_PER_TEXEL as usize;
        }
    }
    total
}

fn strip_padded_rgba16float_region_rows(
    mapped: &[u8],
    size: [u32; 3],
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
) -> Vec<u8> {
    let row_count = size[1] as usize;
    let layer_count = size[2] as usize;
    let mut rgba = vec![0_u8; size[0] as usize * row_count * layer_count * 8];
    for layer in 0..layer_count {
        for row in 0..row_count {
            let source_offset =
                layer * row_count * padded_bytes_per_row + row * padded_bytes_per_row;
            let target_offset =
                layer * row_count * unpadded_bytes_per_row + row * unpadded_bytes_per_row;
            rgba[target_offset..target_offset + unpadded_bytes_per_row]
                .copy_from_slice(&mapped[source_offset..source_offset + unpadded_bytes_per_row]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::{
        rgba16float_cube_mip_chain_size_bytes, strip_padded_rgba16float_region_rows,
        RGBA16FLOAT_BYTES_PER_TEXEL,
    };

    #[test]
    fn rgba16float_region_readback_strips_row_padding_per_layer() {
        let size = [2, 2, 2];
        let unpadded = 16;
        let padded = 32;
        let mut mapped = vec![0_u8; padded * size[1] as usize * size[2] as usize];
        for (index, row) in mapped.chunks_exact_mut(padded).enumerate() {
            row[..unpadded].fill(index as u8 + 1);
            row[unpadded..].fill(255);
        }

        let stripped = strip_padded_rgba16float_region_rows(&mapped, size, unpadded, padded);

        assert_eq!(stripped.len(), 64);
        assert_eq!(&stripped[0..16], &[1_u8; 16]);
        assert_eq!(&stripped[16..32], &[2_u8; 16]);
        assert_eq!(&stripped[32..48], &[3_u8; 16]);
        assert_eq!(&stripped[48..64], &[4_u8; 16]);
        assert!(!stripped.contains(&255));
    }

    #[test]
    fn rgba16float_cube_mip_chain_size_matches_face_major_artifact_layout() {
        assert_eq!(
            rgba16float_cube_mip_chain_size_bytes(4, 3),
            6 * (16 + 4 + 1) * RGBA16FLOAT_BYTES_PER_TEXEL as usize
        );
    }
}
