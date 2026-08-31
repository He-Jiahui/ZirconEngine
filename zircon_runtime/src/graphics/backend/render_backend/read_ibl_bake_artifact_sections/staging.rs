use std::sync::mpsc;

use crate::core::framework::render::{SOURCE_CUBEMAP_FACE_COUNT, source_cubemap_mip_size};

const RGBA16FLOAT_BYTES_PER_TEXEL: u32 = 8;

pub(super) struct BufferReadback {
    buffer: wgpu::Buffer,
}

impl BufferReadback {
    pub(super) fn encode(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Buffer,
        source_offset: u64,
        byte_len: u64,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_len,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(source, source_offset, &buffer, 0, byte_len);
        Self { buffer }
    }

    pub(super) fn map_async(&self, sender: mpsc::Sender<Result<(), String>>) {
        self.buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result.map_err(|error| error.to_string()));
            });
    }

    pub(super) fn into_bytes(self) -> Vec<u8> {
        let slice = self.buffer.slice(..);
        let mapped = slice.get_mapped_range();
        let bytes = mapped.to_vec();
        drop(mapped);
        self.buffer.unmap();
        bytes
    }

    pub(super) fn unmap(&self) {
        self.buffer.unmap();
    }
}

pub(super) struct CubeMipChainReadback {
    buffer: wgpu::Buffer,
    face_size: u32,
    mip_count: u32,
}

impl CubeMipChainReadback {
    pub(super) fn encode(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        face_size: u32,
        mip_count: u32,
        label: &'static str,
    ) -> Self {
        let face_size = face_size.max(1);
        let mip_count = mip_count.max(1);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: cube_mip_staging_size_bytes(face_size, mip_count),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
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
        Self {
            buffer,
            face_size,
            mip_count,
        }
    }

    pub(super) fn map_async(&self, sender: mpsc::Sender<Result<(), String>>) {
        self.buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result.map_err(|error| error.to_string()));
            });
    }

    pub(super) fn into_bytes(self) -> Vec<u8> {
        let slice = self.buffer.slice(..);
        let mapped = slice.get_mapped_range();
        let bytes = strip_padded_cube_mip_chain(&mapped, self.face_size, self.mip_count);
        drop(mapped);
        self.buffer.unmap();
        bytes
    }

    pub(super) fn unmap(&self) {
        self.buffer.unmap();
    }
}

fn cube_mip_staging_size_bytes(face_size: u32, mip_count: u32) -> u64 {
    let mut total = 0;
    for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let unpadded_bytes_per_row = mip_size * RGBA16FLOAT_BYTES_PER_TEXEL;
            let padded_bytes_per_row = unpadded_bytes_per_row
                .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            total += padded_bytes_per_row as u64 * mip_size as u64;
        }
    }
    total
}

pub(super) fn strip_padded_cube_mip_chain(
    mapped: &[u8],
    face_size: u32,
    mip_count: u32,
) -> Vec<u8> {
    let byte_len = (0..SOURCE_CUBEMAP_FACE_COUNT)
        .flat_map(|_| 0..mip_count)
        .map(|mip_level| {
            let mip_size = source_cubemap_mip_size(face_size, mip_level) as usize;
            mip_size * mip_size * RGBA16FLOAT_BYTES_PER_TEXEL as usize
        })
        .sum();
    let mut bytes = vec![0; byte_len];
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
                bytes[target_offset..target_offset + unpadded_bytes_per_row as usize]
                    .copy_from_slice(
                        &mapped[source_offset..source_offset + unpadded_bytes_per_row as usize],
                    );
            }
            staging_offset += padded_bytes_per_row as usize * mip_size as usize;
            output_offset += unpadded_bytes_per_row as usize * mip_size as usize;
        }
    }
    bytes
}
