const GLOBAL_SDF_BUILD_SHADER: &str = include_str!("../../shaders/global_sdf_build.wgsl");

pub(in crate::hybrid_gi::renderer) struct GlobalSdfGpuResources {
    pub(super) bind_group_layout: wgpu::BindGroupLayout,
    pub(super) pipeline: wgpu::ComputePipeline,
}

impl GlobalSdfGpuResources {
    pub(in crate::hybrid_gi::renderer) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-hybrid-gi-global-sdf-bind-group-layout"),
            entries: &[
                buffer_layout_entry(0, wgpu::BufferBindingType::Uniform),
                buffer_layout_entry(1, wgpu::BufferBindingType::Storage { read_only: true }),
                buffer_layout_entry(2, wgpu::BufferBindingType::Storage { read_only: true }),
                buffer_layout_entry(3, wgpu::BufferBindingType::Storage { read_only: true }),
                buffer_layout_entry(4, wgpu::BufferBindingType::Storage { read_only: true }),
                buffer_layout_entry(5, wgpu::BufferBindingType::Storage { read_only: true }),
                buffer_layout_entry(6, wgpu::BufferBindingType::Storage { read_only: false }),
                buffer_layout_entry(7, wgpu::BufferBindingType::Storage { read_only: false }),
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-hybrid-gi-global-sdf-build-shader"),
            source: wgpu::ShaderSource::Wgsl(GLOBAL_SDF_BUILD_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-hybrid-gi-global-sdf-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-hybrid-gi-global-sdf-build-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_build_global_sdf"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self {
            bind_group_layout,
            pipeline,
        }
    }
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
