use super::ticket::ReadbackError;

const R32_UINT_BYTES_PER_TEXEL: u32 = size_of::<u32>() as u32;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextureReadbackLayout {
    pub(crate) unpadded_bytes_per_row: u32,
    pub(crate) padded_bytes_per_row: u32,
    pub(crate) staging_byte_len: u64,
    pub(crate) height: u32,
}

impl TextureReadbackLayout {
    pub(crate) fn unpack_rgba(self, mapped: &[u8]) -> Result<Vec<u8>, ReadbackError> {
        self.validate_mapped_len(mapped)?;
        let row_bytes = usize::try_from(self.unpadded_bytes_per_row)
            .map_err(|_| ReadbackError::CapacityOverflow)?;
        let padded_row_bytes = usize::try_from(self.padded_bytes_per_row)
            .map_err(|_| ReadbackError::CapacityOverflow)?;
        let total_rgba_bytes = row_bytes
            .checked_mul(self.height as usize)
            .ok_or(ReadbackError::CapacityOverflow)?;
        let mut rgba = Vec::with_capacity(total_rgba_bytes);
        for row in 0..self.height as usize {
            let start = row
                .checked_mul(padded_row_bytes)
                .ok_or(ReadbackError::CapacityOverflow)?;
            let end = start
                .checked_add(row_bytes)
                .ok_or(ReadbackError::CapacityOverflow)?;
            let Some(source_row) = mapped.get(start..end) else {
                return Err(ReadbackError::BufferMap(
                    "texture readback row exceeded its mapped staging range".to_string(),
                ));
            };
            rgba.extend_from_slice(source_row);
        }
        Ok(rgba)
    }

    pub(crate) fn unpack_r32_uint(self, mapped: &[u8]) -> Result<u32, ReadbackError> {
        self.validate_mapped_len(mapped)?;
        let bytes = mapped
            .get(..size_of::<u32>())
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| {
                ReadbackError::BufferMap(
                    "R32Uint texture readback did not contain one complete texel".to_string(),
                )
            })?;
        Ok(u32::from_ne_bytes(bytes))
    }

    fn validate_mapped_len(self, mapped: &[u8]) -> Result<usize, ReadbackError> {
        let staging_byte_len =
            usize::try_from(self.staging_byte_len).map_err(|_| ReadbackError::CapacityOverflow)?;
        if mapped.len() < staging_byte_len {
            return Err(ReadbackError::BufferMap(
                "texture readback mapped range was smaller than its staging layout".to_string(),
            ));
        }
        Ok(staging_byte_len)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextureReadbackCopy {
    pub(crate) origin: wgpu::Origin3d,
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) layout: TextureReadbackLayout,
}

pub(crate) fn texture_rgba_readback_copy(
    width: u32,
    height: u32,
) -> Result<TextureReadbackCopy, ReadbackError> {
    Ok(TextureReadbackCopy {
        origin: wgpu::Origin3d::ZERO,
        extent: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        layout: texture_readback_layout(width, height, 4)?,
    })
}

pub(crate) fn texture_r32_uint_texel_readback_copy(
    texture: &wgpu::Texture,
    pixel: [u32; 2],
) -> Result<TextureReadbackCopy, ReadbackError> {
    if texture.format() != wgpu::TextureFormat::R32Uint {
        return Err(ReadbackError::TextureFormatMismatch {
            expected: wgpu::TextureFormat::R32Uint,
            actual: texture.format(),
        });
    }
    if texture.dimension() != wgpu::TextureDimension::D2 {
        return Err(ReadbackError::UnsupportedTextureDimension {
            actual: texture.dimension(),
        });
    }
    if texture.sample_count() != 1 {
        return Err(ReadbackError::UnsupportedTextureSampleCount {
            sample_count: texture.sample_count(),
        });
    }
    if !texture.usage().contains(wgpu::TextureUsages::COPY_SRC) {
        return Err(ReadbackError::TextureMissingCopySourceUsage {
            actual: texture.usage(),
        });
    }
    if pixel[0] >= texture.width() || pixel[1] >= texture.height() {
        return Err(ReadbackError::TextureCoordinateOutOfBounds {
            x: pixel[0],
            y: pixel[1],
            width: texture.width(),
            height: texture.height(),
        });
    }

    Ok(TextureReadbackCopy {
        origin: wgpu::Origin3d {
            x: pixel[0],
            y: pixel[1],
            z: 0,
        },
        extent: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        layout: texture_readback_layout(1, 1, R32_UINT_BYTES_PER_TEXEL)?,
    })
}

fn texture_readback_layout(
    width: u32,
    height: u32,
    bytes_per_texel: u32,
) -> Result<TextureReadbackLayout, ReadbackError> {
    if width == 0 || height == 0 {
        return Err(ReadbackError::InvalidTextureExtent { width, height });
    }
    let unpadded_bytes_per_row = width
        .checked_mul(bytes_per_texel)
        .ok_or(ReadbackError::InvalidTextureExtent { width, height })?;
    let padded_bytes_per_row = unpadded_bytes_per_row
        .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        .checked_mul(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        .ok_or(ReadbackError::InvalidTextureExtent { width, height })?;
    let staging_byte_len = u64::from(padded_bytes_per_row)
        .checked_mul(u64::from(height))
        .ok_or(ReadbackError::InvalidTextureExtent { width, height })?;
    Ok(TextureReadbackLayout {
        unpadded_bytes_per_row,
        padded_bytes_per_row,
        staging_byte_len,
        height,
    })
}
