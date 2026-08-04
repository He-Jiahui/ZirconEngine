use crate::core::framework::render::{SourceCubemapUploadMip, SOURCE_CUBEMAP_FACE_COUNT};

const INITIAL_STAGING_CAPACITY_BYTES: u64 = 64 * 1024;

#[derive(Clone, Copy)]
struct CubemapUploadCopy {
    target_index: usize,
    source_offset: u64,
    mip_level: u32,
    face_size: u32,
    bytes_per_row: u32,
}

/// Reuses host and GPU staging storage for all prepared environment cubemap uploads in a frame.
pub(super) struct CubemapUploadStagingArena {
    buffer: Option<wgpu::Buffer>,
    capacity_bytes: u64,
    bytes: Vec<u8>,
    copies: Vec<CubemapUploadCopy>,
}

impl Default for CubemapUploadStagingArena {
    fn default() -> Self {
        Self {
            buffer: None,
            capacity_bytes: 0,
            bytes: Vec::new(),
            copies: Vec::new(),
        }
    }
}

impl CubemapUploadStagingArena {
    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uploads: &[Option<(&wgpu::Texture, &[SourceCubemapUploadMip])>],
    ) -> bool {
        self.bytes.clear();
        self.copies.clear();

        for (target_index, upload) in uploads.iter().enumerate() {
            let Some((_, mips)) = upload else {
                continue;
            };
            for mip in *mips {
                let Some(source_offset) = aligned_copy_offset(self.bytes.len()) else {
                    return false;
                };
                self.bytes.resize(source_offset, 0);
                self.bytes.extend_from_slice(mip.bytes());
                let Ok(source_offset) = u64::try_from(source_offset) else {
                    return false;
                };
                self.copies.push(CubemapUploadCopy {
                    target_index,
                    source_offset,
                    mip_level: mip.mip_level(),
                    face_size: mip.face_size(),
                    bytes_per_row: mip.bytes_per_row(),
                });
            }
        }

        if self.copies.is_empty() || !self.ensure_capacity(device) {
            return false;
        }
        let Some(buffer) = self.buffer.as_ref() else {
            return false;
        };

        queue.write_buffer(buffer, 0, self.bytes.as_slice());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-scene-environment-cubemap-upload"),
        });
        for copy in &self.copies {
            let Some((texture, _)) = uploads[copy.target_index].as_ref() else {
                return false;
            };
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: copy.source_offset,
                        bytes_per_row: Some(copy.bytes_per_row),
                        rows_per_image: Some(copy.face_size),
                    },
                },
                wgpu::TexelCopyTextureInfo {
                    texture: *texture,
                    mip_level: copy.mip_level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: copy.face_size,
                    height: copy.face_size,
                    depth_or_array_layers: SOURCE_CUBEMAP_FACE_COUNT as u32,
                },
            );
        }
        queue.submit([encoder.finish()]);
        true
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device) -> bool {
        let Ok(required_bytes) = u64::try_from(self.bytes.len()) else {
            return false;
        };
        if required_bytes <= self.capacity_bytes {
            return true;
        }

        let Some(capacity_bytes) = required_bytes
            .max(INITIAL_STAGING_CAPACITY_BYTES)
            .checked_next_power_of_two()
        else {
            return false;
        };
        self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-scene-environment-cubemap-upload-staging"),
            size: capacity_bytes,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        self.capacity_bytes = capacity_bytes;
        true
    }
}

fn aligned_copy_offset(byte_len: usize) -> Option<usize> {
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    byte_len
        .checked_add(alignment - 1)
        .map(|value| value / alignment * alignment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_offsets_keep_wgpu_row_alignment() {
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        assert_eq!(aligned_copy_offset(0), Some(0));
        assert_eq!(aligned_copy_offset(1), Some(alignment));
        assert_eq!(aligned_copy_offset(alignment), Some(alignment));
    }
}
