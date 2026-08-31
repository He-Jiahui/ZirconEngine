use core::ops::Range;
use std::sync::Arc;

mod compressed_mip_upload;
mod texture_format;
mod upload_work;

use crate::asset::assets::{
    LIGHTMAP_RGBA16F_GPU_FORMAT, TextureAsset, TextureUploadCompressionFamily, TextureUploadPlan,
    TextureUploadReadiness,
};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension, TextureMipPolicy,
};
use crate::core::resource::ResourceId;
use crate::graphics::scene::scene_renderer::mip_gen::{
    MipGenColorMode, MipGenDispatchPlan, RuntimeMipGenPass,
};
use crate::graphics::types::GraphicsError;
use zr_rhi::TextureCopyRegion;
use zr_rhi_wgpu::{WgpuTextureUpload, WgpuTextureUploadBatch};

pub(crate) use self::texture_format::texture_upload_support_from_device;
use self::texture_format::{compressed_wgpu_format, rgba8_wgpu_format, wgpu_texture_usages};
#[cfg(test)]
use super::sampler_cache::{
    sampler_descriptor, sampler_descriptor_for_image,
    sampler_descriptor_for_image_with_anisotropy_cap,
};
use super::sampler_cache::{sanitized_anisotropy_clamp, sanitized_anisotropy_clamp_with_cap};
use super::{GpuTextureResource, TextureSamplerCache};
use compressed_mip_upload::enqueue_compressed_texture_uploads;
pub(in crate::graphics::scene::resources) use upload_work::GpuTextureUploadWork;

impl GpuTextureResource {
    pub(in crate::graphics::scene::resources) fn from_asset(
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
        runtime_mip_gen_pass: &RuntimeMipGenPass,
    ) -> Result<GpuTextureUploadWork, GraphicsError> {
        let support = texture_upload_support_from_device(device);
        match payload.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan }
                if plan.compression == TextureUploadCompressionFamily::Uncompressed
                    && plan.format == LIGHTMAP_RGBA16F_GPU_FORMAT =>
            {
                Self::from_lightmap_rgba16f_asset(
                    device,
                    texture_layout,
                    sampler_cache,
                    id,
                    payload,
                )
            }
            TextureUploadReadiness::Ready { plan }
                if plan.compression == TextureUploadCompressionFamily::Uncompressed =>
            {
                Self::from_rgba8_asset(
                    device,
                    texture_layout,
                    sampler_cache,
                    id,
                    payload,
                    plan,
                    runtime_mip_gen_pass,
                )
            }
            TextureUploadReadiness::Ready { plan } => Self::from_compressed_asset(
                device,
                texture_layout,
                sampler_cache,
                id,
                payload,
                plan,
            ),
            TextureUploadReadiness::Unsupported { reason } => Err(GraphicsError::Asset(format!(
                "texture {} is not upload-ready: {reason}",
                payload.uri
            ))),
        }
    }

    /// Rebuild an uncompressed source-mip texture as a smaller physical tail range.
    /// Common source mips are copied from the prior texture; only newly required mips are
    /// reuploaded from the retained asset payload.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::resources) fn rebuild_resident_mips(
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
        previous: &GpuTextureResource,
        previous_range: Range<u8>,
        requested_range: Range<u8>,
    ) -> Result<GpuTextureUploadWork, GraphicsError> {
        if !previous.supports_mip_streaming() {
            return Err(GraphicsError::Asset(format!(
                "texture {} does not support physical mip streaming",
                payload.uri
            )));
        }
        let support = texture_upload_support_from_device(device);
        let plan = match payload.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan }
                if plan.compression == TextureUploadCompressionFamily::Uncompressed =>
            {
                plan
            }
            TextureUploadReadiness::Ready { .. } => {
                return Err(GraphicsError::Asset(format!(
                    "texture {} uses a compressed payload that cannot yet rebuild partial mip ranges",
                    payload.uri
                )));
            }
            TextureUploadReadiness::Unsupported { reason } => {
                return Err(GraphicsError::Asset(format!(
                    "texture {} is not upload-ready: {reason}",
                    payload.uri
                )));
            }
        };
        Self::from_rgba8_asset_resident_mips(
            device,
            texture_layout,
            Arc::clone(&previous.sampler_cache),
            id,
            payload,
            plan,
            previous,
            previous_range,
            requested_range,
        )
    }

    fn from_lightmap_rgba16f_asset(
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
    ) -> Result<GpuTextureUploadWork, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
        let texture_uri = payload.uri.clone();
        let width = payload.width;
        let height = payload.height;
        let layer_count = descriptor.array_layer_count.max(1);
        let crate::asset::TexturePayload::Container { bytes, .. } = payload.payload else {
            return Err(GraphicsError::Asset(format!(
                "lightmap texture {} requires a raw rgba16f container payload",
                texture_uri
            )));
        };
        let upload_payload: Arc<[u8]> = Arc::from(bytes);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-lightmap-rgba16f-array"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: layer_count,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu_texture_usages(&descriptor, wgpu::TextureFormat::Rgba16Float, true),
            view_formats: &[],
        });
        let page_size_bytes = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|texels| texels.checked_mul(8))
            .ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "lightmap texture {} upload size overflows",
                    texture_uri
                ))
            })?;
        let mut upload_batch = WgpuTextureUploadBatch::new();
        for layer in 0..layer_count {
            let start = usize::try_from(page_size_bytes * u64::from(layer)).map_err(|_| {
                GraphicsError::Asset(format!(
                    "lightmap texture {} upload source offset overflows",
                    texture_uri
                ))
            })?;
            let end = start
                .checked_add(usize::try_from(page_size_bytes).map_err(|_| {
                    GraphicsError::Asset(format!(
                        "lightmap texture {} upload size overflows",
                        texture_uri
                    ))
                })?)
                .ok_or_else(|| {
                    GraphicsError::Asset(format!(
                        "lightmap texture {} upload source range overflows",
                        texture_uri
                    ))
                })?;
            enqueue_texture_upload(
                &mut upload_batch,
                texture.clone(),
                TextureCopyRegion::new(width, height).with_origin(0, 0, layer),
                8 * width,
                height,
                Arc::clone(&upload_payload),
                start..end,
                &texture_uri,
            )?;
        }
        let view = texture.create_view(&lightmap_texture_view_descriptor(layer_count));
        let page_zero_bind_group_view =
            texture.create_view(&lightmap_page_zero_bind_group_view_descriptor());
        let sampler = sampler_cache.sampler_for_image(device, &descriptor);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-lightmap-rgba16f-bind-group"),
            layout: texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&page_zero_bind_group_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler.as_ref()),
                },
            ],
        });
        Ok(GpuTextureUploadWork::new(
            Self {
                id: Some(id),
                descriptor,
                texture,
                view,
                sampler,
                sampler_cache,
                mip_streaming_supported: false,
                resident_texture_bytes: page_size_bytes.saturating_mul(u64::from(layer_count)),
                bind_group,
            },
            upload_batch,
            Vec::new(),
            Vec::new(),
        ))
    }

    fn from_rgba8_asset(
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
        plan: TextureUploadPlan,
        runtime_mip_gen_pass: &RuntimeMipGenPass,
    ) -> Result<GpuTextureUploadWork, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
        let texture_uri = payload.uri.clone();
        let width = payload.width;
        let height = payload.height;
        let mip_level_count = descriptor.mip_count.max(1);
        let layer_count = descriptor.depth_or_array_layers.max(1);
        let runtime_mip_generation = descriptor.metadata.mip_policy
            == TextureMipPolicy::GenerateRuntime
            && mip_level_count > 1;
        let mip_streaming_supported = supports_physical_mip_streaming(&descriptor, &plan);
        if runtime_mip_generation
            && !matches!(
                descriptor.dimension,
                RenderImageDimension::D2 | RenderImageDimension::Cube
            )
        {
            return Err(GraphicsError::Asset(format!(
                "runtime mip generation supports only 2d or cube textures: {}",
                texture_uri
            )));
        }
        let format = if runtime_mip_generation {
            wgpu::TextureFormat::Rgba8Unorm
        } else {
            rgba8_wgpu_format(&plan.format)
        };
        let runtime_srgb_view_formats = [wgpu::TextureFormat::Rgba8UnormSrgb];
        let view_formats: &[wgpu::TextureFormat] = if runtime_mip_generation
            && descriptor.metadata.color_space == RenderImageColorSpace::Srgb
        {
            &runtime_srgb_view_formats
        } else {
            &[]
        };
        let mut usage = wgpu_texture_usages(&descriptor, format, true);
        if mip_streaming_supported {
            usage |= wgpu::TextureUsages::COPY_SRC;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: layer_count,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu_dimension(descriptor.dimension),
            format,
            usage,
            view_formats,
        });
        let uploaded_mip_level_count = if runtime_mip_generation {
            1
        } else {
            mip_level_count
        };
        let upload_payload: Arc<[u8]> = Arc::from(payload.rgba);
        let mut upload_batch = WgpuTextureUploadBatch::new();
        for upload in rgba8_mip_uploads(width, height, uploaded_mip_level_count, layer_count) {
            let byte_len = rgba8_level_size_bytes(upload.width, upload.height);
            let end = upload.offset.checked_add(byte_len).ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {} mip upload source range overflows",
                    texture_uri
                ))
            })?;
            enqueue_texture_upload(
                &mut upload_batch,
                texture.clone(),
                TextureCopyRegion::new(upload.width, upload.height)
                    .with_mip_level(upload.level)
                    .with_origin(0, 0, upload.layer),
                4 * upload.width,
                upload.height,
                Arc::clone(&upload_payload),
                byte_range(upload.offset, end, &texture_uri)?,
                &texture_uri,
            )?;
        }
        let mut post_upload_commands = Vec::new();
        if runtime_mip_generation {
            let mip_plan = MipGenDispatchPlan::new(width, height, layer_count, mip_level_count)
                .map_err(|error| {
                    GraphicsError::Asset(format!(
                        "runtime mip generation plan for texture {} is invalid: {error}",
                        texture_uri
                    ))
                })?;
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-runtime-mip-gen-encoder"),
            });
            runtime_mip_gen_pass.encode(
                device,
                &mut encoder,
                &texture,
                &mip_plan,
                MipGenColorMode::from_metadata(&descriptor.metadata),
            );
            post_upload_commands.push(encoder.finish());
        }
        let view = texture.create_view(&texture_view_descriptor(&descriptor));
        let sampler = sampler_cache.sampler_for_image(device, &descriptor);
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
                    resource: wgpu::BindingResource::Sampler(sampler.as_ref()),
                },
            ],
        });
        Ok(GpuTextureUploadWork::new(
            Self {
                id: Some(id),
                descriptor,
                texture,
                view,
                sampler,
                sampler_cache,
                mip_streaming_supported,
                resident_texture_bytes: Self::rgba8_mip_chain_bytes(
                    width,
                    height,
                    layer_count,
                    0..mip_level_count,
                ),
                bind_group,
            },
            upload_batch,
            Vec::new(),
            post_upload_commands,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn from_rgba8_asset_resident_mips(
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
        plan: TextureUploadPlan,
        previous: &GpuTextureResource,
        previous_range: Range<u8>,
        requested_range: Range<u8>,
    ) -> Result<GpuTextureUploadWork, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
        let texture_uri = payload.uri.clone();
        let width = payload.width;
        let height = payload.height;
        if !supports_physical_mip_streaming(&descriptor, &plan) {
            return Err(GraphicsError::Asset(format!(
                "texture {} is not eligible for physical mip streaming",
                payload.uri
            )));
        }
        if previous.descriptor != descriptor {
            return Err(GraphicsError::Asset(format!(
                "texture {} changed descriptor while rebuilding mip residency",
                payload.uri
            )));
        }
        let mip_count = descriptor.mip_count.clamp(1, u32::from(u8::MAX));
        let previous_range =
            normalize_resident_mip_range(mip_count, previous_range).ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {} has an invalid prior mip residency range",
                    payload.uri
                ))
            })?;
        let requested_range =
            normalize_resident_mip_range(mip_count, requested_range).ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {} has an invalid requested mip residency range",
                    payload.uri
                ))
            })?;
        let layer_count = descriptor.depth_or_array_layers.max(1);
        let format = rgba8_wgpu_format(&plan.format);
        let mut usage = wgpu_texture_usages(&descriptor, format, true);
        usage |= wgpu::TextureUsages::COPY_SRC;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-streamed-texture"),
            size: wgpu::Extent3d {
                width: mip_extent(width, requested_range.start),
                height: mip_extent(height, requested_range.start),
                depth_or_array_layers: layer_count,
            },
            mip_level_count: requested_range.end - requested_range.start,
            sample_count: 1,
            dimension: wgpu_dimension(descriptor.dimension),
            format,
            usage,
            view_formats: &[],
        });

        let copy_start = previous_range.start.max(requested_range.start);
        let copy_end = previous_range.end.min(requested_range.end);
        let mut pre_upload_commands = Vec::new();
        if copy_start < copy_end {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-streamed-texture-mip-copy"),
            });
            for source_level in copy_start..copy_end {
                let extent = wgpu::Extent3d {
                    width: mip_extent(width, source_level),
                    height: mip_extent(height, source_level),
                    depth_or_array_layers: 1,
                };
                for layer in 0..layer_count {
                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &previous.texture,
                            mip_level: source_level - previous_range.start,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: layer,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        wgpu::TexelCopyTextureInfo {
                            texture: &texture,
                            mip_level: source_level - requested_range.start,
                            origin: wgpu::Origin3d {
                                x: 0,
                                y: 0,
                                z: layer,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        extent,
                    );
                }
            }
            pre_upload_commands.push(encoder.finish());
        }
        let upload_payload: Arc<[u8]> = Arc::from(payload.rgba);
        let mut upload_batch = WgpuTextureUploadBatch::new();
        for upload in rgba8_resident_mip_uploads(
            width,
            height,
            mip_count,
            layer_count,
            requested_range.clone(),
            previous_range,
        ) {
            let byte_len = rgba8_level_size_bytes(upload.source.width, upload.source.height);
            let end = upload.source.offset.checked_add(byte_len).ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "texture {} streamed mip upload source range overflows",
                    texture_uri
                ))
            })?;
            enqueue_texture_upload(
                &mut upload_batch,
                texture.clone(),
                TextureCopyRegion::new(upload.source.width, upload.source.height)
                    .with_mip_level(upload.destination_level)
                    .with_origin(0, 0, upload.source.layer),
                4 * upload.source.width,
                upload.source.height,
                Arc::clone(&upload_payload),
                byte_range(upload.source.offset, end, &texture_uri)?,
                &texture_uri,
            )?;
        }
        let view = texture.create_view(&texture_view_descriptor(&descriptor));
        let sampler = sampler_cache.sampler_for_image(device, &descriptor);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-streamed-texture-bind-group"),
            layout: texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler.as_ref()),
                },
            ],
        });
        Ok(GpuTextureUploadWork::new(
            Self {
                id: Some(id),
                descriptor,
                texture,
                view,
                sampler,
                sampler_cache,
                mip_streaming_supported: true,
                resident_texture_bytes: Self::rgba8_mip_chain_bytes(
                    width,
                    height,
                    layer_count,
                    u32::from(requested_range.start)..u32::from(requested_range.end),
                ),
                bind_group,
            },
            upload_batch,
            pre_upload_commands,
            Vec::new(),
        ))
    }

    fn from_compressed_asset(
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
        plan: TextureUploadPlan,
    ) -> Result<GpuTextureUploadWork, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
        let texture_uri = payload.uri.clone();
        let width = payload.width;
        let height = payload.height;
        let format = compressed_wgpu_format(&plan, descriptor.color_space).ok_or_else(|| {
            GraphicsError::Asset(format!(
                "texture {} has unsupported upload format {}",
                texture_uri, plan.format
            ))
        })?;
        let data = match payload.payload {
            crate::asset::TexturePayload::Container { bytes, .. } => Arc::<[u8]>::from(bytes),
            crate::asset::TexturePayload::Rgba8 => {
                return Err(GraphicsError::Asset(format!(
                    "texture {} was planned as compressed but has rgba payload",
                    texture_uri
                )));
            }
        };
        let depth_or_array_layers = descriptor.depth_or_array_layers.max(1);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-compressed-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers,
            },
            mip_level_count: descriptor.mip_count.max(1),
            sample_count: 1,
            dimension: wgpu_dimension(descriptor.dimension),
            format,
            usage: wgpu_texture_usages(&descriptor, format, true),
            view_formats: &[],
        });
        let resident_texture_bytes = plan
            .data_length
            .map(|bytes| u64::try_from(bytes).unwrap_or(u64::MAX))
            .unwrap_or_else(|| {
                u64::try_from(data.len().saturating_sub(plan.data_offset)).unwrap_or(u64::MAX)
            });
        let mut upload_batch = WgpuTextureUploadBatch::new();
        enqueue_compressed_texture_uploads(
            &mut upload_batch,
            &texture,
            &texture_uri,
            width,
            height,
            descriptor.mip_count.max(1),
            depth_or_array_layers,
            data,
            &plan,
        )?;
        let view = texture.create_view(&texture_view_descriptor(&descriptor));
        let sampler = sampler_cache.sampler_for_image(device, &descriptor);
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
                    resource: wgpu::BindingResource::Sampler(sampler.as_ref()),
                },
            ],
        });
        Ok(GpuTextureUploadWork::new(
            Self {
                id: Some(id),
                descriptor,
                texture,
                view,
                sampler,
                sampler_cache,
                mip_streaming_supported: false,
                resident_texture_bytes,
                bind_group,
            },
            upload_batch,
            Vec::new(),
            Vec::new(),
        ))
    }
}

impl GpuTextureResource {
    pub(in crate::graphics::scene) fn sampler_variant_for_max_anisotropy(
        &self,
        device: &wgpu::Device,
        max_anisotropy: u8,
    ) -> Option<Arc<wgpu::Sampler>> {
        let base_anisotropy = sanitized_anisotropy_clamp(&self.descriptor);
        let effective_anisotropy =
            sanitized_anisotropy_clamp_with_cap(&self.descriptor, max_anisotropy);
        if effective_anisotropy == base_anisotropy {
            return None;
        }

        Some(self.sampler_cache.sampler_for_image_with_anisotropy_cap(
            device,
            &self.descriptor,
            effective_anisotropy as u8,
        ))
    }
}

fn byte_range<T: std::fmt::Display + ?Sized>(
    start: u64,
    end: u64,
    texture_uri: &T,
) -> Result<Range<usize>, GraphicsError> {
    let start = usize::try_from(start).map_err(|_| {
        GraphicsError::Asset(format!(
            "texture {texture_uri} upload source offset overflows"
        ))
    })?;
    let end = usize::try_from(end).map_err(|_| {
        GraphicsError::Asset(format!(
            "texture {texture_uri} upload source range overflows"
        ))
    })?;
    (start <= end).then_some(start..end).ok_or_else(|| {
        GraphicsError::Asset(format!(
            "texture {texture_uri} upload source range is invalid"
        ))
    })
}

#[allow(clippy::too_many_arguments)]
fn enqueue_texture_upload<T: std::fmt::Display + ?Sized>(
    batch: &mut WgpuTextureUploadBatch,
    texture: wgpu::Texture,
    region: TextureCopyRegion,
    bytes_per_row: u32,
    rows_per_image: u32,
    payload: Arc<[u8]>,
    source_range: Range<usize>,
    texture_uri: &T,
) -> Result<(), GraphicsError> {
    let upload = WgpuTextureUpload::new(
        texture,
        region,
        bytes_per_row,
        rows_per_image,
        payload,
        source_range,
    )
    .ok_or_else(|| {
        GraphicsError::Asset(format!(
            "texture {texture_uri} upload source range is invalid"
        ))
    })?;
    batch.push(upload);
    Ok(())
}

fn wgpu_dimension(dimension: RenderImageDimension) -> wgpu::TextureDimension {
    match dimension {
        RenderImageDimension::D1 => wgpu::TextureDimension::D1,
        RenderImageDimension::D2 => wgpu::TextureDimension::D2,
        RenderImageDimension::D3 => wgpu::TextureDimension::D3,
        RenderImageDimension::Cube => wgpu::TextureDimension::D2,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ResidentRgba8MipUpload {
    source: Rgba8MipUpload,
    destination_level: u32,
}

fn rgba8_mip_uploads(
    width: u32,
    height: u32,
    mip_level_count: u32,
    layer_count: u32,
) -> impl Iterator<Item = Rgba8MipUpload> {
    let mut level = 0_u32;
    let mut layer = 0_u32;
    let mut offset = 0_u64;
    std::iter::from_fn(move || {
        if level >= mip_level_count || layer_count == 0 {
            return None;
        }
        let level_width = mip_extent(width, level);
        let level_height = mip_extent(height, level);
        let level_size = rgba8_level_size_bytes(level_width, level_height);
        let upload = Rgba8MipUpload {
            level,
            layer,
            width: level_width,
            height: level_height,
            offset,
        };
        offset = offset.saturating_add(level_size);
        layer += 1;
        if layer == layer_count {
            layer = 0;
            level += 1;
        }
        Some(upload)
    })
}

fn rgba8_resident_mip_uploads(
    width: u32,
    height: u32,
    mip_count: u32,
    layer_count: u32,
    requested_range: Range<u32>,
    previous_range: Range<u32>,
) -> Vec<ResidentRgba8MipUpload> {
    rgba8_mip_uploads(width, height, mip_count, layer_count)
        .filter(|upload| {
            requested_range.contains(&upload.level) && !previous_range.contains(&upload.level)
        })
        .map(|source| ResidentRgba8MipUpload {
            destination_level: source.level - requested_range.start,
            source,
        })
        .collect()
}

fn normalize_resident_mip_range(mip_count: u32, range: Range<u8>) -> Option<Range<u32>> {
    let mip_count = mip_count.clamp(1, u32::from(u8::MAX));
    let start = u32::from(range.start);
    let end = u32::from(range.end);
    (start < end && end == mip_count && start < mip_count).then_some(start..end)
}

fn supports_physical_mip_streaming(
    descriptor: &RenderImageDescriptor,
    plan: &TextureUploadPlan,
) -> bool {
    descriptor.metadata.allows_mip_streaming(
        descriptor.width,
        descriptor.height,
        descriptor.mip_count,
    ) && plan.compression == TextureUploadCompressionFamily::Uncompressed
        && matches!(
            descriptor.dimension,
            RenderImageDimension::D2 | RenderImageDimension::Cube
        )
}

fn texture_view_descriptor(
    descriptor: &RenderImageDescriptor,
) -> wgpu::TextureViewDescriptor<'static> {
    let format = (descriptor.metadata.mip_policy == TextureMipPolicy::GenerateRuntime
        && descriptor.metadata.color_space == RenderImageColorSpace::Srgb)
        .then_some(wgpu::TextureFormat::Rgba8UnormSrgb);
    match descriptor.dimension {
        RenderImageDimension::D1 => wgpu::TextureViewDescriptor {
            format,
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        },
        RenderImageDimension::D2 => {
            let layer_count = descriptor.array_layer_count.max(1);
            wgpu::TextureViewDescriptor {
                format,
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
            format,
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        },
        RenderImageDimension::Cube => {
            let layer_count = descriptor.array_layer_count.max(6);
            wgpu::TextureViewDescriptor {
                format,
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

fn lightmap_page_zero_bind_group_view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
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
    if shifted == 0 { 1 } else { shifted }
}

#[cfg(test)]
mod tests;
