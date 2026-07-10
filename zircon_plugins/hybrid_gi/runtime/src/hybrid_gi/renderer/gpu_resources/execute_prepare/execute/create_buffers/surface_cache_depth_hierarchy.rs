const SURFACE_CACHE_DEPTH_HIERARCHY_WORKGROUP_SIZE: u32 = 8;

pub(super) const SURFACE_CACHE_DEPTH_HIERARCHY_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba8Unorm;

pub(super) fn surface_cache_depth_hierarchy_mip_level_count(extent: (u32, u32)) -> u32 {
    let full_chain = u32::BITS - extent.0.max(extent.1).max(1).leading_zeros();
    // A 64x64 surface-cache page reaches one texel at mip 6. Stopping there
    // keeps each aligned page independent instead of reducing across cards.
    full_chain.min(7).max(1)
}

pub(super) fn build_surface_cache_depth_hierarchy(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    base_extent: (u32, u32),
    mip_level_count: u32,
) {
    if mip_level_count <= 1 {
        return;
    }

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-surface-cache-depth-hierarchy-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: SURFACE_CACHE_DEPTH_HIERARCHY_FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-surface-cache-depth-hierarchy-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../../shaders/build_surface_cache_depth_hierarchy.wgsl").into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-surface-cache-depth-hierarchy-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-hybrid-gi-surface-cache-depth-hierarchy-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });

    for target_mip_level in 1..mip_level_count {
        let source_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-hybrid-gi-surface-cache-depth-hierarchy-source-view"),
            base_mip_level: target_mip_level - 1,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let target_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-hybrid-gi-surface-cache-depth-hierarchy-target-view"),
            base_mip_level: target_mip_level,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-hybrid-gi-surface-cache-depth-hierarchy-bind-group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&target_view),
                },
            ],
        });
        let target_width = (base_extent.0 >> target_mip_level).max(1);
        let target_height = (base_extent.1 >> target_mip_level).max(1);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("HybridGiSurfaceCacheDepthHierarchyPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(
            target_width.div_ceil(SURFACE_CACHE_DEPTH_HIERARCHY_WORKGROUP_SIZE),
            target_height.div_ceil(SURFACE_CACHE_DEPTH_HIERARCHY_WORKGROUP_SIZE),
            1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_cache_depth_hierarchy_stops_at_one_texel_per_aligned_page() {
        assert_eq!(surface_cache_depth_hierarchy_mip_level_count((512, 64)), 7);
        assert_eq!(surface_cache_depth_hierarchy_mip_level_count((512, 128)), 7);
        assert_eq!(surface_cache_depth_hierarchy_mip_level_count((1, 1)), 1);
    }
}
