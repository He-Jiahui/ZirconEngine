use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

use crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistration;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassGpuRecordingContext, RenderPassGpuResourceFactory,
};
use crate::render_graph::RenderGraphComputeWorkload;

mod executor;

pub(crate) const PLANAR_FILTER_PIPELINE_LABEL: &str = "zircon-planar-reflection-filter";
pub(crate) const PLANAR_FILTER_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
pub const PLANAR_FILTER_EXECUTOR_ID: &str = "planar.filter";
pub const PLANAR_REFLECTION_TEXTURE_RESOURCE: &str = "planar.reflection_texture";

const PLANAR_FILTER_SHADER: &str = include_str!("shaders/filter.wgsl");

pub(crate) fn registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![RenderPassExecutorRegistration::new_executor(
        PLANAR_FILTER_EXECUTOR_ID,
        Arc::new(executor::PlanarReflectionFilterExecutor::default()),
    )]
}

pub fn planar_reflection_filter_compute_workload() -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::per_pixel(
        PLANAR_FILTER_PIPELINE_LABEL,
        PLANAR_FILTER_WORKGROUP_SIZE,
        PLANAR_REFLECTION_TEXTURE_RESOURCE,
        [
            PLANAR_FILTER_WORKGROUP_SIZE[0],
            PLANAR_FILTER_WORKGROUP_SIZE[1],
        ],
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlanarFilterReport {
    pub mip_count: u32,
    pub dispatches: Vec<[u32; 2]>,
    pub uploaded_bytes: u64,
}

pub(crate) struct PlanarReflectionFilterPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl PlanarReflectionFilterPipeline {
    pub(crate) fn new<F: RenderPassGpuResourceFactory + ?Sized>(factory: &F) -> Self {
        let bind_group_layout =
            factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-planar-reflection-filter-bind-group-layout"),
                entries: &[
                    sampled_texture_layout_entry(0),
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba16Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-planar-reflection-filter-shader"),
            source: wgpu::ShaderSource::Wgsl(PLANAR_FILTER_SHADER.into()),
        });
        let layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-planar-reflection-filter-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(PLANAR_FILTER_PIPELINE_LABEL),
            layout: Some(&layout),
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
        source_view: &wgpu::TextureView,
        output_texture: &wgpu::Texture,
        base_extent: wgpu::Extent3d,
        mip_count: u32,
    ) -> Result<PlanarFilterReport, String> {
        validate_filter_request(base_extent, mip_count)?;
        let mut dispatches = Vec::with_capacity(mip_count as usize);
        let mut previous_output_view = None;

        for mip_level in 0..mip_count {
            let output_width = (base_extent.width >> mip_level).max(1);
            let output_height = (base_extent.height >> mip_level).max(1);
            let input_width = if mip_level == 0 {
                base_extent.width
            } else {
                (base_extent.width >> (mip_level - 1)).max(1)
            };
            let input_height = if mip_level == 0 {
                base_extent.height
            } else {
                (base_extent.height >> (mip_level - 1)).max(1)
            };
            let params = GpuPlanarFilterParams {
                input_dimensions: [input_width, input_height],
                output_dimensions: [output_width, output_height],
                kernel: [mip_level.min(2), mip_level, 0, 0],
            };
            let params_buffer =
                context
                    .resource_factory()
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("zircon-planar-reflection-filter-params"),
                        contents: bytemuck::bytes_of(&params),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
            let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("zircon-planar-reflection-filter-output-mip"),
                format: Some(wgpu::TextureFormat::Rgba16Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                ),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip_level,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
            });
            let input_view = previous_output_view.as_ref().unwrap_or(source_view);
            let bind_group =
                context
                    .resource_factory()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("zircon-planar-reflection-filter-bind-group"),
                        layout: &self.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(input_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(&output_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: params_buffer.as_entire_binding(),
                            },
                        ],
                    });
            let dispatch = [
                output_width.div_ceil(PLANAR_FILTER_WORKGROUP_SIZE[0]),
                output_height.div_ceil(PLANAR_FILTER_WORKGROUP_SIZE[1]),
            ];
            let mut pass =
                context
                    .command_encoder()
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("PlanarReflectionFilterPass"),
                        timestamp_writes: None,
                    });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(dispatch[0], dispatch[1], 1);
            drop(pass);
            dispatches.push(dispatch);
            previous_output_view = Some(output_view);
        }

        Ok(PlanarFilterReport {
            mip_count,
            dispatches,
            uploaded_bytes: u64::from(mip_count)
                .saturating_mul(std::mem::size_of::<GpuPlanarFilterParams>() as u64),
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuPlanarFilterParams {
    input_dimensions: [u32; 2],
    output_dimensions: [u32; 2],
    kernel: [u32; 4],
}

fn validate_filter_request(extent: wgpu::Extent3d, mip_count: u32) -> Result<(), String> {
    if extent.width == 0 || extent.height == 0 || extent.depth_or_array_layers != 1 {
        return Err("planar filter requires a non-empty single-layer 2D extent".to_string());
    }
    let maximum_mips = u32::BITS - extent.width.max(extent.height).leading_zeros();
    if mip_count == 0 || mip_count > maximum_mips {
        return Err(format!(
            "planar filter mip count {mip_count} exceeds extent limit {maximum_mips}"
        ));
    }
    Ok(())
}

fn sampled_texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

#[cfg(test)]
mod tests;
