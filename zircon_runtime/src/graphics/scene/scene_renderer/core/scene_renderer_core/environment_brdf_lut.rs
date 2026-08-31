use crate::graphics::backend::SystemTextureGenerationLease;

pub(in crate::graphics::scene::scene_renderer::core) struct SceneEnvironmentBrdfLut {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl SceneEnvironmentBrdfLut {
    pub(in crate::graphics::scene::scene_renderer::core) fn from_system_textures(
        system_textures: &SystemTextureGenerationLease,
    ) -> Self {
        Self {
            texture: system_textures.brdf_lut_texture().clone(),
            view: system_textures.brdf_lut_view().clone(),
        }
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

    pub(in crate::graphics::scene::scene_renderer::core) fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn scene_brdf_binding_is_a_read_only_generation_lease_projection() {
        let source = include_str!("environment_brdf_lut.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or_default();

        assert!(production.contains("from_system_textures"));
        assert!(production.contains("system_textures.brdf_lut_texture().clone()"));
        assert!(production.contains("system_textures.brdf_lut_view().clone()"));
        assert!(!production.contains("create_texture"));
        assert!(!production.contains("write_texture"));
        assert!(!production.contains("wgpu::Queue"));
    }
}
