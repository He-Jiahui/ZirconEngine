use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::post_process::hzb_build_dispatch_groups;
use crate::graphics::visibility::HzbBuildPlan;
use wgpu::util::DeviceExt;

use super::super::super::params::hzb_params::HzbParams;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::core::framework::render::COMPUTE_SHADER_PARAMS_BINDING;
use crate::graphics::shader::{
    hzb_build_dispatch_plan, HZB_SCENE_DEPTH_RESOURCE, HZB_SOURCE_RESOURCE, HZB_TARGET_RESOURCE,
};

const HZB_MAX_MIP_COUNT: usize = u32::BITS as usize;

pub(super) struct HzbBuildMipResources<'a> {
    pub bind_group_layout: &'a wgpu::BindGroupLayout,
    pub pipeline: &'a wgpu::ComputePipeline,
    pub params_buffer: &'a wgpu::Buffer,
    pub fallback_source_view: &'a wgpu::TextureView,
}

pub(super) fn create_hzb_params_upload_buffer(
    device: &wgpu::Device,
    plan: HzbBuildPlan,
) -> wgpu::Buffer {
    let mip_count = plan.mip_count as usize;
    assert!(
        mip_count <= HZB_MAX_MIP_COUNT,
        "HZB mip count exceeds the u32 texture extent domain"
    );
    let mut params = [HzbParams {
        target_size: [1, 1],
        target_mip_level: 0,
        _pad0: 0,
    }; HZB_MAX_MIP_COUNT];
    for target_mip_level in 0..plan.mip_count {
        let target_size = plan.mip_size(target_mip_level);
        params[target_mip_level as usize] = HzbParams {
            target_size: [target_size.x.max(1), target_size.y.max(1)],
            target_mip_level,
            _pad0: 0,
        };
    }
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-hzb-build-params-upload"),
        contents: bytemuck::cast_slice(&params[..mip_count]),
        usage: wgpu::BufferUsages::COPY_SRC,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_hzb_build_mip_with_resources(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    scene_depth_view: &wgpu::TextureView,
    source_hzb_view: Option<&wgpu::TextureView>,
    target_hzb_view: &wgpu::TextureView,
    target_size: UVec2,
    target_mip_level: u32,
    params_upload_buffer: &wgpu::Buffer,
    resources: HzbBuildMipResources<'_>,
) {
    let params_size = std::mem::size_of::<HzbParams>() as u64;
    encoder.copy_buffer_to_buffer(
        params_upload_buffer,
        u64::from(target_mip_level) * params_size,
        resources.params_buffer,
        0,
        params_size,
    );

    let dispatch_plan = hzb_build_dispatch_plan();
    let scene_depth_binding = dispatch_plan
        .resource_binding(HZB_SCENE_DEPTH_RESOURCE)
        .expect("HZB scene-depth binding must exist")
        .abi
        .binding;
    let source_binding = dispatch_plan
        .resource_binding(HZB_SOURCE_RESOURCE)
        .expect("HZB source binding must exist")
        .abi
        .binding;
    let target_binding = dispatch_plan
        .resource_binding(HZB_TARGET_RESOURCE)
        .expect("HZB target binding must exist")
        .abi
        .binding;
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hzb-build-bind-group"),
        layout: resources.bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: COMPUTE_SHADER_PARAMS_BINDING.binding,
                resource: resources.params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: scene_depth_binding,
                resource: wgpu::BindingResource::TextureView(scene_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: source_binding,
                resource: wgpu::BindingResource::TextureView(
                    source_hzb_view.unwrap_or(resources.fallback_source_view),
                ),
            },
            wgpu::BindGroupEntry {
                binding: target_binding,
                resource: wgpu::BindingResource::TextureView(target_hzb_view),
            },
        ],
    });

    let dispatch_groups = hzb_build_dispatch_groups(target_size);
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("HzbBuildPass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(resources.pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(dispatch_groups[0], dispatch_groups[1], dispatch_groups[2]);
}

impl ScenePostProcessResources {
    pub(crate) fn create_hzb_params_upload_buffer(
        &self,
        device: &wgpu::Device,
        plan: HzbBuildPlan,
    ) -> wgpu::Buffer {
        create_hzb_params_upload_buffer(device, plan)
    }

    pub(crate) fn execute_hzb_build_mip(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scene_depth_view: &wgpu::TextureView,
        source_hzb_view: Option<&wgpu::TextureView>,
        target_hzb_view: &wgpu::TextureView,
        target_size: UVec2,
        target_mip_level: u32,
        scene_depth_sample_count: u32,
        params_upload_buffer: &wgpu::Buffer,
    ) {
        let (bind_group_layout, pipeline) = if scene_depth_sample_count > 1 {
            (&self.hzb_msaa_bind_group_layout, &self.hzb_msaa_pipeline)
        } else {
            (&self.hzb_bind_group_layout, &self.hzb_pipeline)
        };
        execute_hzb_build_mip_with_resources(
            device,
            encoder,
            scene_depth_view,
            source_hzb_view,
            target_hzb_view,
            target_size,
            target_mip_level,
            params_upload_buffer,
            HzbBuildMipResources {
                bind_group_layout,
                pipeline,
                params_buffer: &self.hzb_params_buffer,
                fallback_source_view: &self.hzb_source_texture_view,
            },
        );
    }
}
