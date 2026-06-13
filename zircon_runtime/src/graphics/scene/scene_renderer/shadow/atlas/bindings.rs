pub(crate) const SHADOW_ATLAS_BINDING: u32 = 8;
pub(crate) const SHADOW_ATLAS_SAMPLER_BINDING: u32 = 9;
pub(crate) const SHADOW_ATLAS_SLOT_BUFFER_BINDING: u32 = 10;
pub(crate) const SHADOW_GLOBALS_BINDING: u32 = 11;

pub(crate) fn shadow_atlas_bind_group_layout_entries(
    visibility: wgpu::ShaderStages,
) -> [wgpu::BindGroupLayoutEntry; 4] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_ATLAS_BINDING,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Depth,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_ATLAS_SAMPLER_BINDING,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_ATLAS_SLOT_BUFFER_BINDING,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: SHADOW_GLOBALS_BINDING,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_shadow_atlas_group1_bindings_avoid_legacy_shadow_and_light_grid_slots() {
        let legacy_forward = [0, 1, 2, 20, 21, 22];
        let legacy_deferred = [0, 1, 2, 3, 4, 5, 6, 7, 20, 21, 22];
        let shadow_atlas = [
            SHADOW_ATLAS_BINDING,
            SHADOW_ATLAS_SAMPLER_BINDING,
            SHADOW_ATLAS_SLOT_BUFFER_BINDING,
            SHADOW_GLOBALS_BINDING,
        ];

        for binding in shadow_atlas {
            assert!(!legacy_forward.contains(&binding));
            assert!(!legacy_deferred.contains(&binding));
        }
    }

    #[test]
    fn render_shadow_atlas_group1_layout_entries_match_plan_05_resource_types() {
        let entries = shadow_atlas_bind_group_layout_entries(wgpu::ShaderStages::FRAGMENT);

        assert_eq!(entries.map(|entry| entry.binding), [8, 9, 10, 11]);
        assert!(matches!(
            entries[0].ty,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Depth,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            }
        ));
        assert!(matches!(
            entries[1].ty,
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison)
        ));
        assert!(matches!(
            entries[2].ty,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            }
        ));
        assert!(matches!(
            entries[3].ty,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            }
        ));
    }
}
