use serde::{Deserialize, Serialize};

use super::capabilities::{RenderOperation, RenderQueueClass};
use super::texture_copy::TextureCopyRegion;

mod error;
mod handles;
mod render_device;
mod render_pass;

pub use self::error::RhiError;
pub use self::handles::{
    BindGroupHandle, BindGroupLayoutHandle, BufferHandle, PipelineHandle, PipelineLayoutHandle,
    RenderResourceHandleAllocationError, RenderResourceHandleAllocator, RenderResourceHandleError,
    RenderResourceKind, SamplerHandle, ShaderModuleHandle, TextureHandle, TextureViewHandle,
};
pub use self::render_device::RenderDevice;
pub use self::render_pass::{
    RenderClearColor, RenderPassColorAttachmentDesc, RenderPassColorLoadOp, RenderPassDepthLoadOp,
    RenderPassDepthStencilAttachmentDesc, RenderPassStencilLoadOp, RenderPassStoreOp,
    RenderPassTextureViewDesc,
};

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
pub struct BindGroupBufferBinding {
    pub buffer: BufferHandle,
    pub offset: u64,
    pub size: Option<u64>,
}

impl BindGroupBufferBinding {
    pub const fn new(buffer: BufferHandle, offset: u64, size: Option<u64>) -> Self {
        Self {
            buffer,
            offset,
            size,
        }
    }

    /// Binds the buffer from byte zero through its declared extent.
    pub const fn whole(buffer: BufferHandle) -> Self {
        Self::new(buffer, 0, None)
    }
}

/// A bind-group entry owns an explicit buffer subrange, a texture view, or a
/// sampler. Dynamic offsets are recorded on `CommandList::set_bind_group` so
/// the group descriptor stays immutable and reusable across draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindGroupEntryResource {
    Buffer(BindGroupBufferBinding),
    TextureView(TextureViewHandle),
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
    CopyTextureToTexture {
        source: TextureHandle,
        destination: TextureHandle,
        source_region: TextureCopyRegion,
        destination_region: TextureCopyRegion,
    },
    BeginRenderPass {
        label: String,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
    },
    BeginRenderPassWithDiagnostics {
        label: String,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
        diagnostic_scope: super::diagnostic_query::DiagnosticPassQueryScope,
    },
    EndRenderPass,
    BeginComputePass {
        label: String,
    },
    BeginComputePassWithDiagnostics {
        label: String,
        diagnostic_scope: super::diagnostic_query::DiagnosticPassQueryScope,
    },
    EndComputePass,
    SetPipeline {
        pipeline: PipelineHandle,
    },
    SetBindGroup {
        slot: u32,
        bind_group: BindGroupHandle,
        dynamic_offsets: Vec<u32>,
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
    DrawIndirect {
        arguments: BufferHandle,
        offset: u64,
    },
    DrawIndexedIndirect {
        arguments: BufferHandle,
        offset: u64,
    },
    MultiDrawIndirect {
        arguments: BufferHandle,
        offset: u64,
        count: u32,
    },
    MultiDrawIndexedIndirect {
        arguments: BufferHandle,
        offset: u64,
        count: u32,
    },
    MultiDrawIndirectCount {
        arguments: BufferHandle,
        offset: u64,
        count_buffer: BufferHandle,
        count_offset: u64,
        max_count: u32,
    },
    MultiDrawIndexedIndirectCount {
        arguments: BufferHandle,
        offset: u64,
        count_buffer: BufferHandle,
        count_offset: u64,
        max_count: u32,
    },
    DispatchCompute {
        x: u32,
        y: u32,
        z: u32,
    },
    DispatchComputeIndirect {
        arguments: BufferHandle,
        offset: u64,
    },
}

impl CommandListCommand {
    /// Returns the neutral operation that must have been admitted before this
    /// encoded command can execute. State-only commands inherit admission from
    /// their enclosing pass or draw/dispatch command.
    pub fn required_operation(&self) -> Option<RenderOperation> {
        match self {
            Self::DebugMarker { .. } => Some(RenderOperation::DebugMarker),
            Self::PushDebugGroup { .. } | Self::PopDebugGroup => Some(RenderOperation::DebugGroup),
            Self::CopyBufferToBuffer { .. } => Some(RenderOperation::BufferToBufferCopy),
            Self::CopyBufferToTexture { .. } => Some(RenderOperation::BufferToTextureCopy),
            Self::CopyTextureToBuffer { .. } => Some(RenderOperation::TextureToBufferCopy),
            Self::CopyTextureToTexture { .. } => Some(RenderOperation::TextureToTextureCopy),
            Self::Draw { .. } => Some(RenderOperation::DirectDraw),
            Self::DrawIndexed { .. } => Some(RenderOperation::IndexedDraw),
            Self::DrawIndirect { .. } | Self::DrawIndexedIndirect { .. } => {
                Some(RenderOperation::IndirectDraw)
            }
            Self::MultiDrawIndirect { .. } | Self::MultiDrawIndexedIndirect { .. } => {
                Some(RenderOperation::MultiDrawIndirect)
            }
            Self::MultiDrawIndirectCount { .. } | Self::MultiDrawIndexedIndirectCount { .. } => {
                Some(RenderOperation::MultiDrawIndirectCount)
            }
            Self::DispatchCompute { .. } => Some(RenderOperation::ComputeDispatch),
            Self::DispatchComputeIndirect { .. } => Some(RenderOperation::ComputeDispatchIndirect),
            Self::BeginRenderPass { .. }
            | Self::BeginRenderPassWithDiagnostics { .. }
            | Self::EndRenderPass
            | Self::BeginComputePass { .. }
            | Self::BeginComputePassWithDiagnostics { .. }
            | Self::EndComputePass
            | Self::SetPipeline { .. }
            | Self::SetBindGroup { .. }
            | Self::SetViewport { .. }
            | Self::SetScissorRect { .. }
            | Self::SetVertexBuffer { .. }
            | Self::SetIndexBuffer { .. } => None,
        }
    }
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
    /// Records a same-format, single-sampled color subresource copy. Source
    /// and destination must be distinct physical textures with equal copy
    /// extents; depth/stencil and reinterpretation copies remain deferred.
    fn copy_texture_to_texture(
        &mut self,
        source: TextureHandle,
        destination: TextureHandle,
        source_region: TextureCopyRegion,
        destination_region: TextureCopyRegion,
    );

    fn begin_render_pass(
        &mut self,
        label: &str,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
    );

    /// Begins a pass whose timestamp and/or pipeline-statistics range was
    /// allocated by the diagnostic plan attached to its submission packet.
    fn begin_render_pass_with_diagnostics(
        &mut self,
        label: &str,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
        diagnostic_scope: super::diagnostic_query::DiagnosticPassQueryScope,
    );

    fn end_render_pass(&mut self);

    /// Begins an explicit compute scope. Dispatches outside this scope remain
    /// valid for simple one-dispatch command lists, while graph materializers
    /// use this boundary to retain one logical compute pass.
    fn begin_compute_pass(&mut self, label: &str);

    /// Begins a compute pass whose query range belongs to its submission
    /// packet's diagnostic plan.
    fn begin_compute_pass_with_diagnostics(
        &mut self,
        label: &str,
        diagnostic_scope: super::diagnostic_query::DiagnosticPassQueryScope,
    );

    fn end_compute_pass(&mut self);

    fn set_pipeline(&mut self, pipeline: PipelineHandle);

    fn set_bind_group(&mut self, slot: u32, bind_group: BindGroupHandle) {
        self.set_bind_group_with_dynamic_offsets(slot, bind_group, Vec::new());
    }

    /// Binds one immutable descriptor set with the dynamic offsets declared
    /// by its layout, in ascending layout-binding order.
    fn set_bind_group_with_dynamic_offsets(
        &mut self,
        slot: u32,
        bind_group: BindGroupHandle,
        dynamic_offsets: Vec<u32>,
    );

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

    /// Records one non-indexed draw whose four `u32` arguments reside in an
    /// `INDIRECT` buffer at a four-byte-aligned byte offset.
    fn draw_indirect(&mut self, arguments: BufferHandle, offset: u64);

    /// Records one indexed draw whose five-word argument ABI resides in an
    /// `INDIRECT` buffer at a four-byte-aligned byte offset.
    fn draw_indexed_indirect(&mut self, arguments: BufferHandle, offset: u64);

    /// Records a fixed number of non-indexed indirect draws. GPU-written
    /// count-buffer variants remain a separate optional capability.
    fn multi_draw_indirect(&mut self, arguments: BufferHandle, offset: u64, count: u32);

    /// Records a fixed number of indexed indirect draws. GPU-written
    /// count-buffer variants remain a separate optional capability.
    fn multi_draw_indexed_indirect(&mut self, arguments: BufferHandle, offset: u64, count: u32);

    /// Records non-indexed indirect draws whose actual count is read from one
    /// `u32` in an `INDIRECT` count buffer. The argument range covers
    /// `max_count` tightly packed commands.
    fn multi_draw_indirect_count(
        &mut self,
        arguments: BufferHandle,
        offset: u64,
        count_buffer: BufferHandle,
        count_offset: u64,
        max_count: u32,
    );

    /// Records indexed indirect draws whose actual count is read from one
    /// `u32` in an `INDIRECT` count buffer. The argument range covers
    /// `max_count` tightly packed commands.
    fn multi_draw_indexed_indirect_count(
        &mut self,
        arguments: BufferHandle,
        offset: u64,
        count_buffer: BufferHandle,
        count_offset: u64,
        max_count: u32,
    );

    fn dispatch_compute(&mut self, x: u32, y: u32, z: u32);

    /// Records a compute dispatch whose three workgroup dimensions reside in
    /// an `INDIRECT` buffer at a four-byte-aligned byte offset.
    fn dispatch_compute_indirect(&mut self, arguments: BufferHandle, offset: u64);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransientAllocatorStats {
    pub bytes_reserved: u64,
    pub allocations: u32,
}
