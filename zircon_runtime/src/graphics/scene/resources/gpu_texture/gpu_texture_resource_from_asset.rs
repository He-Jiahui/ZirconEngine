use crate::asset::assets::{
    TextureAsset, TextureUploadCompressionFamily, TextureUploadPlan, TextureUploadReadiness,
    TextureUploadSupport,
};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDimension, RenderSamplerAddressMode, RenderSamplerFilter,
};
use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;

use super::GpuTextureResource;

impl GpuTextureResource {
    pub(crate) fn from_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
    ) -> Result<Self, GraphicsError> {
        let support = texture_upload_support_from_device(device);
        match payload.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan }
                if plan.compression == TextureUploadCompressionFamily::Uncompressed =>
            {
                Ok(Self::from_rgba8_asset(
                    device,
                    queue,
                    texture_layout,
                    id,
                    payload,
                ))
            }
            TextureUploadReadiness::Ready { plan } => {
                Self::from_compressed_asset(device, queue, texture_layout, id, payload, plan)
            }
            TextureUploadReadiness::Unsupported { reason } => Err(GraphicsError::Asset(format!(
                "texture {} is not upload-ready: {reason}",
                payload.uri
            ))),
        }
    }

    fn from_rgba8_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-texture"),
            size: wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::graphics::scene::OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &payload.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * payload.width),
                rows_per_image: Some(payload.height),
            },
            wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-texture-bind-group"),
            layout: texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            id: Some(id),
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    fn from_compressed_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
        plan: TextureUploadPlan,
    ) -> Result<Self, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
        let format = compressed_wgpu_format(&plan, descriptor.color_space).ok_or_else(|| {
            GraphicsError::Asset(format!(
                "texture {} has unsupported upload format {}",
                payload.uri, plan.format
            ))
        })?;
        let data = match &payload.payload {
            crate::asset::TexturePayload::Container { bytes, .. } => bytes,
            crate::asset::TexturePayload::Rgba8 => {
                return Err(GraphicsError::Asset(format!(
                    "texture {} was planned as compressed but has rgba payload",
                    payload.uri
                )));
            }
        };
        let upload_bytes = data.get(plan.data_offset..).ok_or_else(|| {
            GraphicsError::Asset(format!(
                "texture {} missing compressed payload after {} byte header",
                payload.uri, plan.data_offset
            ))
        })?;
        let depth_or_array_layers = descriptor.depth_or_array_layers.max(1);
        let block_columns = div_ceil(payload.width.max(1), plan.block_width.max(1));
        let block_rows = div_ceil(payload.height.max(1), plan.block_height.max(1));
        let bytes_per_row = block_columns
            .checked_mul(plan.bytes_per_block)
            .ok_or_else(|| {
                GraphicsError::Asset(format!("texture {} row pitch overflows", payload.uri))
            })?;
        let required_bytes = u64::from(bytes_per_row)
            .checked_mul(u64::from(block_rows))
            .and_then(|bytes| bytes.checked_mul(u64::from(depth_or_array_layers)))
            .ok_or_else(|| {
                GraphicsError::Asset(format!("texture {} upload size overflows", payload.uri))
            })?;
        if upload_bytes.len() < required_bytes as usize {
            return Err(GraphicsError::Asset(format!(
                "texture {} compressed payload has {} bytes but needs at least {}",
                payload.uri,
                upload_bytes.len(),
                required_bytes
            )));
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-compressed-texture"),
            size: wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers,
            },
            mip_level_count: descriptor.mip_count.max(1),
            sample_count: 1,
            dimension: wgpu_dimension(descriptor.dimension),
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &upload_bytes[..required_bytes as usize],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(block_rows),
            },
            wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&sampler_descriptor(&descriptor.sampler));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-compressed-texture-bind-group"),
            layout: texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Ok(Self {
            id: Some(id),
            texture,
            view,
            sampler,
            bind_group,
        })
    }
}

pub(crate) fn texture_upload_support_from_device(device: &wgpu::Device) -> TextureUploadSupport {
    let features = device.features();
    TextureUploadSupport {
        bc: features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
        bc_sliced_3d: features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC_SLICED_3D),
        etc2: features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2),
        astc_ldr: features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC),
        astc_sliced_3d: features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC_SLICED_3D),
    }
}

fn compressed_wgpu_format(
    plan: &TextureUploadPlan,
    color_space: RenderImageColorSpace,
) -> Option<wgpu::TextureFormat> {
    let srgb = color_space == RenderImageColorSpace::Srgb;
    match plan.format.as_str() {
        "dds/dxt1" => Some(if srgb {
            wgpu::TextureFormat::Bc1RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Bc1RgbaUnorm
        }),
        "dds/dxt3" => Some(if srgb {
            wgpu::TextureFormat::Bc2RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Bc2RgbaUnorm
        }),
        "dds/dxt5" => Some(if srgb {
            wgpu::TextureFormat::Bc3RgbaUnormSrgb
        } else {
            wgpu::TextureFormat::Bc3RgbaUnorm
        }),
        "dds/dxgi-71" => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        "dds/dxgi-72" => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        "dds/dxgi-74" => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        "dds/dxgi-75" => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        "dds/dxgi-77" => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        "dds/dxgi-78" => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        "dds/dxgi-98" => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        "dds/dxgi-99" => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        format if format.starts_with("astc/") && plan.block_depth == 1 => {
            Some(wgpu::TextureFormat::Astc {
                block: astc_block(plan.block_width, plan.block_height)?,
                channel: if srgb {
                    wgpu::AstcChannel::UnormSrgb
                } else {
                    wgpu::AstcChannel::Unorm
                },
            })
        }
        _ => None,
    }
}

fn astc_block(width: u32, height: u32) -> Option<wgpu::AstcBlock> {
    match (width, height) {
        (4, 4) => Some(wgpu::AstcBlock::B4x4),
        (5, 4) => Some(wgpu::AstcBlock::B5x4),
        (5, 5) => Some(wgpu::AstcBlock::B5x5),
        (6, 5) => Some(wgpu::AstcBlock::B6x5),
        (6, 6) => Some(wgpu::AstcBlock::B6x6),
        (8, 5) => Some(wgpu::AstcBlock::B8x5),
        (8, 6) => Some(wgpu::AstcBlock::B8x6),
        (8, 8) => Some(wgpu::AstcBlock::B8x8),
        (10, 5) => Some(wgpu::AstcBlock::B10x5),
        (10, 6) => Some(wgpu::AstcBlock::B10x6),
        (10, 8) => Some(wgpu::AstcBlock::B10x8),
        (10, 10) => Some(wgpu::AstcBlock::B10x10),
        (12, 10) => Some(wgpu::AstcBlock::B12x10),
        (12, 12) => Some(wgpu::AstcBlock::B12x12),
        _ => None,
    }
}

fn wgpu_dimension(dimension: RenderImageDimension) -> wgpu::TextureDimension {
    match dimension {
        RenderImageDimension::D1 => wgpu::TextureDimension::D1,
        RenderImageDimension::D2 => wgpu::TextureDimension::D2,
        RenderImageDimension::D3 => wgpu::TextureDimension::D3,
    }
}

fn sampler_descriptor(
    descriptor: &crate::core::framework::render::RenderSamplerDescriptor,
) -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        mag_filter: filter_mode(descriptor.mag_filter),
        min_filter: filter_mode(descriptor.min_filter),
        mipmap_filter: mipmap_filter_mode(descriptor.mipmap_filter),
        address_mode_u: address_mode(descriptor.address_mode_u),
        address_mode_v: address_mode(descriptor.address_mode_v),
        address_mode_w: address_mode(descriptor.address_mode_w),
        ..Default::default()
    }
}

fn filter_mode(filter: RenderSamplerFilter) -> wgpu::FilterMode {
    match filter {
        RenderSamplerFilter::Nearest => wgpu::FilterMode::Nearest,
        RenderSamplerFilter::Linear => wgpu::FilterMode::Linear,
    }
}

fn mipmap_filter_mode(filter: RenderSamplerFilter) -> wgpu::MipmapFilterMode {
    match filter {
        RenderSamplerFilter::Nearest => wgpu::MipmapFilterMode::Nearest,
        RenderSamplerFilter::Linear => wgpu::MipmapFilterMode::Linear,
    }
}

fn address_mode(mode: RenderSamplerAddressMode) -> wgpu::AddressMode {
    match mode {
        RenderSamplerAddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        RenderSamplerAddressMode::Repeat => wgpu::AddressMode::Repeat,
        RenderSamplerAddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}

fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor.max(1)
}
