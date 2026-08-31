use std::num::NonZeroU64;

use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderPassGpuRecordingContext, RenderPassGpuResourceFactory,
};
use crate::graphics::types::ViewportRenderRegion;

use super::{SSS_RECOMBINE_PIPELINE_LABEL, SSS_SCATTER_PIPELINE_LABEL, SSS_SETUP_PIPELINE_LABEL};

pub(super) const SETUP_SHADER: &str = include_str!("shaders/setup.wgsl");
pub(super) const SCATTER_SHADER: &str = include_str!("shaders/scatter.wgsl");
pub(super) const RECOMBINE_SHADER: &str = include_str!("shaders/recombine.wgsl");

pub(super) struct SubsurfacePipelines {
    setup_layout: wgpu::BindGroupLayout,
    setup: wgpu::ComputePipeline,
    scatter_layout: wgpu::BindGroupLayout,
    scatter: wgpu::ComputePipeline,
    recombine_layout: wgpu::BindGroupLayout,
    recombine: wgpu::RenderPipeline,
}

impl SubsurfacePipelines {
    pub(super) fn new<F: RenderPassGpuResourceFactory + ?Sized>(
        factory: &F,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let setup_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sss.setup.bind-group-layout"),
            entries: &[
                sampled_texture_entry(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    1,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                storage_buffer_entry(2, false, wgpu::ShaderStages::COMPUTE),
                storage_buffer_entry(3, false, wgpu::ShaderStages::COMPUTE),
                uniform_entry(4, wgpu::ShaderStages::COMPUTE),
            ],
        });
        let setup = compute_pipeline(
            factory,
            SSS_SETUP_PIPELINE_LABEL,
            &setup_layout,
            SETUP_SHADER,
        );

        let scatter_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sss.scatter.bind-group-layout"),
            entries: &[
                sampled_texture_entry(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    1,
                    wgpu::TextureSampleType::Depth,
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    2,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    3,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                storage_buffer_entry(4, true, wgpu::ShaderStages::COMPUTE),
                uniform_entry(5, wgpu::ShaderStages::COMPUTE),
                uniform_entry(6, wgpu::ShaderStages::COMPUTE),
                storage_texture_entry(7),
            ],
        });
        let scatter = compute_pipeline(
            factory,
            SSS_SCATTER_PIPELINE_LABEL,
            &scatter_layout,
            SCATTER_SHADER,
        );

        let recombine_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sss.recombine.bind-group-layout"),
            entries: &[
                sampled_texture_entry(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::FRAGMENT,
                ),
                sampled_texture_entry(
                    1,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::FRAGMENT,
                ),
                sampled_texture_entry(
                    2,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::FRAGMENT,
                ),
            ],
        });
        let recombine =
            render_pipeline(factory, target_format, &recombine_layout, RECOMBINE_SHADER);

        Self {
            setup_layout,
            setup,
            scatter_layout,
            scatter,
            recombine_layout,
            recombine,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_setup<C: RenderPassGpuRecordingContext>(
        &self,
        context: &mut C,
        material: &wgpu::TextureView,
        normal: &wgpu::TextureView,
        tile_list: wgpu::BufferBinding<'_>,
        indirect_args: wgpu::BufferBinding<'_>,
        params: wgpu::BufferBinding<'_>,
        dispatch: [u32; 3],
    ) {
        context.command_encoder().clear_buffer(
            indirect_args.buffer,
            indirect_args.offset,
            indirect_args.size.map(NonZeroU64::get),
        );
        let bind_group = context
            .resource_factory()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sss.setup.bind-group"),
                layout: &self.setup_layout,
                entries: &[
                    texture_entry(0, material),
                    texture_entry(1, normal),
                    buffer_entry(2, tile_list),
                    buffer_entry(3, indirect_args),
                    buffer_entry(4, params),
                ],
            });
        let mut pass = context
            .command_encoder()
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(SSS_SETUP_PIPELINE_LABEL),
                timestamp_writes: None,
            });
        pass.set_pipeline(&self.setup);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_scatter<C: RenderPassGpuRecordingContext>(
        &self,
        context: &mut C,
        diffuse: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        material: &wgpu::TextureView,
        normal: &wgpu::TextureView,
        tile_list: wgpu::BufferBinding<'_>,
        indirect_args: wgpu::BufferBinding<'_>,
        profiles: wgpu::BufferBinding<'_>,
        params: wgpu::BufferBinding<'_>,
        scattered: &wgpu::TextureView,
    ) {
        let bind_group = context
            .resource_factory()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sss.scatter.bind-group"),
                layout: &self.scatter_layout,
                entries: &[
                    texture_entry(0, diffuse),
                    texture_entry(1, depth),
                    texture_entry(2, material),
                    texture_entry(3, normal),
                    buffer_entry(4, tile_list),
                    buffer_entry(5, profiles),
                    buffer_entry(6, params),
                    texture_entry(7, scattered),
                ],
            });
        let mut pass = context
            .command_encoder()
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(SSS_SCATTER_PIPELINE_LABEL),
                timestamp_writes: None,
            });
        pass.set_pipeline(&self.scatter);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups_indirect(indirect_args.buffer, indirect_args.offset);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_recombine<C: RenderPassGpuRecordingContext>(
        &self,
        context: &mut C,
        render_region: ViewportRenderRegion,
        scattered: &wgpu::TextureView,
        specular: &wgpu::TextureView,
        material: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let bind_group = context
            .resource_factory()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sss.recombine.bind-group"),
                layout: &self.recombine_layout,
                entries: &[
                    texture_entry(0, scattered),
                    texture_entry(1, specular),
                    texture_entry(2, material),
                ],
            });
        let mut pass = context
            .command_encoder()
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(&self.recombine);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn compute_pipeline<F: RenderPassGpuResourceFactory + ?Sized>(
    factory: &F,
    label: &'static str,
    bind_group_layout: &wgpu::BindGroupLayout,
    source: &'static str,
) -> wgpu::ComputePipeline {
    let layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn render_pipeline<F: RenderPassGpuResourceFactory + ?Sized>(
    factory: &F,
    target_format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    source: &'static str,
) -> wgpu::RenderPipeline {
    let layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    factory.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn sampled_texture_entry(
    binding: u32,
    sample_type: wgpu::TextureSampleType,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn storage_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    }
}

fn storage_buffer_entry(
    binding: u32,
    read_only: bool,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

fn buffer_entry(binding: u32, buffer: wgpu::BufferBinding<'_>) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(buffer),
    }
}
