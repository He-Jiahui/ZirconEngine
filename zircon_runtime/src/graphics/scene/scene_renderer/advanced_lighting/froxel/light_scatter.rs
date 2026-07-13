use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::framework::render::FroxelGridParams;
use crate::graphics::scene::scene_renderer::shadow::atlas::shadow_atlas_bind_group_layout_entries;

use super::{FroxelViewReconstruction, GpuFroxelTemporalReprojection, GpuFroxelViewParams};

pub(crate) const VOLUMETRIC_LIGHT_SCATTER_PIPELINE_LABEL: &str = "zircon-volumetric-light-scatter";
pub(crate) const VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE: [u32; 3] = [4, 4, 4];

const LIGHT_SCATTER_SHADER: &str = concat!(
    include_str!("shaders/zr_froxel_reconstruct.wgsl"),
    "\n",
    include_str!("light_scatter/shaders/types.wgsl"),
    "\n// include: zr_light_grid.wgsl\n",
    include_str!("../../lighting/shaders/zr_light_grid.wgsl"),
    "\n// include: zr_shadow.wgsl\n",
    include_str!("../../shadow/shaders/zr_shadow.wgsl"),
    "\n",
    include_str!("light_scatter/shaders/main.wgsl"),
);

pub(crate) struct FroxelLightScatterRequest<'a> {
    pub grid: FroxelGridParams,
    pub view: FroxelViewReconstruction,
    pub phase_g: f32,
    pub viewport_size: [u32; 2],
    pub media_view: &'a wgpu::TextureView,
    pub history_view: &'a wgpu::TextureView,
    pub temporal: GpuFroxelTemporalReprojection,
    pub light_buffer: &'a wgpu::Buffer,
    pub light_count: u32,
    pub light_grid_params_buffer: &'a wgpu::Buffer,
    pub light_zbins_buffer: &'a wgpu::Buffer,
    pub light_tile_masks_buffer: &'a wgpu::Buffer,
    pub shadow_atlas_view: &'a wgpu::TextureView,
    pub shadow_sampler: &'a wgpu::Sampler,
    pub shadow_slots_buffer: &'a wgpu::Buffer,
    pub shadow_globals_buffer: &'a wgpu::Buffer,
    pub output_view: &'a wgpu::TextureView,
}

pub(crate) struct FroxelLightScatterPipeline {
    resources_layout: wgpu::BindGroupLayout,
    lighting_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl FroxelLightScatterPipeline {
    pub(crate) const UPLOADED_BYTES_PER_DISPATCH: u64 =
        std::mem::size_of::<GpuLightScatterParams>() as u64;

    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let resources_layout = create_resources_layout(device);
        let lighting_layout = create_lighting_layout(device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-volumetric-light-scatter-shader"),
            source: wgpu::ShaderSource::Wgsl(LIGHT_SCATTER_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-volumetric-light-scatter-pipeline-layout"),
            bind_group_layouts: &[Some(&resources_layout), Some(&lighting_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(VOLUMETRIC_LIGHT_SCATTER_PIPELINE_LABEL),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            resources_layout,
            lighting_layout,
            pipeline,
        }
    }

    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        request: FroxelLightScatterRequest<'_>,
    ) -> Result<[u32; 3], String> {
        let params = GpuLightScatterParams::from_request(&request)?;
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-volumetric-light-scatter-params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let resources = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-volumetric-light-scatter-resources"),
            layout: &self.resources_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(request.media_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: request.light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(request.output_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(request.history_view),
                },
            ],
        });
        let lighting = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-volumetric-light-scatter-lighting"),
            layout: &self.lighting_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(request.shadow_atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(request.shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: request.shadow_slots_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: request.shadow_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 20,
                    resource: request.light_grid_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 21,
                    resource: request.light_zbins_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 22,
                    resource: request.light_tile_masks_buffer.as_entire_binding(),
                },
            ],
        });
        let dispatch = dispatch_size(params.grid_and_light_count[..3].try_into().unwrap());
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("VolumetricLightScatterPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &resources, &[]);
        pass.set_bind_group(1, &lighting, &[]);
        pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
        Ok(dispatch)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuLightScatterParams {
    grid_and_light_count: [u32; 4],
    viewport_size: [u32; 4],
    phase_g: [f32; 4],
    view: GpuFroxelViewParams,
    temporal: GpuFroxelTemporalReprojection,
}

impl GpuLightScatterParams {
    fn from_request(request: &FroxelLightScatterRequest<'_>) -> Result<Self, String> {
        let grid = request.grid.sanitized();
        Ok(Self {
            grid_and_light_count: [
                grid.dimensions[0],
                grid.dimensions[1],
                grid.dimensions[2],
                request.light_count,
            ],
            viewport_size: [
                request.viewport_size[0].max(1),
                request.viewport_size[1].max(1),
                0,
                0,
            ],
            phase_g: [request.phase_g.clamp(-0.9, 0.9), 0.0, 0.0, 0.0],
            view: GpuFroxelViewParams::new(request.view, grid)?,
            temporal: request.temporal,
        })
    }
}

fn create_resources_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-volumetric-light-scatter-resources-layout"),
        entries: &[
            buffer_layout_entry(0, wgpu::BufferBindingType::Uniform),
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            },
            buffer_layout_entry(2, wgpu::BufferBindingType::Storage { read_only: true }),
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D3,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            },
        ],
    })
}

fn create_lighting_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let mut entries = shadow_atlas_bind_group_layout_entries(wgpu::ShaderStages::COMPUTE).to_vec();
    entries.extend([
        buffer_layout_entry(20, wgpu::BufferBindingType::Uniform),
        buffer_layout_entry(21, wgpu::BufferBindingType::Storage { read_only: true }),
        buffer_layout_entry(22, wgpu::BufferBindingType::Storage { read_only: true }),
    ]);
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-volumetric-light-scatter-lighting-layout"),
        entries: &entries,
    })
}

fn buffer_layout_entry(binding: u32, ty: wgpu::BufferBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn dispatch_size(dimensions: [u32; 3]) -> [u32; 3] {
    [
        dimensions[0].div_ceil(VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE[0]),
        dimensions[1].div_ceil(VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE[1]),
        dimensions[2].div_ceil(VOLUMETRIC_LIGHT_SCATTER_WORKGROUP_SIZE[2]),
    ]
}

#[cfg(test)]
mod tests;
