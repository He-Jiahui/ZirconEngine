use crate::graphics::shader::{hzb_build_dispatch_plan, hzb_build_msaa_dispatch_plan};

pub(super) fn hzb_pipeline(
    device: &wgpu::Device,
    hzb_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let plan = hzb_build_dispatch_plan();
    create_hzb_pipeline(
        device,
        hzb_bind_group_layout,
        "zircon-hzb-build-shader",
        plan,
        include_str!("../../../shaders/hzb_build.wgsl"),
    )
}

pub(super) fn hzb_msaa_pipeline(
    device: &wgpu::Device,
    hzb_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let plan = hzb_build_msaa_dispatch_plan();
    create_hzb_pipeline(
        device,
        hzb_bind_group_layout,
        "zircon-hzb-build-msaa-shader",
        plan,
        include_str!("../../../shaders/hzb_build_msaa.wgsl"),
    )
}

fn create_hzb_pipeline(
    device: &wgpu::Device,
    hzb_bind_group_layout: &wgpu::BindGroupLayout,
    shader_label: &'static str,
    plan: &crate::core::framework::render::ComputeDispatchPlan,
    shader_source: &'static str,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(shader_label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });
    let pipeline_layout_label = format!("{}-layout", plan.pipeline_label);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&pipeline_layout_label),
        bind_group_layouts: &[Some(hzb_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(&plan.pipeline_label),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some(&plan.kernel.kernel),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    const HZB_BUILD_SHADER: &str = include_str!("../../../shaders/hzb_build.wgsl");
    const HZB_BUILD_MSAA_SHADER: &str = include_str!("../../../shaders/hzb_build_msaa.wgsl");

    #[test]
    fn hzb_shader_declares_reduce_entry_and_storage_target() {
        assert!(HZB_BUILD_SHADER.contains("@compute @workgroup_size(8, 8, 1)"));
        assert!(HZB_BUILD_SHADER.contains("fn cs_main"));
        assert!(HZB_BUILD_SHADER.contains("texture_storage_2d<rgba16float, write>"));
    }

    #[test]
    fn hzb_shader_preserves_furthest_and_closest_depth_per_mip() {
        for source in [HZB_BUILD_SHADER, HZB_BUILD_MSAA_SHADER] {
            assert!(source.contains("struct HzbDepthRange"));
            assert!(source.contains("furthest_depth"));
            assert!(source.contains("closest_depth"));
            assert!(source.contains("parent_range.y"));
            assert!(source.contains("depth_range.furthest"));
            assert!(source.contains("depth_range.closest"));
        }
        assert!(HZB_BUILD_MSAA_SHADER.contains("texture_depth_multisampled_2d"));
        assert!(HZB_BUILD_MSAA_SHADER.contains("textureNumSamples(scene_depth_tex)"));
    }
}
