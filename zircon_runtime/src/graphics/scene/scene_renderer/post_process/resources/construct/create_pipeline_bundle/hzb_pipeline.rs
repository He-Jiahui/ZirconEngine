use crate::graphics::shader::{hzb_build_dispatch_plan, hzb_build_msaa_dispatch_plan};

const ZR_REDUCE_INCLUDE: &str = include_str!("../../../../../../shader/includes/zr_reduce.wgsl");

/// 单采样深度路径依据统一 dispatch plan 构造 HZB 管线，与布局和 WGSL 入口保持一致。
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
    plan: &crate::graphics::shader::invocation::ComputeDispatchPlan,
    shader_source: &str,
) -> wgpu::ComputePipeline {
    let shader_source = [ZR_REDUCE_INCLUDE, shader_source].concat();
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
#[path = "tests/hzb_pipeline.rs"]
mod tests;
