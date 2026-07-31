use crate::asset::{RGBA8_UNORM_FORMAT, TextureAsset, TexturePayload};
use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};
use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;

const RGBA8_BYTES_PER_TEXEL: u32 = 4;

pub(in crate::graphics::scene::resources) struct PostProcessLutTextureResource {
    pub(in crate::graphics::scene::resources) descriptor: RenderImageDescriptor,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl PostProcessLutTextureResource {
    pub(in crate::graphics::scene::resources) const RETAINED_LUT_TEXTURE_OWNER_COUNT: usize = 2;

    pub(in crate::graphics::scene::resources) fn retained_lut_texture_owner_count(&self) -> usize {
        let _retained_lut_texture_owners = (&self.texture, &self.view);
        Self::RETAINED_LUT_TEXTURE_OWNER_COUNT
    }

    pub(in crate::graphics::scene::resources) fn view(&self) -> &wgpu::TextureView {
        debug_assert_eq!(
            self.retained_lut_texture_owner_count(),
            Self::RETAINED_LUT_TEXTURE_OWNER_COUNT,
            "PostProcessLutTextureResource must retain texture and view while exposing 3D LUT bindings",
        );
        &self.view
    }

    pub(in crate::graphics::scene::resources) fn from_rgba8_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: ResourceId,
        payload: TextureAsset,
    ) -> Result<Self, GraphicsError> {
        if payload.payload != TexturePayload::Rgba8 {
            return Err(GraphicsError::Asset(format!(
                "post-process LUT texture {id} must be a decoded rgba8 asset"
            )));
        }

        let descriptor = payload.render_image_descriptor();
        let depth_or_array_layers = descriptor.depth_or_array_layers.max(1);
        let upload_size = rgba8_upload_size(payload.width, payload.height, depth_or_array_layers)
            .ok_or_else(|| {
            GraphicsError::Asset(format!(
                "post-process LUT texture {id} extent {}x{}x{} overflows",
                payload.width, payload.height, depth_or_array_layers
            ))
        })?;
        if payload.rgba.len() < upload_size {
            return Err(GraphicsError::Asset(format!(
                "post-process LUT texture {id} has {} bytes but needs {upload_size}",
                payload.rgba.len()
            )));
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-post-process-lut-texture"),
            size: wgpu::Extent3d {
                width: payload.width.max(1),
                height: payload.height.max(1),
                depth_or_array_layers,
            },
            mip_level_count: descriptor.mip_count.max(1),
            sample_count: 1,
            dimension: wgpu_dimension(descriptor.dimension),
            format: rgba8_wgpu_format(&descriptor),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &payload.rgba[..upload_size],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(payload.width.max(1) * RGBA8_BYTES_PER_TEXEL),
                rows_per_image: Some(payload.height.max(1)),
            },
            wgpu::Extent3d {
                width: payload.width.max(1),
                height: payload.height.max(1),
                depth_or_array_layers,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            descriptor,
            texture,
            view,
        })
    }
}

fn rgba8_upload_size(width: u32, height: u32, depth_or_array_layers: u32) -> Option<usize> {
    width
        .max(1)
        .checked_mul(height.max(1))?
        .checked_mul(depth_or_array_layers.max(1))?
        .checked_mul(RGBA8_BYTES_PER_TEXEL)?
        .try_into()
        .ok()
}

fn wgpu_dimension(dimension: RenderImageDimension) -> wgpu::TextureDimension {
    match dimension {
        RenderImageDimension::D1 => wgpu::TextureDimension::D1,
        RenderImageDimension::D2 => wgpu::TextureDimension::D2,
        RenderImageDimension::D3 => wgpu::TextureDimension::D3,
        RenderImageDimension::Cube => wgpu::TextureDimension::D2,
    }
}

fn rgba8_wgpu_format(descriptor: &RenderImageDescriptor) -> wgpu::TextureFormat {
    if descriptor
        .format
        .trim()
        .eq_ignore_ascii_case(RGBA8_UNORM_FORMAT)
    {
        wgpu::TextureFormat::Rgba8Unorm
    } else {
        wgpu::TextureFormat::Rgba8UnormSrgb
    }
}
