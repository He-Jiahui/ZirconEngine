use std::sync::Arc;

use crate::graphics::types::GraphicsError;
use zr_rhi::TextureCopyRegion;
use zr_rhi_wgpu::WgpuTextureUpload;

use super::super::viewport_icon_sprite::ViewportIconSprite;

const RGBA8_BYTES_PER_TEXEL: u32 = 4;

pub(super) struct PreparedViewportIconSprite {
    pub(super) sprite: Arc<ViewportIconSprite>,
    pub(super) upload: WgpuTextureUpload,
}

pub(super) fn prepare_sprite(
    device: &wgpu::Device,
    texture_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> Result<PreparedViewportIconSprite, GraphicsError> {
    let bytes_per_row = width
        .checked_mul(RGBA8_BYTES_PER_TEXEL)
        .filter(|bytes_per_row| *bytes_per_row > 0 && height > 0)
        .ok_or_else(|| GraphicsError::Asset("viewport icon extent overflows".to_string()))?;
    let expected_byte_len: usize = bytes_per_row
        .checked_mul(height)
        .and_then(|byte_len| byte_len.try_into().ok())
        .ok_or_else(|| GraphicsError::Asset("viewport icon byte length overflows".to_string()))?;
    if rgba.len() != expected_byte_len {
        return Err(GraphicsError::Asset(format!(
            "viewport icon has {} rgba8 bytes but needs {expected_byte_len}",
            rgba.len()
        )));
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-viewport-icon-texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let upload = WgpuTextureUpload::from_owned_bytes(
        texture.clone(),
        TextureCopyRegion::new(width, height),
        bytes_per_row,
        height,
        rgba,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-viewport-icon-bind-group"),
        layout: texture_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    }));

    Ok(PreparedViewportIconSprite {
        sprite: Arc::new(ViewportIconSprite {
            _texture: texture,
            _view: view,
            bind_group,
        }),
        upload,
    })
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("create_sprite.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("viewport icon sprite source should retain a test-module boundary")
    }

    #[test]
    fn viewport_icon_sprite_prepares_owned_copy_upload_without_raw_queue_writes() {
        let source = production_source();

        assert!(!source.contains("wgpu::Queue"));
        assert!(!source.contains("write_texture"));
        assert!(source.contains("WgpuTextureUpload::from_owned_bytes("));
        assert!(source.contains("checked_mul(RGBA8_BYTES_PER_TEXEL)"));
        assert!(source.contains("checked_mul(height)"));
    }
}
