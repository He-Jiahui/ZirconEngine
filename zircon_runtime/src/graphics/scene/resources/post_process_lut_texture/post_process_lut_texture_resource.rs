use std::sync::Arc;

use crate::asset::{RGBA8_UNORM_FORMAT, TextureAsset, TexturePayload};
use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};
use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;
use zr_rhi::TextureCopyRegion;
use zr_rhi_wgpu::{WgpuTextureUpload, WgpuTextureUploadBatch};

const RGBA8_BYTES_PER_TEXEL: u32 = 4;

pub(in crate::graphics::scene::resources) struct PostProcessLutTextureResource {
    pub(in crate::graphics::scene::resources) descriptor: RenderImageDescriptor,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

pub(in crate::graphics::scene::resources) struct PostProcessLutTextureUploadWork {
    pub(in crate::graphics::scene::resources) resource: PostProcessLutTextureResource,
    pub(in crate::graphics::scene::resources) upload_batch: WgpuTextureUploadBatch,
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

    pub(in crate::graphics::scene::resources) fn prepare_from_rgba8_asset(
        device: &wgpu::Device,
        id: ResourceId,
        payload: &TextureAsset,
    ) -> Result<PostProcessLutTextureUploadWork, GraphicsError> {
        if payload.payload != TexturePayload::Rgba8 {
            return Err(GraphicsError::Asset(format!(
                "post-process LUT texture {id} must be a decoded rgba8 asset"
            )));
        }

        let descriptor = payload.render_image_descriptor();
        let upload_layout = rgba8_upload_layout(
            payload.width,
            payload.height,
            descriptor.depth_or_array_layers,
        )
        .ok_or_else(|| {
            GraphicsError::Asset(format!(
                "post-process LUT texture {id} extent {}x{}x{} overflows",
                payload.width, payload.height, descriptor.depth_or_array_layers
            ))
        })?;
        if payload.rgba.len() < upload_layout.byte_len {
            return Err(GraphicsError::Asset(format!(
                "post-process LUT texture {id} has {} bytes but needs {}",
                payload.rgba.len(),
                upload_layout.byte_len
            )));
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-post-process-lut-texture"),
            size: wgpu::Extent3d {
                width: upload_layout.width,
                height: upload_layout.height,
                depth_or_array_layers: upload_layout.depth_or_array_layers,
            },
            mip_level_count: descriptor.mip_count.max(1),
            sample_count: 1,
            dimension: wgpu_dimension(descriptor.dimension),
            format: rgba8_wgpu_format(&descriptor),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let rgba: Arc<[u8]> = Arc::from(&payload.rgba[..upload_layout.byte_len]);
        let upload = WgpuTextureUpload::new(
            texture.clone(),
            TextureCopyRegion::new(upload_layout.width, upload_layout.height)
                .with_depth_or_array_layers(upload_layout.depth_or_array_layers),
            upload_layout.bytes_per_row,
            upload_layout.rows_per_image,
            rgba,
            0..upload_layout.byte_len,
        )
        .expect("validated LUT upload range must reference its immutable payload");
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(PostProcessLutTextureUploadWork {
            resource: Self {
                descriptor,
                texture,
                view,
            },
            upload_batch: WgpuTextureUploadBatch::from(upload),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rgba8UploadLayout {
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    bytes_per_row: u32,
    rows_per_image: u32,
    byte_len: usize,
}

fn rgba8_upload_layout(
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
) -> Option<Rgba8UploadLayout> {
    let width = width.max(1);
    let height = height.max(1);
    let depth_or_array_layers = depth_or_array_layers.max(1);
    let bytes_per_row = width.checked_mul(RGBA8_BYTES_PER_TEXEL)?;
    let byte_len = bytes_per_row
        .checked_mul(height)?
        .checked_mul(depth_or_array_layers)?
        .try_into()
        .ok()?;
    Some(Rgba8UploadLayout {
        width,
        height,
        depth_or_array_layers,
        bytes_per_row,
        rows_per_image: height,
        byte_len,
    })
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

#[cfg(test)]
mod tests {
    use super::rgba8_upload_layout;

    #[test]
    fn rgba8_upload_layout_describes_one_contiguous_volume_copy() {
        let layout = rgba8_upload_layout(32, 32, 32).expect("valid LUT upload layout");

        assert_eq!(layout.bytes_per_row, 128);
        assert_eq!(layout.rows_per_image, 32);
        assert_eq!(layout.depth_or_array_layers, 32);
        assert_eq!(layout.byte_len, 131_072);
    }

    #[test]
    fn rgba8_upload_layout_rejects_extent_overflow() {
        assert!(rgba8_upload_layout(u32::MAX, 2, 2).is_none());
    }

    #[test]
    fn lut_resource_preparation_has_no_private_queue_write() {
        let production = include_str!("post_process_lut_texture_resource.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("LUT resource test boundary");

        assert!(production.contains("let rgba: Arc<[u8]> = Arc::from("));
        assert!(production.contains("WgpuTextureUpload::new("));
        assert!(production.contains(".with_depth_or_array_layers("));
        assert!(production.contains("WgpuTextureUploadBatch::from(upload)"));
        assert!(!production.contains("queue.write_texture"));
        assert!(!production.contains("wgpu::Queue"));
    }
}
