use crate::asset::assets::{
    TextureAsset, TextureUploadCompressionFamily, TextureUploadPlan, TextureUploadReadiness,
    TextureUploadSupport, LIGHTMAP_RGBA16F_GPU_FORMAT, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension, RenderImageUsage,
    RenderSamplerAddressMode, RenderSamplerFilter,
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
                if plan.compression == TextureUploadCompressionFamily::Uncompressed
                    && plan.format == LIGHTMAP_RGBA16F_GPU_FORMAT =>
            {
                Self::from_lightmap_rgba16f_asset(device, queue, texture_layout, id, payload)
            }
            TextureUploadReadiness::Ready { plan }
                if plan.compression == TextureUploadCompressionFamily::Uncompressed =>
            {
                Ok(Self::from_rgba8_asset(
                    device,
                    queue,
                    texture_layout,
                    id,
                    payload,
                    plan,
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

    fn from_lightmap_rgba16f_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
    ) -> Result<Self, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
        let layer_count = descriptor.array_layer_count.max(1);
        let crate::asset::TexturePayload::Container { bytes, .. } = &payload.payload else {
            return Err(GraphicsError::Asset(format!(
                "lightmap texture {} requires a raw rgba16f container payload",
                payload.uri
            )));
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-lightmap-rgba16f-array"),
            size: wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers: layer_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu_texture_usages(&descriptor, wgpu::TextureFormat::Rgba16Float, true),
            view_formats: &[],
        });
        let page_size_bytes = u64::from(payload.width)
            .checked_mul(u64::from(payload.height))
            .and_then(|texels| texels.checked_mul(8))
            .ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "lightmap texture {} upload size overflows",
                    payload.uri
                ))
            })?;
        for layer in 0..layer_count {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                bytes,
                wgpu::TexelCopyBufferLayout {
                    offset: page_size_bytes * u64::from(layer),
                    bytes_per_row: Some(8 * payload.width),
                    rows_per_image: Some(payload.height),
                },
                wgpu::Extent3d {
                    width: payload.width,
                    height: payload.height,
                    depth_or_array_layers: 1,
                },
            );
        }
        let view = texture.create_view(&lightmap_texture_view_descriptor(layer_count));
        let legacy_bind_group_view =
            texture.create_view(&lightmap_legacy_bind_group_view_descriptor());
        let sampler = device.create_sampler(&sampler_descriptor(&descriptor.sampler));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-lightmap-rgba16f-bind-group"),
            layout: texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&legacy_bind_group_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Ok(Self {
            id: Some(id),
            descriptor,
            texture,
            view,
            sampler,
            bind_group,
        })
    }

    fn from_rgba8_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
        plan: TextureUploadPlan,
    ) -> Self {
        let descriptor = payload.render_image_descriptor();
        let mip_level_count = descriptor.mip_count.max(1);
        let layer_count = descriptor.depth_or_array_layers.max(1);
        let format = rgba8_wgpu_format(&plan.format);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-texture"),
            size: wgpu::Extent3d {
                width: payload.width,
                height: payload.height,
                depth_or_array_layers: layer_count,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu_dimension(descriptor.dimension),
            format,
            usage: wgpu_texture_usages(&descriptor, format, true),
            view_formats: &[],
        });
        for upload in rgba8_mip_uploads(payload.width, payload.height, mip_level_count, layer_count)
        {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: upload.level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: upload.layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &payload.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: upload.offset,
                    bytes_per_row: Some(4 * upload.width),
                    rows_per_image: Some(upload.height),
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
        }
        let view = texture.create_view(&texture_view_descriptor(&descriptor));
        let sampler = device.create_sampler(&sampler_descriptor(&descriptor.sampler));
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
            descriptor,
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
        let upload_bytes = if let Some(data_length) = plan.data_length {
            upload_bytes.get(..data_length).ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {} declares {} compressed payload bytes but only {} are available",
                    payload.uri,
                    data_length,
                    upload_bytes.len()
                ))
            })?
        } else {
            upload_bytes
        };
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
            usage: wgpu_texture_usages(&descriptor, format, true),
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
        let view = texture.create_view(&texture_view_descriptor(&descriptor));
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
            descriptor,
            texture,
            view,
            sampler,
            bind_group,
        })
    }
}

fn wgpu_texture_usages(
    descriptor: &RenderImageDescriptor,
    format: wgpu::TextureFormat,
    requires_upload_dst: bool,
) -> wgpu::TextureUsages {
    let mut usages = wgpu::TextureUsages::empty();
    for usage in &descriptor.usage {
        match usage {
            RenderImageUsage::Sampled => usages |= wgpu::TextureUsages::TEXTURE_BINDING,
            RenderImageUsage::Storage if supports_storage_binding_usage(format) => {
                usages |= wgpu::TextureUsages::STORAGE_BINDING;
            }
            RenderImageUsage::Storage => {}
            RenderImageUsage::RenderTarget if supports_render_attachment_usage(format) => {
                usages |= wgpu::TextureUsages::RENDER_ATTACHMENT;
            }
            RenderImageUsage::RenderTarget => {}
            RenderImageUsage::CopySrc => usages |= wgpu::TextureUsages::COPY_SRC,
            RenderImageUsage::CopyDst => usages |= wgpu::TextureUsages::COPY_DST,
        }
    }
    if requires_upload_dst {
        usages |= wgpu::TextureUsages::COPY_DST;
    }
    usages
}

fn supports_render_attachment_usage(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
    )
}

fn supports_storage_binding_usage(format: wgpu::TextureFormat) -> bool {
    matches!(
        format,
        wgpu::TextureFormat::R8Unorm
            | wgpu::TextureFormat::R16Float
            | wgpu::TextureFormat::R32Float
            | wgpu::TextureFormat::Rg16Float
            | wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba16Float
            | wgpu::TextureFormat::Rgba32Float
    )
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

fn rgba8_wgpu_format(format: &str) -> wgpu::TextureFormat {
    if format.trim().eq_ignore_ascii_case(RGBA8_UNORM_FORMAT) {
        wgpu::TextureFormat::Rgba8Unorm
    } else if format.trim().eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT) {
        wgpu::TextureFormat::Rgba8UnormSrgb
    } else {
        wgpu::TextureFormat::Rgba8UnormSrgb
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
        "dds/ati1" | "dds/bc4u" => Some(wgpu::TextureFormat::Bc4RUnorm),
        "dds/bc4s" => Some(wgpu::TextureFormat::Bc4RSnorm),
        "dds/ati2" | "dds/bc5u" => Some(wgpu::TextureFormat::Bc5RgUnorm),
        "dds/bc5s" => Some(wgpu::TextureFormat::Bc5RgSnorm),
        "dds/dxgi-71" => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        "dds/dxgi-72" => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        "dds/dxgi-74" => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        "dds/dxgi-75" => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        "dds/dxgi-77" => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        "dds/dxgi-78" => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        "dds/dxgi-80" => Some(wgpu::TextureFormat::Bc4RUnorm),
        "dds/dxgi-81" => Some(wgpu::TextureFormat::Bc4RSnorm),
        "dds/dxgi-83" => Some(wgpu::TextureFormat::Bc5RgUnorm),
        "dds/dxgi-84" => Some(wgpu::TextureFormat::Bc5RgSnorm),
        "dds/dxgi-95" => Some(wgpu::TextureFormat::Bc6hRgbUfloat),
        "dds/dxgi-96" => Some(wgpu::TextureFormat::Bc6hRgbFloat),
        "dds/dxgi-98" => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        "dds/dxgi-99" => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        format if format.starts_with("ktx/gl-internal-0x") => ktx_gl_wgpu_format(format),
        format if format.starts_with("ktx2/vk-") => ktx2_vk_wgpu_format(format),
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

fn ktx_gl_wgpu_format(format: &str) -> Option<wgpu::TextureFormat> {
    let value = format.strip_prefix("ktx/gl-internal-0x")?;
    let gl_internal_format = u32::from_str_radix(value, 16).ok()?;
    match gl_internal_format {
        0x83f0 | 0x83f1 => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        0x8c4c | 0x8c4d => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        0x83f2 => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        0x8c4e => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        0x83f3 => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        0x8c4f => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        0x8dbb => Some(wgpu::TextureFormat::Bc4RUnorm),
        0x8dbc => Some(wgpu::TextureFormat::Bc4RSnorm),
        0x8dbd => Some(wgpu::TextureFormat::Bc5RgUnorm),
        0x8dbe => Some(wgpu::TextureFormat::Bc5RgSnorm),
        0x8e8f => Some(wgpu::TextureFormat::Bc6hRgbUfloat),
        0x8e8e => Some(wgpu::TextureFormat::Bc6hRgbFloat),
        0x8e8c => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        0x8e8d => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        0x9274 => Some(wgpu::TextureFormat::Etc2Rgb8Unorm),
        0x9275 => Some(wgpu::TextureFormat::Etc2Rgb8UnormSrgb),
        0x9276 => Some(wgpu::TextureFormat::Etc2Rgb8A1Unorm),
        0x9277 => Some(wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb),
        0x9278 => Some(wgpu::TextureFormat::Etc2Rgba8Unorm),
        0x9279 => Some(wgpu::TextureFormat::Etc2Rgba8UnormSrgb),
        0x9270 => Some(wgpu::TextureFormat::EacR11Unorm),
        0x9271 => Some(wgpu::TextureFormat::EacR11Snorm),
        0x9272 => Some(wgpu::TextureFormat::EacRg11Unorm),
        0x9273 => Some(wgpu::TextureFormat::EacRg11Snorm),
        0x93b0..=0x93bd => Some(wgpu::TextureFormat::Astc {
            block: ktx_gl_astc_block(gl_internal_format)?,
            channel: wgpu::AstcChannel::Unorm,
        }),
        0x93d0..=0x93dd => Some(wgpu::TextureFormat::Astc {
            block: ktx_gl_astc_block(gl_internal_format)?,
            channel: wgpu::AstcChannel::UnormSrgb,
        }),
        _ => None,
    }
}

fn ktx_gl_astc_block(gl_internal_format: u32) -> Option<wgpu::AstcBlock> {
    let index = if (0x93b0..=0x93bd).contains(&gl_internal_format) {
        gl_internal_format - 0x93b0
    } else if (0x93d0..=0x93dd).contains(&gl_internal_format) {
        gl_internal_format - 0x93d0
    } else {
        return None;
    };
    astc_block_by_index(index)
}

fn ktx2_vk_wgpu_format(format: &str) -> Option<wgpu::TextureFormat> {
    let vk_format = format
        .split('/')
        .find_map(|part| part.strip_prefix("vk-"))
        .and_then(|value| value.parse::<u32>().ok())?;
    match vk_format {
        131 | 133 => Some(wgpu::TextureFormat::Bc1RgbaUnorm),
        132 | 134 => Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb),
        135 => Some(wgpu::TextureFormat::Bc2RgbaUnorm),
        136 => Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb),
        137 => Some(wgpu::TextureFormat::Bc3RgbaUnorm),
        138 => Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb),
        139 => Some(wgpu::TextureFormat::Bc4RUnorm),
        140 => Some(wgpu::TextureFormat::Bc4RSnorm),
        141 => Some(wgpu::TextureFormat::Bc5RgUnorm),
        142 => Some(wgpu::TextureFormat::Bc5RgSnorm),
        143 => Some(wgpu::TextureFormat::Bc6hRgbUfloat),
        144 => Some(wgpu::TextureFormat::Bc6hRgbFloat),
        145 => Some(wgpu::TextureFormat::Bc7RgbaUnorm),
        146 => Some(wgpu::TextureFormat::Bc7RgbaUnormSrgb),
        147 => Some(wgpu::TextureFormat::Etc2Rgb8Unorm),
        148 => Some(wgpu::TextureFormat::Etc2Rgb8UnormSrgb),
        149 => Some(wgpu::TextureFormat::Etc2Rgb8A1Unorm),
        150 => Some(wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb),
        151 => Some(wgpu::TextureFormat::Etc2Rgba8Unorm),
        152 => Some(wgpu::TextureFormat::Etc2Rgba8UnormSrgb),
        153 => Some(wgpu::TextureFormat::EacR11Unorm),
        154 => Some(wgpu::TextureFormat::EacR11Snorm),
        155 => Some(wgpu::TextureFormat::EacRg11Unorm),
        156 => Some(wgpu::TextureFormat::EacRg11Snorm),
        157..=184 => {
            let (block, channel) = ktx2_astc_format(vk_format)?;
            Some(wgpu::TextureFormat::Astc { block, channel })
        }
        _ => None,
    }
}

fn ktx2_astc_format(vk_format: u32) -> Option<(wgpu::AstcBlock, wgpu::AstcChannel)> {
    if !(157..=184).contains(&vk_format) {
        return None;
    }
    let block = astc_block_by_index((vk_format - 157) / 2)?;
    let channel = if vk_format % 2 == 0 {
        wgpu::AstcChannel::UnormSrgb
    } else {
        wgpu::AstcChannel::Unorm
    };
    Some((block, channel))
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

fn astc_block_by_index(index: u32) -> Option<wgpu::AstcBlock> {
    Some(match index {
        0 => wgpu::AstcBlock::B4x4,
        1 => wgpu::AstcBlock::B5x4,
        2 => wgpu::AstcBlock::B5x5,
        3 => wgpu::AstcBlock::B6x5,
        4 => wgpu::AstcBlock::B6x6,
        5 => wgpu::AstcBlock::B8x5,
        6 => wgpu::AstcBlock::B8x6,
        7 => wgpu::AstcBlock::B8x8,
        8 => wgpu::AstcBlock::B10x5,
        9 => wgpu::AstcBlock::B10x6,
        10 => wgpu::AstcBlock::B10x8,
        11 => wgpu::AstcBlock::B10x10,
        12 => wgpu::AstcBlock::B12x10,
        13 => wgpu::AstcBlock::B12x12,
        _ => return None,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rgba8MipUpload {
    level: u32,
    layer: u32,
    width: u32,
    height: u32,
    offset: u64,
}

fn rgba8_mip_uploads(
    width: u32,
    height: u32,
    mip_level_count: u32,
    layer_count: u32,
) -> Vec<Rgba8MipUpload> {
    let mut offset = 0_u64;
    let mut uploads = Vec::new();
    for level in 0..mip_level_count {
        let level_width = mip_extent(width, level);
        let level_height = mip_extent(height, level);
        let level_size = rgba8_level_size_bytes(level_width, level_height);
        for layer in 0..layer_count {
            uploads.push(Rgba8MipUpload {
                level,
                layer,
                width: level_width,
                height: level_height,
                offset,
            });
            offset = offset.saturating_add(level_size);
        }
    }
    uploads
}

fn texture_view_descriptor(
    descriptor: &RenderImageDescriptor,
) -> wgpu::TextureViewDescriptor<'static> {
    match descriptor.dimension {
        RenderImageDimension::D1 => wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        },
        RenderImageDimension::D2 => {
            let layer_count = descriptor.array_layer_count.max(1);
            wgpu::TextureViewDescriptor {
                dimension: Some(if layer_count > 1 {
                    wgpu::TextureViewDimension::D2Array
                } else {
                    wgpu::TextureViewDimension::D2
                }),
                base_array_layer: 0,
                array_layer_count: Some(layer_count),
                ..Default::default()
            }
        }
        RenderImageDimension::D3 => wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        },
        RenderImageDimension::Cube => {
            let layer_count = descriptor.array_layer_count.max(6);
            wgpu::TextureViewDescriptor {
                dimension: Some(if layer_count > 6 {
                    wgpu::TextureViewDimension::CubeArray
                } else {
                    wgpu::TextureViewDimension::Cube
                }),
                base_array_layer: 0,
                array_layer_count: Some(layer_count),
                ..Default::default()
            }
        }
    }
}

fn lightmap_texture_view_descriptor(layer_count: u32) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        base_array_layer: 0,
        array_layer_count: Some(layer_count.max(1)),
        ..Default::default()
    }
}

fn lightmap_legacy_bind_group_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2),
        base_array_layer: 0,
        array_layer_count: Some(1),
        ..Default::default()
    }
}

fn rgba8_level_size_bytes(width: u32, height: u32) -> u64 {
    u64::from(width)
        .saturating_mul(u64::from(height))
        .saturating_mul(4)
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

#[cfg(test)]
mod tests;
