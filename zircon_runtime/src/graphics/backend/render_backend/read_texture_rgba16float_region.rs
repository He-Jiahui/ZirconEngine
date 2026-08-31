// Staged Plan 11 acquisition helpers land before the runtime bake scheduler consumes them.

use std::sync::mpsc;

#[cfg(test)]
use crate::core::framework::render::{SOURCE_CUBEMAP_FACE_COUNT, source_cubemap_mip_size};
use crate::graphics::debug_markers::{RENDERDOC_MARKER_READBACK, insert_marker};
use crate::graphics::types::GraphicsError;

const RGBA16FLOAT_BYTES_PER_TEXEL: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rgba16FloatTextureRegionReadback {
    pub mip_level: u32,
    pub origin: wgpu::Origin3d,
    pub size: wgpu::Extent3d,
    pub label: &'static str,
}

#[cfg(test)]
pub(crate) fn read_texture_rgba16float_cube_mip_chain(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    face_size: u32,
    mip_count: u32,
) -> Result<Vec<u8>, GraphicsError> {
    let face_size = face_size.max(1);
    let mip_count = mip_count.max(1);
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-rgba16float-cube-mip-chain-buffer"),
        size: rgba16float_cube_mip_staging_size_bytes(face_size, mip_count),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-rgba16float-cube-mip-chain-encoder"),
    });
    insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
    let mut staging_offset = 0;
    for face in 0..SOURCE_CUBEMAP_FACE_COUNT as u32 {
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let unpadded_bytes_per_row = mip_size * RGBA16FLOAT_BYTES_PER_TEXEL;
            let padded_bytes_per_row = unpadded_bytes_per_row
                .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: staging_offset,
                        bytes_per_row: Some(padded_bytes_per_row),
                        rows_per_image: Some(mip_size),
                    },
                },
                wgpu::Extent3d {
                    width: mip_size,
                    height: mip_size,
                    depth_or_array_layers: 1,
                },
            );
            staging_offset += padded_bytes_per_row as u64 * mip_size as u64;
        }
    }
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
    let bytes = strip_padded_rgba16float_cube_mip_chain(&mapped, face_size, mip_count);
    drop(mapped);
    buffer.unmap();

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

#[cfg(test)]
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

#[cfg(test)]
fn rgba16float_cube_mip_staging_size_bytes(face_size: u32, mip_count: u32) -> u64 {
    let mut total = 0;
    for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip_level in 0..mip_count.max(1) {
            let mip_size = source_cubemap_mip_size(face_size.max(1), mip_level);
            let unpadded_bytes_per_row = mip_size * RGBA16FLOAT_BYTES_PER_TEXEL;
            let padded_bytes_per_row = unpadded_bytes_per_row
                .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            total += padded_bytes_per_row as u64 * mip_size as u64;
        }
    }
    total
}

#[cfg(test)]
fn strip_padded_rgba16float_cube_mip_chain(
    mapped: &[u8],
    face_size: u32,
    mip_count: u32,
) -> Vec<u8> {
    let face_size = face_size.max(1);
    let mip_count = mip_count.max(1);
    let mut rgba = vec![0_u8; rgba16float_cube_mip_chain_size_bytes(face_size, mip_count)];
    let mut staging_offset = 0;
    let mut output_offset = 0;
    for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let unpadded_bytes_per_row = mip_size * RGBA16FLOAT_BYTES_PER_TEXEL;
            let padded_bytes_per_row = unpadded_bytes_per_row
                .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            for row in 0..mip_size as usize {
                let source_offset = staging_offset + row * padded_bytes_per_row as usize;
                let target_offset = output_offset + row * unpadded_bytes_per_row as usize;
                rgba[target_offset..target_offset + unpadded_bytes_per_row as usize]
                    .copy_from_slice(
                        &mapped[source_offset..source_offset + unpadded_bytes_per_row as usize],
                    );
            }
            staging_offset += padded_bytes_per_row as usize * mip_size as usize;
            output_offset += unpadded_bytes_per_row as usize * mip_size as usize;
        }
    }
    rgba
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
        RGBA16FLOAT_BYTES_PER_TEXEL, rgba16float_cube_mip_chain_size_bytes,
        rgba16float_cube_mip_staging_size_bytes, strip_padded_rgba16float_cube_mip_chain,
        strip_padded_rgba16float_region_rows,
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

    #[test]
    fn rgba16float_cube_mip_readback_batches_all_regions_in_one_staging_layout() {
        assert_eq!(rgba16float_cube_mip_staging_size_bytes(4, 3), 10_752);

        let mut mapped = vec![0_u8; 10_752];
        let mut staging_offset = 0;
        let mut value = 1_u8;
        for _face in 0..6 {
            for mip_size in [4_u32, 2, 1] {
                let padded_bytes_per_row = 256_usize;
                for row in 0..mip_size as usize {
                    let row_offset = staging_offset + row * padded_bytes_per_row;
                    mapped[row_offset..row_offset + mip_size as usize * 8].fill(value);
                }
                staging_offset += padded_bytes_per_row * mip_size as usize;
                value = value.wrapping_add(1);
            }
        }

        let stripped = strip_padded_rgba16float_cube_mip_chain(&mapped, 4, 3);

        assert_eq!(stripped.len(), 1_008);
        assert_eq!(&stripped[..128], &[1_u8; 128]);
        assert_eq!(&stripped[128..160], &[2_u8; 32]);
        assert_eq!(&stripped[160..168], &[3_u8; 8]);
        assert_eq!(&stripped[840..968], &[16_u8; 128]);
        assert_eq!(&stripped[968..1_000], &[17_u8; 32]);
        assert_eq!(&stripped[1_000..], &[18_u8; 8]);
    }

    #[test]
    fn synchronous_cube_mip_chain_readback_owner_is_test_only() {
        let source = include_str!("read_texture_rgba16float_region.rs");
        let backend_root = include_str!("mod.rs");
        let graphics_backend_root = include_str!("../mod.rs");

        assert!(
            source.contains("#[cfg(test)]\npub(crate) fn read_texture_rgba16float_cube_mip_chain(")
        );
        assert!(backend_root.contains(
            "#[cfg(test)]\npub(crate) use read_texture_rgba16float_region::{\n    read_texture_rgba16float_cube_mip_chain, read_texture_rgba16float_region,"
        ));
        assert!(graphics_backend_root.contains(
            "#[cfg(test)]\npub(crate) use render_backend::{\n    read_texture_rgba16float_cube_mip_chain, read_texture_rgba16float_region,"
        ));
        assert!(backend_root.contains("#[cfg(test)]\nmod read_texture_rgba16float_region;"));
    }
}
