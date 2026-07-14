pub(crate) const TRANSMISSION_SCENE_COLOR_BINDING: u32 = 31;
pub(crate) const TRANSMISSION_SCENE_COLOR_SAMPLER_BINDING: u32 = 32;
const TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL: [u8; 4] = [0, 0, 0, 0];

pub(crate) struct TransmissionSceneColorFallbackResources {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl TransmissionSceneColorFallbackResources {
    pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-transmission-scene-color-fallback"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-transmission-scene-color-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        Self {
            _texture: texture,
            view,
            sampler,
        }
    }

    pub(crate) fn bind_group_entries<'a>(
        &'a self,
        scene_color_view: Option<&'a wgpu::TextureView>,
    ) -> [wgpu::BindGroupEntry<'a>; 2] {
        [
            wgpu::BindGroupEntry {
                binding: TRANSMISSION_SCENE_COLOR_BINDING,
                resource: wgpu::BindingResource::TextureView(
                    scene_color_view.unwrap_or(&self.view),
                ),
            },
            wgpu::BindGroupEntry {
                binding: TRANSMISSION_SCENE_COLOR_SAMPLER_BINDING,
                resource: wgpu::BindingResource::Sampler(&self.sampler),
            },
        ]
    }
}

pub(crate) fn transmission_scene_color_bind_group_layout_entries() -> [wgpu::BindGroupLayoutEntry; 2]
{
    [
        wgpu::BindGroupLayoutEntry {
            binding: TRANSMISSION_SCENE_COLOR_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: TRANSMISSION_SCENE_COLOR_SAMPLER_BINDING,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL;

    #[test]
    fn render_transmission_zero_step_fallback_marks_scene_copy_unavailable() {
        assert_eq!(TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL, [0, 0, 0, 0]);
    }
}
