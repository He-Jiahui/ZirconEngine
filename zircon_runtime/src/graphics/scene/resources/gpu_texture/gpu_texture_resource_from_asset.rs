use core::ops::Range;
use std::sync::Arc;

mod compressed_mip_upload;
mod texture_format;

use crate::asset::assets::{
    TextureAsset, TextureUploadCompressionFamily, TextureUploadPlan, TextureUploadReadiness,
    LIGHTMAP_RGBA16F_GPU_FORMAT,
};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageDimension, TextureMipPolicy,
};
use crate::core::resource::ResourceId;
use crate::graphics::scene::scene_renderer::mip_gen::{
    MipGenColorMode, MipGenDispatchPlan, RuntimeMipGenPass,
};
use crate::graphics::types::GraphicsError;

pub(crate) use self::texture_format::texture_upload_support_from_device;
use self::texture_format::{compressed_wgpu_format, rgba8_wgpu_format, wgpu_texture_usages};
#[cfg(test)]
use super::sampler_cache::{
    sampler_descriptor, sampler_descriptor_for_image,
    sampler_descriptor_for_image_with_anisotropy_cap,
};
use super::sampler_cache::{sanitized_anisotropy_clamp, sanitized_anisotropy_clamp_with_cap};
use super::{GpuTextureResource, TextureSamplerCache};
use compressed_mip_upload::upload_compressed_texture_bytes;

impl GpuTextureResource {
    pub(crate) fn from_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
        runtime_mip_gen_pass: &RuntimeMipGenPass,
    ) -> Result<Self, GraphicsError> {
        let support = texture_upload_support_from_device(device);
        match payload.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan }
                if plan.compression == TextureUploadCompressionFamily::Uncompressed
                    && plan.format == LIGHTMAP_RGBA16F_GPU_FORMAT =>
            {
                Self::from_lightmap_rgba16f_asset(
                    device,
                    queue,
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
                    queue,
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
                queue,
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
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        payload: TextureAsset,
        previous: &GpuTextureResource,
        previous_range: Range<u8>,
        requested_range: Range<u8>,
    ) -> Result<Self, GraphicsError> {
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
            queue,
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
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
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
        Ok(Self {
            id: Some(id),
            descriptor,
            texture,
            view,
            sampler,
            sampler_cache,
            mip_streaming_supported: false,
            resident_texture_bytes: page_size_bytes.saturating_mul(u64::from(layer_count)),
            bind_group,
        })
    }

    fn from_rgba8_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
        plan: TextureUploadPlan,
        runtime_mip_gen_pass: &RuntimeMipGenPass,
    ) -> Result<Self, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
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
                payload.uri
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
                width: payload.width,
                height: payload.height,
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
        for upload in rgba8_mip_uploads(
            payload.width,
            payload.height,
            uploaded_mip_level_count,
            layer_count,
        ) {
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
        if runtime_mip_generation {
            let mip_plan = MipGenDispatchPlan::new(
                payload.width,
                payload.height,
                layer_count,
                mip_level_count,
            )
            .map_err(|error| {
                GraphicsError::Asset(format!(
                    "runtime mip generation plan for texture {} is invalid: {error}",
                    payload.uri
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
            queue.submit(std::iter::once(encoder.finish()));
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
        Ok(Self {
            id: Some(id),
            descriptor,
            texture,
            view,
            sampler,
            sampler_cache,
            mip_streaming_supported,
            resident_texture_bytes: Self::rgba8_mip_chain_bytes(
                payload.width,
                payload.height,
                layer_count,
                0..mip_level_count,
            ),
            bind_group,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn from_rgba8_asset_resident_mips(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
        id: ResourceId,
        payload: TextureAsset,
        plan: TextureUploadPlan,
        previous: &GpuTextureResource,
        previous_range: Range<u8>,
        requested_range: Range<u8>,
    ) -> Result<Self, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
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
                width: mip_extent(payload.width, requested_range.start),
                height: mip_extent(payload.height, requested_range.start),
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
        if copy_start < copy_end {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-streamed-texture-mip-copy"),
            });
            for source_level in copy_start..copy_end {
                let extent = wgpu::Extent3d {
                    width: mip_extent(payload.width, source_level),
                    height: mip_extent(payload.height, source_level),
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
            queue.submit(std::iter::once(encoder.finish()));
        }
        for upload in rgba8_resident_mip_uploads(
            payload.width,
            payload.height,
            mip_count,
            layer_count,
            requested_range.clone(),
            previous_range,
        ) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: upload.destination_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: upload.source.layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &payload.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: upload.source.offset,
                    bytes_per_row: Some(4 * upload.source.width),
                    rows_per_image: Some(upload.source.height),
                },
                wgpu::Extent3d {
                    width: upload.source.width,
                    height: upload.source.height,
                    depth_or_array_layers: 1,
                },
            );
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
        Ok(Self {
            id: Some(id),
            descriptor,
            texture,
            view,
            sampler,
            sampler_cache,
            mip_streaming_supported: true,
            resident_texture_bytes: Self::rgba8_mip_chain_bytes(
                payload.width,
                payload.height,
                layer_count,
                u32::from(requested_range.start)..u32::from(requested_range.end),
            ),
            bind_group,
        })
    }

    fn from_compressed_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        sampler_cache: Arc<TextureSamplerCache>,
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
        upload_compressed_texture_bytes(
            queue,
            &texture,
            &payload.uri,
            payload.width,
            payload.height,
            descriptor.mip_count.max(1),
            depth_or_array_layers,
            data,
            upload_bytes,
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
        Ok(Self {
            id: Some(id),
            descriptor,
            texture,
            view,
            sampler,
            sampler_cache,
            mip_streaming_supported: false,
            resident_texture_bytes: u64::try_from(upload_bytes.len()).unwrap_or(u64::MAX),
            bind_group,
        })
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
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

#[cfg(test)]
mod tests;
