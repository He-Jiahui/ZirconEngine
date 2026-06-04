use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::capabilities::{RenderBackendCaps, RenderDebugInstrumentationStatus, RenderQueueClass};
use super::descriptors::{
    BindGroupLayoutDesc, BufferDesc, BufferUsage, PipelineDesc, PipelineKind, PipelineLayoutDesc,
    SamplerDesc, ShaderModuleDesc, TextureDesc, TextureUsage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferHandle(u64);

impl BufferHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureHandle(u64);

impl TextureHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SamplerHandle(u64);

impl SamplerHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BindGroupLayoutHandle(u64);

impl BindGroupLayoutHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BindGroupHandle(u64);

impl BindGroupHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderModuleHandle(u64);

impl ShaderModuleHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineLayoutHandle(u64);

impl PipelineLayoutHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineHandle(u64);

impl PipelineHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RhiError {
    #[error("render queue `{0:?}` is not supported by this backend")]
    UnsupportedQueue(RenderQueueClass),
    #[error("buffer `{0}` does not exist")]
    UnknownBuffer(u64),
    #[error("texture `{0}` does not exist")]
    UnknownTexture(u64),
    #[error("sampler `{0}` does not exist")]
    UnknownSampler(u64),
    #[error("bind group layout `{0}` does not exist")]
    UnknownBindGroupLayout(u64),
    #[error("bind group `{0}` does not exist")]
    UnknownBindGroup(u64),
    #[error("shader module `{0}` does not exist")]
    UnknownShaderModule(u64),
    #[error("pipeline layout `{0}` does not exist")]
    UnknownPipelineLayout(u64),
    #[error("pipeline `{0}` does not exist")]
    UnknownPipeline(u64),
    #[error("render surface is unavailable: {0}")]
    SurfaceUnavailable(String),
    #[error("invalid surface descriptor `{label:?}`: {reason}")]
    InvalidSurfaceDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("fence `{0}` was not issued by this device")]
    UnknownFence(u64),
    #[error("invalid buffer descriptor `{label:?}`: {reason}")]
    InvalidBufferDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid texture descriptor `{label:?}`: {reason}")]
    InvalidTextureDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid sampler descriptor `{label:?}`: {reason}")]
    InvalidSamplerDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid bind group layout descriptor `{label:?}`: {reason}")]
    InvalidBindGroupLayoutDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid bind group descriptor `{label:?}`: {reason}")]
    InvalidBindGroupDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid bind group usage: {reason}")]
    InvalidBindGroupUsage { reason: String },
    #[error("invalid render pass: {reason}")]
    InvalidRenderPass { reason: String },
    #[error("invalid shader module descriptor `{label:?}`: {reason}")]
    InvalidShaderModuleDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid pipeline layout descriptor `{label:?}`: {reason}")]
    InvalidPipelineLayoutDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid pipeline descriptor `{label:?}`: {reason}")]
    InvalidPipelineDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("readback range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    ReadbackOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error("write range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    WriteOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error("buffer `{buffer}` usage {actual:?} does not include required usage {required:?}")]
    InvalidBufferUsage {
        buffer: u64,
        required: BufferUsage,
        actual: BufferUsage,
    },
    #[error("texture `{texture}` usage {actual:?} does not include required usage {required:?}")]
    InvalidTextureUsage {
        texture: u64,
        required: TextureUsage,
        actual: TextureUsage,
    },
    #[error("pipeline `{pipeline}` kind {actual:?} does not satisfy required kind {required:?}")]
    InvalidPipelineUsage {
        pipeline: u64,
        required: PipelineKind,
        actual: PipelineKind,
    },
    #[error("command `{command}` cannot be recorded for queue `{queue:?}`")]
    InvalidCommandQueue {
        queue: RenderQueueClass,
        command: String,
    },
    #[error("invalid compute dispatch: {reason}")]
    InvalidComputeDispatch { reason: String },
    #[error("invalid raster draw: {reason}")]
    InvalidRasterDraw { reason: String },
    #[error("invalid debug marker: {reason}")]
    InvalidDebugMarker { reason: String },
    #[error("buffer binding range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    BufferBindingOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error(
        "buffer copy range is outside source `{source_buffer}` or destination `{destination_buffer}`: source offset {source_offset}, destination offset {destination_offset}, size {size}"
    )]
    BufferCopyOutOfRange {
        source_buffer: u64,
        destination_buffer: u64,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    },
    #[error(
        "buffer-to-texture copy is outside source `{source_buffer}` or destination `{destination_texture}`: source offset {source_offset}, bytes per row {bytes_per_row}, mip {mip_level}, origin ({origin_x}, {origin_y}, {origin_z}), width {width}, height {height}"
    )]
    BufferToTextureCopyOutOfRange {
        source_buffer: u64,
        destination_texture: u64,
        source_offset: u64,
        bytes_per_row: u64,
        mip_level: u32,
        origin_x: u32,
        origin_y: u32,
        origin_z: u32,
        width: u32,
        height: u32,
    },
    #[error(
        "texture-to-buffer copy is outside source `{source_texture}` or destination `{destination_buffer}`: destination offset {destination_offset}, bytes per row {bytes_per_row}, mip {mip_level}, origin ({origin_x}, {origin_y}, {origin_z}), width {width}, height {height}"
    )]
    TextureToBufferCopyOutOfRange {
        source_texture: u64,
        destination_buffer: u64,
        destination_offset: u64,
        bytes_per_row: u64,
        mip_level: u32,
        origin_x: u32,
        origin_y: u32,
        origin_z: u32,
        width: u32,
        height: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

impl IndexFormat {
    pub const fn size_bytes(self) -> u64 {
        match self {
            Self::Uint16 => 2,
            Self::Uint32 => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindGroupEntryResource {
    Buffer(BufferHandle),
    Texture(TextureHandle),
    Sampler(SamplerHandle),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindGroupEntryDesc {
    pub binding: u32,
    pub resource: BindGroupEntryResource,
}

impl BindGroupEntryDesc {
    pub const fn new(binding: u32, resource: BindGroupEntryResource) -> Self {
        Self { binding, resource }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindGroupDesc {
    pub label: Option<String>,
    pub layout: BindGroupLayoutHandle,
    pub entries: Vec<BindGroupEntryDesc>,
}

impl BindGroupDesc {
    pub fn new(
        label: impl Into<String>,
        layout: BindGroupLayoutHandle,
        entries: Vec<BindGroupEntryDesc>,
    ) -> Self {
        Self {
            label: Some(label.into()),
            layout,
            entries,
        }
    }
}

/// Identifies the mip level, layer or slice, and rectangle for a texture copy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureCopyRegion {
    pub mip_level: u32,
    pub origin_x: u32,
    pub origin_y: u32,
    pub origin_z: u32,
    pub width: u32,
    pub height: u32,
}

impl TextureCopyRegion {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            mip_level: 0,
            origin_x: 0,
            origin_y: 0,
            origin_z: 0,
            width,
            height,
        }
    }

    pub const fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.mip_level = mip_level;
        self
    }

    pub const fn with_origin(mut self, x: u32, y: u32, z: u32) -> Self {
        self.origin_x = x;
        self.origin_y = y;
        self.origin_z = z;
        self
    }
}

/// Color value used when a render pass clears a color attachment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RenderClearColor {
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn is_finite(self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite() && self.a.is_finite()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RenderPassColorLoadOp {
    Load,
    Clear(RenderClearColor),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RenderPassDepthLoadOp {
    Load,
    Clear(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderPassStencilLoadOp {
    Load,
    Clear(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderPassStoreOp {
    Store,
    Discard,
}

/// Identifies the texture subresource used by a render-pass attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPassTextureViewDesc {
    pub texture: TextureHandle,
    pub mip_level: u32,
    pub array_layer: u32,
}

impl RenderPassTextureViewDesc {
    pub const fn new(texture: TextureHandle) -> Self {
        Self {
            texture,
            mip_level: 0,
            array_layer: 0,
        }
    }

    pub const fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.mip_level = mip_level;
        self
    }

    pub const fn with_array_layer(mut self, array_layer: u32) -> Self {
        self.array_layer = array_layer;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPassColorAttachmentDesc {
    pub view: RenderPassTextureViewDesc,
    pub resolve_target: Option<RenderPassTextureViewDesc>,
    pub load: RenderPassColorLoadOp,
    pub store: RenderPassStoreOp,
}

impl RenderPassColorAttachmentDesc {
    pub const fn new(
        texture: TextureHandle,
        load: RenderPassColorLoadOp,
        store: RenderPassStoreOp,
    ) -> Self {
        Self {
            view: RenderPassTextureViewDesc::new(texture),
            resolve_target: None,
            load,
            store,
        }
    }

    pub const fn with_view(mut self, view: RenderPassTextureViewDesc) -> Self {
        self.view = view;
        self
    }

    pub const fn with_resolve_target(mut self, resolve_target: TextureHandle) -> Self {
        self.resolve_target = Some(RenderPassTextureViewDesc::new(resolve_target));
        self
    }

    pub const fn with_resolve_view(mut self, resolve_target: RenderPassTextureViewDesc) -> Self {
        self.resolve_target = Some(resolve_target);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPassDepthStencilAttachmentDesc {
    pub view: RenderPassTextureViewDesc,
    pub depth_load: RenderPassDepthLoadOp,
    pub depth_store: RenderPassStoreOp,
    pub stencil_load: Option<RenderPassStencilLoadOp>,
    pub stencil_store: Option<RenderPassStoreOp>,
}

impl RenderPassDepthStencilAttachmentDesc {
    pub const fn depth(
        texture: TextureHandle,
        depth_load: RenderPassDepthLoadOp,
        depth_store: RenderPassStoreOp,
    ) -> Self {
        Self {
            view: RenderPassTextureViewDesc::new(texture),
            depth_load,
            depth_store,
            stencil_load: None,
            stencil_store: None,
        }
    }

    pub const fn with_stencil(
        mut self,
        stencil_load: RenderPassStencilLoadOp,
        stencil_store: RenderPassStoreOp,
    ) -> Self {
        self.stencil_load = Some(stencil_load);
        self.stencil_store = Some(stencil_store);
        self
    }

    pub const fn with_view(mut self, view: RenderPassTextureViewDesc) -> Self {
        self.view = view;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderViewportDesc {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl RenderViewportDesc {
    pub const fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderScissorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl RenderScissorRect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CommandListCommand {
    DebugMarker {
        label: String,
    },
    PushDebugGroup {
        label: String,
    },
    PopDebugGroup,
    CopyBufferToBuffer {
        source: BufferHandle,
        destination: BufferHandle,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    },
    CopyBufferToTexture {
        source: BufferHandle,
        destination: TextureHandle,
        source_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    },
    CopyTextureToBuffer {
        source: TextureHandle,
        destination: BufferHandle,
        destination_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    },
    BeginRenderPass {
        label: String,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
    },
    EndRenderPass,
    SetPipeline {
        pipeline: PipelineHandle,
    },
    SetBindGroup {
        slot: u32,
        bind_group: BindGroupHandle,
    },
    SetViewport {
        viewport: RenderViewportDesc,
    },
    SetScissorRect {
        rect: RenderScissorRect,
    },
    SetVertexBuffer {
        slot: u32,
        buffer: BufferHandle,
        offset: u64,
        size: u64,
    },
    SetIndexBuffer {
        buffer: BufferHandle,
        offset: u64,
        size: u64,
        format: IndexFormat,
    },
    Draw {
        vertex_start: u32,
        vertex_count: u32,
        instance_start: u32,
        instance_count: u32,
    },
    DrawIndexed {
        index_start: u32,
        index_count: u32,
        base_vertex: i32,
        instance_start: u32,
        instance_count: u32,
    },
    DispatchCompute {
        x: u32,
        y: u32,
        z: u32,
    },
}

pub trait CommandList: Send {
    fn queue_class(&self) -> RenderQueueClass;
    fn label(&self) -> Option<&str>;
    fn recorded_commands(&self) -> &[CommandListCommand];

    fn recorded_command_count(&self) -> usize {
        self.recorded_commands().len()
    }

    fn push_debug_marker(&mut self, label: &str);

    fn push_debug_group(&mut self, label: &str);

    fn pop_debug_group(&mut self);

    fn copy_buffer_to_buffer(
        &mut self,
        source: BufferHandle,
        destination: BufferHandle,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    );

    fn copy_buffer_to_texture(
        &mut self,
        source: BufferHandle,
        destination: TextureHandle,
        source_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    );

    fn copy_texture_to_buffer(
        &mut self,
        source: TextureHandle,
        destination: BufferHandle,
        destination_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    );

    fn begin_render_pass(
        &mut self,
        label: &str,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
    );

    fn end_render_pass(&mut self);

    fn set_pipeline(&mut self, pipeline: PipelineHandle);

    fn set_bind_group(&mut self, slot: u32, bind_group: BindGroupHandle);

    fn set_viewport(&mut self, viewport: RenderViewportDesc);

    fn set_scissor_rect(&mut self, rect: RenderScissorRect);

    fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferHandle, offset: u64, size: u64);

    fn set_index_buffer(
        &mut self,
        buffer: BufferHandle,
        offset: u64,
        size: u64,
        format: IndexFormat,
    );

    fn draw(
        &mut self,
        vertex_start: u32,
        vertex_count: u32,
        instance_start: u32,
        instance_count: u32,
    );

    fn draw_indexed(
        &mut self,
        index_start: u32,
        index_count: u32,
        base_vertex: i32,
        instance_start: u32,
        instance_count: u32,
    );

    fn dispatch_compute(&mut self, x: u32, y: u32, z: u32);
}

pub trait RenderDevice: Send + Sync {
    fn caps(&self) -> &RenderBackendCaps;

    fn backend_name(&self) -> &str {
        &self.caps().backend_name
    }

    fn debug_instrumentation_status(&self) -> RenderDebugInstrumentationStatus {
        RenderDebugInstrumentationStatus::from_caps(self.caps())
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError>;
    fn buffer_desc(&self, handle: BufferHandle) -> Result<BufferDesc, RhiError>;
    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError>;
    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError>;
    fn texture_desc(&self, handle: TextureHandle) -> Result<TextureDesc, RhiError>;
    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError>;
    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError>;
    fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError>;
    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError>;
    fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDesc,
    ) -> Result<BindGroupLayoutHandle, RhiError>;
    fn bind_group_layout_desc(
        &self,
        handle: BindGroupLayoutHandle,
    ) -> Result<BindGroupLayoutDesc, RhiError>;
    fn destroy_bind_group_layout(&self, handle: BindGroupLayoutHandle) -> Result<(), RhiError>;
    fn create_bind_group(&self, desc: &BindGroupDesc) -> Result<BindGroupHandle, RhiError>;
    fn bind_group_desc(&self, handle: BindGroupHandle) -> Result<BindGroupDesc, RhiError>;
    fn destroy_bind_group(&self, handle: BindGroupHandle) -> Result<(), RhiError>;
    fn create_shader_module(&self, desc: &ShaderModuleDesc)
        -> Result<ShaderModuleHandle, RhiError>;
    fn shader_module_desc(&self, handle: ShaderModuleHandle) -> Result<ShaderModuleDesc, RhiError>;
    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError>;
    fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDesc,
    ) -> Result<PipelineLayoutHandle, RhiError>;
    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<PipelineLayoutDesc, RhiError>;
    fn destroy_pipeline_layout(&self, handle: PipelineLayoutHandle) -> Result<(), RhiError>;
    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError>;
    fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError>;
    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError>;
    fn create_command_list(
        &self,
        queue_class: RenderQueueClass,
        label: impl Into<String>,
    ) -> Result<Box<dyn CommandList>, RhiError>;
    fn submit(&self, command_list: Box<dyn CommandList>) -> Result<FenceValue, RhiError>;
    fn is_fence_complete(&self, fence: FenceValue) -> Result<bool, RhiError>;
    fn transient_allocator_stats(&self) -> TransientAllocatorStats;
    fn write_buffer(&self, handle: BufferHandle, offset: u64, data: &[u8]) -> Result<(), RhiError>;
    fn read_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, RhiError>;
    fn read_texture(&self, handle: TextureHandle) -> Result<Vec<u8>, RhiError>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FenceValue(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransientAllocatorStats {
    pub bytes_reserved: u64,
    pub allocations: u32,
}
