use crate::asset::{RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT, TextureAsset};
use crate::core::framework::render::{
    RenderImageDescriptor, RenderImageDimension, RenderImageUsage, RenderSamplerAddressMode,
    RenderSamplerFilter, TextureMetadata,
};
use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;

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
        validate_output_target_descriptor(id, &descriptor)?;
        let format = output_target_wgpu_format(&descriptor).ok_or_else(|| {
            GraphicsError::Asset(format!(
                "output target texture {id} has unsupported render target format {}",
                descriptor.format
            ))
        })?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(OUTPUT_TARGET_TEXTURE_LABEL),
            size: wgpu::Extent3d {
                width: descriptor.width,
                height: descriptor.height,
                depth_or_array_layers: 1,
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
) -> Result<(), GraphicsError> {
    if descriptor.width == 0 || descriptor.height == 0 {
        return Err(GraphicsError::Asset(format!(
            "output target texture {id} must have nonzero extent"
        )));
    }
    if descriptor.dimension != RenderImageDimension::D2
        || descriptor.depth_or_array_layers != 1
        || descriptor.array_layer_count != 1
        || descriptor.mip_count != 1
    {
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
    Ok(())
}

fn output_target_texture_usages(
    descriptor: &RenderImageDescriptor,
    format: wgpu::TextureFormat,
) -> wgpu::TextureUsages {
    let mut usages = wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST;
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
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderImageColorSpace, RenderImageFallbackKind, RenderSamplerDescriptor,
    };

    #[test]
    fn output_target_texture_usages_prepare_render_target_only_without_sampled_binding() {
        let descriptor = texture_descriptor(vec![RenderImageUsage::RenderTarget]);

        let usages = output_target_texture_usages(&descriptor, wgpu::TextureFormat::Rgba8UnormSrgb);

        assert!(usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT));
        assert!(usages.contains(wgpu::TextureUsages::COPY_SRC));
        assert!(usages.contains(wgpu::TextureUsages::COPY_DST));
        assert!(!usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
    }

    #[test]
    fn output_target_texture_usages_preserve_copy_and_sampled_authoring_flags() {
        let descriptor = texture_descriptor(vec![
            RenderImageUsage::RenderTarget,
            RenderImageUsage::Sampled,
            RenderImageUsage::CopySrc,
        ]);

        let usages = output_target_texture_usages(&descriptor, wgpu::TextureFormat::Rgba8Unorm);

        assert!(usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT));
        assert!(usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
        assert!(usages.contains(wgpu::TextureUsages::COPY_SRC));
        assert!(usages.contains(wgpu::TextureUsages::COPY_DST));
    }

    #[test]
    fn output_target_wgpu_format_uses_descriptor_label() {
        assert_eq!(
            output_target_wgpu_format(&texture_descriptor_with_format(RGBA8_UNORM_FORMAT)),
            Some(wgpu::TextureFormat::Rgba8Unorm)
        );
        assert_eq!(
            output_target_wgpu_format(&texture_descriptor_with_format(RGBA8_UNORM_SRGB_FORMAT)),
            Some(wgpu::TextureFormat::Rgba8UnormSrgb)
        );
        assert_eq!(
            output_target_wgpu_format(&texture_descriptor_with_format("dds/dxt1")),
            None
        );
    }

    #[test]
    fn validate_output_target_descriptor_rejects_sampled_only_target() {
        let descriptor = texture_descriptor(vec![RenderImageUsage::Sampled]);

        let error = validate_output_target_descriptor(
            ResourceId::from_stable_label("tests/target"),
            &descriptor,
        )
        .unwrap_err();

        assert!(
            matches!(error, GraphicsError::Asset(message) if message.contains("render_target usage"))
        );
    }

    fn texture_descriptor(usage: Vec<RenderImageUsage>) -> RenderImageDescriptor {
        RenderImageDescriptor {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
            dimension: RenderImageDimension::D2,
            format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
            color_space: RenderImageColorSpace::Srgb,
            metadata: TextureMetadata::default(),
            sampler: RenderSamplerDescriptor::default(),
            usage,
            asset_usage: Vec::new(),
            mip_count: 1,
            array_layer_count: 1,
            fallback: RenderImageFallbackKind::MissingImage,
        }
    }

    fn texture_descriptor_with_format(format: &str) -> RenderImageDescriptor {
        RenderImageDescriptor {
            format: format.to_string(),
            ..texture_descriptor(vec![RenderImageUsage::RenderTarget])
        }
    }
}
