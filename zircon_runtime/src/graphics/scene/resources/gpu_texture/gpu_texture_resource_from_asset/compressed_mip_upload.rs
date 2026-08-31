use std::fmt::Display;
use std::sync::Arc;

use crate::asset::TextureUploadPlan;
use crate::graphics::types::GraphicsError;
use zr_rhi::TextureCopyRegion;
use zr_rhi_wgpu::{WgpuTextureUpload, WgpuTextureUploadBatch};

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
pub(super) fn enqueue_compressed_texture_uploads<T: Display + ?Sized>(
    batch: &mut WgpuTextureUploadBatch,
    texture: &wgpu::Texture,
    texture_uri: &T,
    width: u32,
    height: u32,
    mip_count: u32,
    layer_count: u32,
    data: Arc<[u8]>,
    plan: &TextureUploadPlan,
) -> Result<(), GraphicsError> {
    let payload_start = plan.data_offset;
    let payload_end = match plan.data_length {
        Some(length) => payload_start.checked_add(length).ok_or_else(|| {
            GraphicsError::Asset(format!(
                "texture {texture_uri} compressed payload range overflows"
            ))
        })?,
        None => data.len(),
    };
    if payload_start > payload_end || payload_end > data.len() {
        return Err(GraphicsError::Asset(format!(
            "texture {texture_uri} compressed payload is shorter than its upload plan"
        )));
    }
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
            enqueue_compressed_subresource(
                batch,
                texture,
                upload.mip_level,
                upload.array_layer,
                mip_extent(width, upload.mip_level),
                mip_extent(height, upload.mip_level),
                upload.bytes_per_row,
                upload.block_rows,
                Arc::clone(&data),
                upload.data_offset..source_end,
                payload_start..payload_end,
                texture_uri,
            )?;
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
            enqueue_compressed_subresource(
                batch,
                texture,
                upload.level,
                upload.layer,
                upload.width,
                upload.height,
                upload.bytes_per_row,
                upload.block_rows,
                Arc::clone(&data),
                upload.data_offset..source_end,
                payload_start..payload_end,
                texture_uri,
            )?;
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
    let layer_bytes = usize::try_from(
        u64::from(bytes_per_row)
            .checked_mul(u64::from(block_rows))
            .ok_or_else(|| {
                GraphicsError::Asset(format!("texture {texture_uri} upload size overflows"))
            })?,
    )
    .map_err(|_| GraphicsError::Asset(format!("texture {texture_uri} upload size overflows")))?;
    for layer in 0..layer_count {
        let start = payload_start
            .checked_add(layer_bytes.checked_mul(layer as usize).ok_or_else(|| {
                GraphicsError::Asset(format!("texture {texture_uri} upload size overflows"))
            })?)
            .ok_or_else(|| {
                GraphicsError::Asset(format!("texture {texture_uri} upload size overflows"))
            })?;
        let end = start.checked_add(layer_bytes).ok_or_else(|| {
            GraphicsError::Asset(format!("texture {texture_uri} upload size overflows"))
        })?;
        enqueue_compressed_subresource(
            batch,
            texture,
            0,
            layer,
            width,
            height,
            bytes_per_row,
            block_rows,
            Arc::clone(&data),
            start..end,
            payload_start..payload_end,
            texture_uri,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn enqueue_compressed_subresource<T: Display + ?Sized>(
    batch: &mut WgpuTextureUploadBatch,
    texture: &wgpu::Texture,
    mip_level: u32,
    layer: u32,
    width: u32,
    height: u32,
    bytes_per_row: u32,
    block_rows: u32,
    data: Arc<[u8]>,
    source_range: core::ops::Range<usize>,
    payload_range: core::ops::Range<usize>,
    texture_uri: &T,
) -> Result<(), GraphicsError> {
    if source_range.start < payload_range.start || source_range.end > payload_range.end {
        return Err(GraphicsError::Asset(format!(
            "texture {texture_uri} compressed upload range is outside its declared payload"
        )));
    }
    let upload = WgpuTextureUpload::new(
        texture.clone(),
        TextureCopyRegion::new(width, height)
            .with_mip_level(mip_level)
            .with_origin(0, 0, layer),
        bytes_per_row,
        block_rows,
        data,
        source_range,
    )
    .ok_or_else(|| {
        GraphicsError::Asset(format!(
            "texture {texture_uri} compressed source range is invalid"
        ))
    })?;
    batch.push(upload);
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
