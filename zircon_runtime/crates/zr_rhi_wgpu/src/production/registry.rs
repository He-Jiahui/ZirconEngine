//! Generation-local WGPU resource tables.
//!
//! The root owns identity and native object storage. Resource, binding, and
//! pipeline behavior live with their respective owners so this module remains
//! a stable internal facade rather than a growing implementation sink.

use std::collections::{HashMap, HashSet};

use zr_rhi::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutHandle, BufferDesc,
    BufferHandle, DeviceGeneration, DeviceId, GpuMemoryBudget, PipelineDesc, PipelineHandle,
    PipelineLayoutDesc, PipelineLayoutHandle, RenderResourceHandleAllocator, SamplerDesc,
    SamplerHandle, ShaderModuleDesc, ShaderModuleHandle, SubmissionTicket, TextureDesc,
    TextureHandle, TextureViewDesc, TextureViewHandle,
};

mod bindings;
mod pipelines;
mod resources;
mod retirement;
mod usage;

#[cfg(test)]
pub(super) use resources::validate_wgpu_buffer_usage;

struct WgpuBufferResource {
    desc: BufferDesc,
    native: wgpu::Buffer,
    last_uses: Vec<SubmissionTicket>,
}

struct WgpuTextureResource {
    desc: TextureDesc,
    native: wgpu::Texture,
    last_uses: Vec<SubmissionTicket>,
}

struct WgpuTextureViewResource {
    desc: TextureViewDesc,
    native: wgpu::TextureView,
    last_uses: Vec<SubmissionTicket>,
}

struct WgpuSamplerResource {
    desc: SamplerDesc,
    native: wgpu::Sampler,
    last_uses: Vec<SubmissionTicket>,
}

struct WgpuBindGroupLayoutResource {
    desc: BindGroupLayoutDesc,
    native: wgpu::BindGroupLayout,
    last_uses: Vec<SubmissionTicket>,
}

struct WgpuBindGroupResource {
    desc: BindGroupDesc,
    native: wgpu::BindGroup,
    last_uses: Vec<SubmissionTicket>,
}

struct WgpuShaderModuleResource {
    desc: ShaderModuleDesc,
    native: wgpu::ShaderModule,
    last_uses: Vec<SubmissionTicket>,
}

struct WgpuPipelineLayoutResource {
    desc: PipelineLayoutDesc,
    native: wgpu::PipelineLayout,
    last_uses: Vec<SubmissionTicket>,
}

enum WgpuPipelineResource {
    Raster {
        desc: PipelineDesc,
        native: wgpu::RenderPipeline,
        last_uses: Vec<SubmissionTicket>,
    },
    Compute {
        desc: PipelineDesc,
        native: wgpu::ComputePipeline,
        last_uses: Vec<SubmissionTicket>,
    },
}

impl WgpuPipelineResource {
    fn desc(&self) -> &PipelineDesc {
        match self {
            Self::Raster { desc, .. } | Self::Compute { desc, .. } => desc,
        }
    }

    fn last_uses_mut(&mut self) -> &mut Vec<SubmissionTicket> {
        match self {
            Self::Raster { last_uses, .. } | Self::Compute { last_uses, .. } => last_uses,
        }
    }
}

enum WgpuRetiredResource {
    Buffer(WgpuBufferResource),
    Texture(WgpuTextureResource),
    TextureView(WgpuTextureViewResource),
    Sampler(WgpuSamplerResource),
    BindGroupLayout(WgpuBindGroupLayoutResource),
    BindGroup(WgpuBindGroupResource),
    ShaderModule(WgpuShaderModuleResource),
    PipelineLayout(WgpuPipelineLayoutResource),
    Pipeline(WgpuPipelineResource),
}

struct WgpuRetirement {
    after: Vec<SubmissionTicket>,
    resource: WgpuRetiredResource,
}

/// Native resource table for exactly one WGPU device generation.
///
/// All native lookup passes through the same allocator that created the
/// neutral handle, so a handle from another device, generation, or released
/// slot cannot reach a WGPU object.
pub(crate) struct WgpuResourceRegistry {
    handles: RenderResourceHandleAllocator,
    memory_budget: GpuMemoryBudget,
    buffers: HashMap<BufferHandle, WgpuBufferResource>,
    textures: HashMap<TextureHandle, WgpuTextureResource>,
    texture_views: HashMap<TextureViewHandle, WgpuTextureViewResource>,
    texture_view_counts: HashMap<TextureHandle, u32>,
    surface_owned_textures: HashSet<TextureHandle>,
    surface_owned_texture_views: HashSet<TextureViewHandle>,
    surface_frame_submissions: HashMap<TextureHandle, HashSet<SubmissionTicket>>,
    samplers: HashMap<SamplerHandle, WgpuSamplerResource>,
    bind_group_layouts: HashMap<BindGroupLayoutHandle, WgpuBindGroupLayoutResource>,
    bind_groups: HashMap<BindGroupHandle, WgpuBindGroupResource>,
    shader_modules: HashMap<ShaderModuleHandle, WgpuShaderModuleResource>,
    pipeline_layouts: HashMap<PipelineLayoutHandle, WgpuPipelineLayoutResource>,
    pipelines: HashMap<PipelineHandle, WgpuPipelineResource>,
    retired: Vec<WgpuRetirement>,
}

impl WgpuResourceRegistry {
    pub(crate) fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        memory_budget: GpuMemoryBudget,
    ) -> Self {
        Self {
            handles: RenderResourceHandleAllocator::new(device_id, generation),
            memory_budget,
            buffers: HashMap::new(),
            textures: HashMap::new(),
            texture_views: HashMap::new(),
            texture_view_counts: HashMap::new(),
            surface_owned_textures: HashSet::new(),
            surface_owned_texture_views: HashSet::new(),
            surface_frame_submissions: HashMap::new(),
            samplers: HashMap::new(),
            bind_group_layouts: HashMap::new(),
            bind_groups: HashMap::new(),
            shader_modules: HashMap::new(),
            pipeline_layouts: HashMap::new(),
            pipelines: HashMap::new(),
            retired: Vec::new(),
        }
    }
}
