use std::fmt;
use std::sync::Arc;

use crate::core::framework::render::{SOURCE_CUBEMAP_FACE_COUNT, SourceCubemapUploadMip};
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CubemapUploadStagingError {
    EmptyUpload,
    OffsetOverflow,
    ByteLengthOverflow,
    CapacityOverflow,
    MissingTarget,
    InvalidUploadRange,
}

impl fmt::Display for CubemapUploadStagingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyUpload => "environment cubemap staging upload is empty",
            Self::OffsetOverflow => "environment cubemap staging offset overflowed",
            Self::ByteLengthOverflow => "environment cubemap staging byte length overflowed",
            Self::CapacityOverflow => "environment cubemap staging capacity overflowed",
            Self::MissingTarget => "environment cubemap staging target is missing",
            Self::InvalidUploadRange => "environment cubemap staging upload range is invalid",
        })
    }
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
    pub(super) fn encode(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uploads: &[Option<(&wgpu::Texture, &[SourceCubemapUploadMip])>],
        frame_uploads: &mut WgpuBufferUploadBatch,
    ) -> Result<(), CubemapUploadStagingError> {
        self.bytes.clear();
        self.copies.clear();

        for (target_index, upload) in uploads.iter().enumerate() {
            let Some((_, mips)) = upload else {
                continue;
            };
            for mip in *mips {
                let source_offset = aligned_copy_offset(self.bytes.len())
                    .ok_or(CubemapUploadStagingError::OffsetOverflow)?;
                self.bytes.resize(source_offset, 0);
                self.bytes.extend_from_slice(mip.bytes());
                let source_offset = u64::try_from(source_offset)
                    .map_err(|_| CubemapUploadStagingError::OffsetOverflow)?;
                self.copies.push(CubemapUploadCopy {
                    target_index,
                    source_offset,
                    mip_level: mip.mip_level(),
                    face_size: mip.face_size(),
                    bytes_per_row: mip.bytes_per_row(),
                });
            }
        }

        if self.copies.is_empty() {
            return Err(CubemapUploadStagingError::EmptyUpload);
        }
        self.ensure_capacity(device)?;
        let Some(buffer) = self.buffer.as_ref() else {
            return Err(CubemapUploadStagingError::CapacityOverflow);
        };
        if self.copies.iter().any(|copy| {
            uploads
                .get(copy.target_index)
                .and_then(Option::as_ref)
                .is_none()
        }) {
            return Err(CubemapUploadStagingError::MissingTarget);
        }

        let payload: Arc<[u8]> = Arc::from(self.bytes.as_slice());
        frame_uploads.push(
            WgpuBufferUpload::new(buffer.clone(), 0, payload, 0..self.bytes.len())
                .ok_or(CubemapUploadStagingError::InvalidUploadRange)?,
        );
        for copy in &self.copies {
            let Some((texture, _)) = uploads.get(copy.target_index).and_then(Option::as_ref) else {
                return Err(CubemapUploadStagingError::MissingTarget);
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
        Ok(())
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device) -> Result<(), CubemapUploadStagingError> {
        let required_bytes = u64::try_from(self.bytes.len())
            .map_err(|_| CubemapUploadStagingError::ByteLengthOverflow)?;
        if required_bytes <= self.capacity_bytes {
            return Ok(());
        }

        let capacity_bytes = required_bytes
            .max(INITIAL_STAGING_CAPACITY_BYTES)
            .checked_next_power_of_two()
            .ok_or(CubemapUploadStagingError::CapacityOverflow)?;
        self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-scene-environment-cubemap-upload-staging"),
            size: capacity_bytes,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
        self.capacity_bytes = capacity_bytes;
        Ok(())
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

    #[test]
    fn prepared_upload_batch_encodes_into_the_caller_frame_encoder() {
        let product = include_str!("upload_batch.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("product source precedes tests");

        assert!(product.contains("encoder: &mut wgpu::CommandEncoder"));
        assert!(product.contains("frame_uploads: &mut WgpuBufferUploadBatch"));
        assert!(product.contains("frame_uploads.push("));
        assert!(product.contains("return Err(CubemapUploadStagingError::MissingTarget)"));
        assert!(!product.contains("cubemap upload target was validated before batch publication"));
        assert!(!product.contains("queue.submit("));
        assert!(!product.contains("queue.write_buffer("));
        assert!(!product.contains("encoder.finish()"));
    }
}
