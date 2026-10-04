use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;

/// 构造与 deferred shader group 布局匹配的绑定 ABI；环境预览保留局部 provider 槽位，全量光照再扩展 shadow 等资源。
pub(in crate::graphics::scene::scene_renderer::deferred) fn create_lighting_bind_group_layout(
    device: &wgpu::Device,
    deferred_lighting_profile: SceneRendererDeferredLightingProfile,
) -> wgpu::BindGroupLayout {
    const ENVIRONMENT_ENTRY_COUNT: usize = 10;
    const FULL_LIGHTING_ENTRY_COUNT: usize = 29;
    let full_lighting = deferred_lighting_profile.uses_full_lighting_bind_group();
    let mut entries = Vec::with_capacity(if full_lighting {
        FULL_LIGHTING_ENTRY_COUNT
    } else {
        ENVIRONMENT_ENTRY_COUNT
    });
    entries.extend([
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 4,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Depth,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 5,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        },
    ]);
    entries.extend(
        crate::graphics::scene::scene_renderer::environment::reflection_probe_bind_group_layout_entries(),
    );
    if full_lighting {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
        });
        entries.extend(
            crate::graphics::scene::scene_renderer::shadow::atlas::shadow_atlas_bind_group_layout_entries(wgpu::ShaderStages::FRAGMENT),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::environment::lightmap_bind_group_layout_entries(
            ),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::advanced_lighting::froxel::volumetric_apply_bind_group_layout_entries(
                wgpu::ShaderStages::FRAGMENT,
            ),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::light_cookie_bind_group_layout_entries(),
        );
        entries.extend(
            crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::irradiance_volume_bind_group_layout_entries(),
        );
        entries.extend([
            wgpu::BindGroupLayoutEntry {
                binding: 20,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 21,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 22,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]);
    }
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-deferred-lighting-bind-group-layout"),
        entries: &entries,
    })
}

#[cfg(test)]
#[path = "tests/create.rs"]
mod tests;
