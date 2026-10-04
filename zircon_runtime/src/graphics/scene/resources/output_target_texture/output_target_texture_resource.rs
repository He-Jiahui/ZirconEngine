use crate::asset::{TextureAsset, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT};
use crate::core::framework::render::{
    RenderImageDescriptor, RenderImageShape, RenderImageUsage, RenderSamplerAddressMode,
    RenderSamplerFilter, TextureMetadata, TextureViewKind,
};
use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

const OUTPUT_TARGET_TEXTURE_LABEL: &str = "zircon-output-target-texture";

pub(in crate::graphics::scene) struct OutputTargetTextureResource {
    descriptor: RenderImageDescriptor,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl OutputTargetTextureResource {
    pub(in crate::graphics::scene) const RETAINED_OUTPUT_TARGET_TEXTURE_OWNER_COUNT: usize = 4;

    pub(in crate::graphics::scene) fn retained_output_target_texture_owner_count(&self) -> usize {
        let _retained_output_target_texture_owners =
            (&self.descriptor, &self.texture, &self.view, &self.sampler);
        Self::RETAINED_OUTPUT_TARGET_TEXTURE_OWNER_COUNT
    }

    pub(in crate::graphics::scene::resources) fn from_asset(
        device: &wgpu::Device,
        id: ResourceId,
        payload: TextureAsset,
    ) -> Result<Self, GraphicsError> {
        let descriptor = payload.render_image_descriptor();
        let shape = validate_output_target_descriptor(id, &descriptor)?;
        let format = output_target_wgpu_format(&descriptor).ok_or_else(|| {
            GraphicsError::Asset(format!(
                "output target texture {id} has unsupported render target format {}",
                descriptor.format
            ))
        })?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(OUTPUT_TARGET_TEXTURE_LABEL),
            size: wgpu::Extent3d {
                width: shape.extent.width,
                height: shape.extent.height,
                depth_or_array_layers: shape.extent.depth_or_array_layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: output_target_texture_usages(&descriptor, format),
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&sampler_descriptor(&descriptor.sampler));

        Ok(Self {
            descriptor,
            texture,
            view,
            sampler,
        })
    }

    pub(in crate::graphics::scene) fn descriptor(&self) -> &RenderImageDescriptor {
        self.assert_retained_output_target_texture_owner_count();
        &self.descriptor
    }

    pub(in crate::graphics::scene) fn size(&self) -> crate::core::math::UVec2 {
        let descriptor = self.descriptor();
        crate::core::math::UVec2::new(descriptor.width, descriptor.height)
    }

    pub(in crate::graphics::scene) fn texture(&self) -> &wgpu::Texture {
        self.assert_retained_output_target_texture_owner_count();
        &self.texture
    }

    pub(in crate::graphics::scene) fn view(&self) -> &wgpu::TextureView {
        self.assert_retained_output_target_texture_owner_count();
        &self.view
    }

    pub(in crate::graphics::scene) fn sampler(&self) -> &wgpu::Sampler {
        self.assert_retained_output_target_texture_owner_count();
        &self.sampler
    }

    pub(in crate::graphics::scene) fn graph_texture_desc(
        &self,
        label: &str,
    ) -> Result<TextureDesc, GraphicsError> {
        let format = output_target_rhi_format(&self.descriptor).ok_or_else(|| {
            GraphicsError::Asset(format!(
                "output target texture graph binding has unsupported format {}",
                self.descriptor.format
            ))
        })?;
        Ok(TextureDesc::new(
            label,
            self.descriptor.width,
            self.descriptor.height,
            format,
            output_target_rhi_usages(&self.descriptor, format),
        ))
    }

    fn assert_retained_output_target_texture_owner_count(&self) {
        debug_assert_eq!(
            self.retained_output_target_texture_owner_count(),
            Self::RETAINED_OUTPUT_TARGET_TEXTURE_OWNER_COUNT,
            "OutputTargetTextureResource must retain descriptor, texture, view, and sampler while exposing output target writeback and graph-import bindings",
        );
    }
}

fn validate_output_target_descriptor(
    id: ResourceId,
    descriptor: &RenderImageDescriptor,
) -> Result<RenderImageShape, GraphicsError> {
    let shape = descriptor.validated_shape().map_err(|error| {
        GraphicsError::Asset(format!(
            "output target texture {id} has invalid storage/view shape metadata: {error}"
        ))
    })?;
    if shape.view_kind != TextureViewKind::D2 || descriptor.mip_count != 1 {
        return Err(GraphicsError::Asset(format!(
            "output target texture {id} must be a 2d single-layer single-mip texture"
        )));
    }
    if output_target_wgpu_format(descriptor).is_none() {
        return Err(GraphicsError::Asset(format!(
            "output target texture {id} must use a renderable rgba8 format"
        )));
    }
    if !descriptor.usage.contains(&RenderImageUsage::RenderTarget) {
        return Err(GraphicsError::Asset(format!(
            "output target texture {id} must include render_target usage"
        )));
    }
    Ok(shape)
}

fn output_target_texture_usages(
    descriptor: &RenderImageDescriptor,
    format: wgpu::TextureFormat,
) -> wgpu::TextureUsages {
    // Camera output targets are terminal graph products and remain readable by
    // composition passes even when the asset author only requests render-target use.
    let mut usages = wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::COPY_DST
        | wgpu::TextureUsages::TEXTURE_BINDING;
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
    usages
}

fn output_target_wgpu_format(descriptor: &RenderImageDescriptor) -> Option<wgpu::TextureFormat> {
    let format = descriptor.format.trim();
    if format.eq_ignore_ascii_case(RGBA8_UNORM_FORMAT) {
        Some(wgpu::TextureFormat::Rgba8Unorm)
    } else if format.eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT) {
        Some(wgpu::TextureFormat::Rgba8UnormSrgb)
    } else {
        None
    }
}

fn output_target_rhi_format(descriptor: &RenderImageDescriptor) -> Option<TextureFormat> {
    let format = descriptor.format.trim();
    if format.eq_ignore_ascii_case(RGBA8_UNORM_FORMAT) {
        Some(TextureFormat::Rgba8Unorm)
    } else if format.eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT) {
        Some(TextureFormat::Rgba8UnormSrgb)
    } else {
        None
    }
}

fn output_target_rhi_usages(
    descriptor: &RenderImageDescriptor,
    format: TextureFormat,
) -> TextureUsage {
    let mut usages = TextureUsage::COPY_SRC | TextureUsage::COPY_DST | TextureUsage::SAMPLED;
    for usage in &descriptor.usage {
        match usage {
            RenderImageUsage::Sampled => usages |= TextureUsage::SAMPLED,
            RenderImageUsage::Storage if format.supports_write_only_storage() => {
                usages |= TextureUsage::STORAGE;
            }
            RenderImageUsage::Storage => {}
            RenderImageUsage::RenderTarget => usages |= TextureUsage::RENDER_ATTACHMENT,
            RenderImageUsage::CopySrc => usages |= TextureUsage::COPY_SRC,
            RenderImageUsage::CopyDst => usages |= TextureUsage::COPY_DST,
        }
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

#[cfg(test)]
#[path = "tests/output_target_texture_resource.rs"]
mod tests;
