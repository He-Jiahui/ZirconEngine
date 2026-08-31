use zr_rhi::{RhiError, TextureCopyRegion};

use super::super::super::diagnostics::{
    DiagnosticTextureMipChainReadbackLayout, DiagnosticTextureReadbackLayout,
};

pub(super) fn record_native_diagnostic_texture_copy(
    encoder: &mut wgpu::CommandEncoder,
    staging: &wgpu::Buffer,
    staging_offset: u64,
    source: &wgpu::Texture,
    region: TextureCopyRegion,
    layout: DiagnosticTextureReadbackLayout,
) {
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: source,
            mip_level: region.mip_level,
            origin: wgpu::Origin3d {
                x: region.origin_x,
                y: region.origin_y,
                z: region.origin_z,
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: staging_offset,
                bytes_per_row: Some(layout.padded_bytes_per_row()),
                rows_per_image: Some(layout.height()),
            },
        },
        wgpu::Extent3d {
            width: region.width,
            height: region.height,
            depth_or_array_layers: 1,
        },
    );
}

pub(super) fn ensure_native_rgba16float_texture_mip_chain_readback(
    source: &wgpu::Texture,
    array_layer: u32,
    mip_count: u32,
) -> Result<DiagnosticTextureMipChainReadbackLayout, RhiError> {
    if mip_count == 0 || mip_count > source.mip_level_count() {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native RGBA16F diagnostic mip chain count {mip_count} exceeds source mip count {}",
                source.mip_level_count()
            ),
        });
    }
    let source_size = source.size();
    if source_size.width != source_size.height {
        return Err(RhiError::InvalidCopy {
            reason: format!(
                "native RGBA16F diagnostic mip chain requires a square texture, found {}x{}",
                source_size.width, source_size.height
            ),
        });
    }
    let mut subresources = Vec::with_capacity(mip_count as usize);
    for mip_level in 0..mip_count {
        let mip_size = source_size.width.checked_shr(mip_level).unwrap_or(0).max(1);
        subresources.push(super::ensure_native_rgba16float_texture_readback(
            source,
            mip_level,
            array_layer,
            mip_size,
            mip_size,
        )?);
    }
    DiagnosticTextureMipChainReadbackLayout::new(subresources).ok_or_else(|| {
        RhiError::InvalidCopy {
            reason: "native RGBA16F diagnostic mip-chain layout overflowed".to_string(),
        }
    })
}
