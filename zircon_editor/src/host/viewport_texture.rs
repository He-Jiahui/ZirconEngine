//! Host-side validation for GPU viewport textures before a UI toolkit imports them.

use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ViewportTextureBridgeError {
    #[error("unsupported viewport texture format {0:?}")]
    UnsupportedFormat(wgpu::TextureFormat),
    #[error("viewport texture is missing required usage {0:?}")]
    MissingUsage(wgpu::TextureUsages),
    #[error("viewport texture size must be non-zero")]
    ZeroSize,
}

#[derive(Clone, Debug, Default)]
pub struct ViewportTextureBridge;

impl ViewportTextureBridge {
    pub fn validate_metadata(
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Result<(), ViewportTextureBridgeError> {
        if !matches!(
            format,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
        ) {
            return Err(ViewportTextureBridgeError::UnsupportedFormat(format));
        }
        if !usage.contains(wgpu::TextureUsages::TEXTURE_BINDING) {
            return Err(ViewportTextureBridgeError::MissingUsage(
                wgpu::TextureUsages::TEXTURE_BINDING,
            ));
        }
        if !usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            return Err(ViewportTextureBridgeError::MissingUsage(
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ));
        }
        if width == 0 || height == 0 {
            return Err(ViewportTextureBridgeError::ZeroSize);
        }
        Ok(())
    }
}
