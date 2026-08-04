use std::fmt::Display;

use crate::asset::TextureUploadPlan;
use crate::graphics::types::GraphicsError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CompressedMipUpload {
    pub(super) level: u32,
    pub(super) layer: u32,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) data_offset: usize,
    pub(super) data_len: usize,
    pub(super) bytes_per_row: u32,
    pub(super) block_rows: u32,
}

impl CompressedMipUpload {
    const fn new(
        level: u32,
        layer: u32,
        width: u32,
        height: u32,
        data_offset: usize,
        data_len: usize,
        bytes_per_row: u32,
        block_rows: u32,
    ) -> Self {
        Self {
            level,
            layer,
            width,
            height,
            data_offset,
            data_len,
            bytes_per_row,
            block_rows,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn upload_compressed_texture_bytes<T: Display + ?Sized>(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    texture_uri: &T,
    width: u32,
    height: u32,
    mip_count: u32,
    layer_count: u32,
    data: &[u8],
    upload_bytes: &[u8],
    plan: &TextureUploadPlan,
) -> Result<(), GraphicsError> {
    if !plan.subresources.is_empty() {
        for upload in &plan.subresources {
            let source_end = upload
                .data_offset
                .checked_add(upload.data_length)
                .ok_or_else(|| {
                    GraphicsError::Asset(format!(
                        "texture {texture_uri} compressed subresource range overflows"
                    ))
                })?;
            let source = data.get(upload.data_offset..source_end).ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {texture_uri} compressed payload is missing mip {} layer {}",
                    upload.mip_level, upload.array_layer
                ))
            })?;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: upload.mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: upload.array_layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                source,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.bytes_per_row),
                    rows_per_image: Some(upload.block_rows),
                },
                wgpu::Extent3d {
                    width: mip_extent(width, upload.mip_level),
                    height: mip_extent(height, upload.mip_level),
                    depth_or_array_layers: 1,
                },
            );
        }
        return Ok(());
    }

    if plan.format.starts_with("dds/") {
        let uploads = dds_compressed_mip_uploads(width, height, mip_count, layer_count, plan)
            .ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {texture_uri} compressed DDS mip layout overflows"
                ))
            })?;
        for upload in uploads {
            let source_end = upload
                .data_offset
                .checked_add(upload.data_len)
                .ok_or_else(|| {
                    GraphicsError::Asset(format!(
                        "texture {texture_uri} compressed DDS subresource range overflows"
                    ))
                })?;
            let source = data.get(upload.data_offset..source_end).ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {texture_uri} compressed DDS payload is missing mip {} layer {}",
                    upload.level, upload.layer
                ))
            })?;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: upload.level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: upload.layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                source,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.bytes_per_row),
                    rows_per_image: Some(upload.block_rows),
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
        }
        return Ok(());
    }

    let block_columns = div_ceil(width.max(1), plan.block_width.max(1));
    let block_rows = div_ceil(height.max(1), plan.block_height.max(1));
    let bytes_per_row = block_columns
        .checked_mul(plan.bytes_per_block)
        .ok_or_else(|| {
            GraphicsError::Asset(format!("texture {texture_uri} row pitch overflows"))
        })?;
    let required_bytes = u64::from(bytes_per_row)
        .checked_mul(u64::from(block_rows))
        .and_then(|bytes| bytes.checked_mul(u64::from(layer_count)))
        .ok_or_else(|| {
            GraphicsError::Asset(format!("texture {texture_uri} upload size overflows"))
        })?;
    let required_bytes = usize::try_from(required_bytes).map_err(|_| {
        GraphicsError::Asset(format!("texture {texture_uri} upload size overflows"))
    })?;
    let source = upload_bytes.get(..required_bytes).ok_or_else(|| {
        GraphicsError::Asset(format!(
            "texture {texture_uri} compressed payload has {} bytes but needs at least {required_bytes}",
            upload_bytes.len()
        ))
    })?;
    queue.write_texture(
        texture.as_image_copy(),
        source,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(block_rows),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layer_count,
        },
    );
    Ok(())
}

#[derive(Clone, Copy)]
struct CompressedMipLevelLayout {
    level: u32,
    width: u32,
    height: u32,
    data_len: usize,
    bytes_per_row: u32,
    block_rows: u32,
}

pub(super) fn dds_compressed_mip_uploads(
    width: u32,
    height: u32,
    mip_count: u32,
    layer_count: u32,
    plan: &TextureUploadPlan,
) -> Option<Vec<CompressedMipUpload>> {
    if plan.block_depth != 1 {
        return None;
    }
    let mut levels = Vec::with_capacity(usize::try_from(mip_count).ok()?);
    let mut bytes_per_layer = 0_usize;
    for level in 0..mip_count {
        let width = mip_extent(width, level);
        let height = mip_extent(height, level);
        let block_columns = div_ceil(width, plan.block_width.max(1));
        let block_rows = div_ceil(height, plan.block_height.max(1));
        let bytes_per_row = block_columns.checked_mul(plan.bytes_per_block)?;
        let data_len =
            usize::try_from(u64::from(bytes_per_row).checked_mul(u64::from(block_rows))?).ok()?;
        bytes_per_layer = bytes_per_layer.checked_add(data_len)?;
        levels.push(CompressedMipLevelLayout {
            level,
            width,
            height,
            data_len,
            bytes_per_row,
            block_rows,
        });
    }

    let capacity = usize::try_from(mip_count.checked_mul(layer_count)?).ok()?;
    let mut uploads = Vec::with_capacity(capacity);
    for layer in 0..layer_count {
        let layer_offset = plan
            .data_offset
            .checked_add(usize::try_from(layer).ok()?.checked_mul(bytes_per_layer)?)?;
        let mut level_offset = layer_offset;
        for level in &levels {
            uploads.push(CompressedMipUpload::new(
                level.level,
                layer,
                level.width,
                level.height,
                level_offset,
                level.data_len,
                level.bytes_per_row,
                level.block_rows,
            ));
            level_offset = level_offset.checked_add(level.data_len)?;
        }
    }
    Some(uploads)
}

fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor.max(1)
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 { 1 } else { shifted }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{TextureUploadCompressionFamily, TextureUploadPlan};

    #[test]
    fn dds_compressed_mip_uploads_use_layer_major_source_offsets() {
        let plan = TextureUploadPlan {
            format: "dds/ati2".to_string(),
            compression: TextureUploadCompressionFamily::Bc,
            data_offset: 128,
            data_length: None,
            block_width: 4,
            block_height: 4,
            block_depth: 1,
            bytes_per_block: 16,
            subresources: Vec::new(),
        };

        let uploads = dds_compressed_mip_uploads(8, 4, 3, 2, &plan)
            .expect("valid BC5 DDS mip layout should fit in address space");

        assert_eq!(
            uploads,
            vec![
                CompressedMipUpload::new(0, 0, 8, 4, 128, 32, 32, 1),
                CompressedMipUpload::new(1, 0, 4, 2, 160, 16, 16, 1),
                CompressedMipUpload::new(2, 0, 2, 1, 176, 16, 16, 1),
                CompressedMipUpload::new(0, 1, 8, 4, 192, 32, 32, 1),
                CompressedMipUpload::new(1, 1, 4, 2, 224, 16, 16, 1),
                CompressedMipUpload::new(2, 1, 2, 1, 240, 16, 16, 1),
            ]
        );
    }
}
