use crate::core::framework::render::{
    ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT, ENVIRONMENT_BRDF_LUT_SIZE, EnvironmentBrdfLutTexel,
    build_environment_brdf_lut,
};

use super::half_float::push_f16_le_bytes;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneEnvironmentBrdfLut {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl SceneEnvironmentBrdfLut {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-scene-environment-brdf-lut"),
            size: wgpu::Extent3d {
                width: ENVIRONMENT_BRDF_LUT_SIZE,
                height: ENVIRONMENT_BRDF_LUT_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-scene-environment-brdf-lut-view"),
            format: Some(wgpu::TextureFormat::Rg16Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
        });

        let texels = build_environment_brdf_lut(
            ENVIRONMENT_BRDF_LUT_SIZE,
            ENVIRONMENT_BRDF_LUT_SAMPLE_COUNT,
        );
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rg16float_texels(&texels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ENVIRONMENT_BRDF_LUT_SIZE * 4),
                rows_per_image: Some(ENVIRONMENT_BRDF_LUT_SIZE),
            },
            wgpu::Extent3d {
                width: ENVIRONMENT_BRDF_LUT_SIZE,
                height: ENVIRONMENT_BRDF_LUT_SIZE,
                depth_or_array_layers: 1,
            },
        );

        Self { texture, view }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn texture_layout_entry(
        binding: u32,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn binding_resource(
        &self,
    ) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::TextureView(&self.view)
    }
}

fn rg16float_texels(texels: &[EnvironmentBrdfLutTexel]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(texels.len() * 4);
    for texel in texels {
        for channel in texel {
            push_f16_le_bytes(&mut bytes, *channel);
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rg16float_texels_encode_two_channels_per_texel() {
        let bytes = rg16float_texels(&[[1.0, 0.0], [0.5, 0.25]]);
        assert_eq!(bytes.len(), 8);
        assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), 0x3c00);
        assert_eq!(u16::from_le_bytes([bytes[2], bytes[3]]), 0x0000);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 0x3800);
        assert_eq!(u16::from_le_bytes([bytes[6], bytes[7]]), 0x3400);
    }
}
