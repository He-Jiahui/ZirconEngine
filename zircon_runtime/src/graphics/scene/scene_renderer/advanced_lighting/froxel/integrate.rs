use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::FroxelGridParams;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassGpuRecordingContext, RenderPassGpuResourceFactory,
};

use super::{FroxelViewReconstruction, GpuFroxelViewParams};

pub(crate) const VOLUMETRIC_INTEGRATE_PIPELINE_LABEL: &str = "zircon-volumetric-integrate";
pub(crate) const VOLUMETRIC_INTEGRATE_WORKGROUP_SIZE: [u32; 2] = [8, 8];

const INTEGRATE_SHADER: &str = concat!(
    include_str!("shaders/zr_froxel_reconstruct.wgsl"),
    "\n",
    include_str!("integrate/shaders/integrate.wgsl"),
);

pub(crate) struct FroxelIntegrateRequest<'a> {
    pub grid: FroxelGridParams,
    pub view: FroxelViewReconstruction,
    pub scattering_view: &'a wgpu::TextureView,
    pub output_view: &'a wgpu::TextureView,
}

pub(crate) struct FroxelIntegratePipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl FroxelIntegratePipeline {
    pub(crate) const UPLOADED_BYTES_PER_DISPATCH: u64 =
        std::mem::size_of::<GpuIntegrateParams>() as u64;

    pub(crate) fn new<F: RenderPassGpuResourceFactory + ?Sized>(factory: &F) -> Self {
        let bind_group_layout =
            factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-volumetric-integrate-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    sampled_texture_layout_entry(1, wgpu::TextureViewDimension::D3),
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D3,
                        },
                        count: None,
                    },
                ],
            });
        let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-volumetric-integrate-shader"),
            source: wgpu::ShaderSource::Wgsl(INTEGRATE_SHADER.into()),
        });
        let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-volumetric-integrate-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(VOLUMETRIC_INTEGRATE_PIPELINE_LABEL),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            bind_group_layout,
            pipeline,
        }
    }

    pub(crate) fn encode<C: RenderPassGpuRecordingContext>(
        &self,
        context: &mut C,
        request: FroxelIntegrateRequest<'_>,
    ) -> Result<[u32; 2], String> {
        let grid = request.grid.sanitized();
        let params = GpuIntegrateParams {
            grid_dimensions: [
                grid.dimensions[0],
                grid.dimensions[1],
                grid.dimensions[2],
                0,
            ],
            view: GpuFroxelViewParams::new(request.view, grid)?,
        };
        let params_buffer =
            context
                .resource_factory()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("zircon-volumetric-integrate-params"),
                    contents: bytemuck::bytes_of(&params),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        let bind_group = context
            .resource_factory()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("zircon-volumetric-integrate-bind-group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(request.scattering_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(request.output_view),
                    },
                ],
            });
        let dispatch = [
            grid.dimensions[0].div_ceil(VOLUMETRIC_INTEGRATE_WORKGROUP_SIZE[0]),
            grid.dimensions[1].div_ceil(VOLUMETRIC_INTEGRATE_WORKGROUP_SIZE[1]),
        ];
        let mut pass = context
            .command_encoder()
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("VolumetricIntegratePass"),
                timestamp_writes: None,
            });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch[0], dispatch[1], 1);
        Ok(dispatch)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuIntegrateParams {
    grid_dimensions: [u32; 4],
    view: GpuFroxelViewParams,
}

fn sampled_texture_layout_entry(
    binding: u32,
    view_dimension: wgpu::TextureViewDimension,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension,
            multisampled: false,
        },
        count: None,
    }
}

#[cfg(test)]
mod tests;
