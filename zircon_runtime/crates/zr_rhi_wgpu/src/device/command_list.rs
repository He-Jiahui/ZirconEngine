use zr_rhi::{
    BindGroupHandle, BufferHandle, CommandList, CommandListCommand, IndexFormat, PipelineHandle,
    RenderPassColorAttachmentDesc, RenderPassDepthStencilAttachmentDesc, RenderQueueClass,
    RenderScissorRect, RenderViewportDesc, TextureCopyRegion, TextureHandle,
};

#[derive(Clone, Debug)]
pub(crate) struct DeterministicRhiContractCommandList {
    queue_class: RenderQueueClass,
    label: Option<String>,
    commands: Vec<CommandListCommand>,
}

impl DeterministicRhiContractCommandList {
    pub(crate) fn new(queue_class: RenderQueueClass, label: impl Into<String>) -> Self {
        Self {
            queue_class,
            label: Some(label.into()),
            commands: Vec::new(),
        }
    }
}

impl CommandList for DeterministicRhiContractCommandList {
    fn queue_class(&self) -> RenderQueueClass {
        self.queue_class
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn recorded_commands(&self) -> &[CommandListCommand] {
        &self.commands
    }

    fn push_debug_marker(&mut self, label: &str) {
        self.commands.push(CommandListCommand::DebugMarker {
            label: label.to_string(),
        });
    }

    fn push_debug_group(&mut self, label: &str) {
        self.commands.push(CommandListCommand::PushDebugGroup {
            label: label.to_string(),
        });
    }

    fn pop_debug_group(&mut self) {
        self.commands.push(CommandListCommand::PopDebugGroup);
    }

    fn copy_buffer_to_buffer(
        &mut self,
        source: BufferHandle,
        destination: BufferHandle,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    ) {
        self.commands.push(CommandListCommand::CopyBufferToBuffer {
            source,
            destination,
            source_offset,
            destination_offset,
            size,
        });
    }

    fn copy_buffer_to_texture(
        &mut self,
        source: BufferHandle,
        destination: TextureHandle,
        source_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    ) {
        self.commands.push(CommandListCommand::CopyBufferToTexture {
            source,
            destination,
            source_offset,
            bytes_per_row,
            region,
        });
    }

    fn copy_texture_to_buffer(
        &mut self,
        source: TextureHandle,
        destination: BufferHandle,
        destination_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    ) {
        self.commands.push(CommandListCommand::CopyTextureToBuffer {
            source,
            destination,
            destination_offset,
            bytes_per_row,
            region,
        });
    }

    fn begin_render_pass(
        &mut self,
        label: &str,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
    ) {
        self.commands.push(CommandListCommand::BeginRenderPass {
            label: label.to_string(),
            color_attachments,
            depth_stencil_attachment,
        });
    }

    fn end_render_pass(&mut self) {
        self.commands.push(CommandListCommand::EndRenderPass);
    }

    fn set_pipeline(&mut self, pipeline: PipelineHandle) {
        self.commands
            .push(CommandListCommand::SetPipeline { pipeline });
    }

    fn set_bind_group(&mut self, slot: u32, bind_group: BindGroupHandle) {
        self.commands
            .push(CommandListCommand::SetBindGroup { slot, bind_group });
    }

    fn set_viewport(&mut self, viewport: RenderViewportDesc) {
        self.commands
            .push(CommandListCommand::SetViewport { viewport });
    }

    fn set_scissor_rect(&mut self, rect: RenderScissorRect) {
        self.commands
            .push(CommandListCommand::SetScissorRect { rect });
    }

    fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferHandle, offset: u64, size: u64) {
        self.commands.push(CommandListCommand::SetVertexBuffer {
            slot,
            buffer,
            offset,
            size,
        });
    }

    fn set_index_buffer(
        &mut self,
        buffer: BufferHandle,
        offset: u64,
        size: u64,
        format: IndexFormat,
    ) {
        self.commands.push(CommandListCommand::SetIndexBuffer {
            buffer,
            offset,
            size,
            format,
        });
    }

    fn draw(
        &mut self,
        vertex_start: u32,
        vertex_count: u32,
        instance_start: u32,
        instance_count: u32,
    ) {
        self.commands.push(CommandListCommand::Draw {
            vertex_start,
            vertex_count,
            instance_start,
            instance_count,
        });
    }

    fn draw_indexed(
        &mut self,
        index_start: u32,
        index_count: u32,
        base_vertex: i32,
        instance_start: u32,
        instance_count: u32,
    ) {
        self.commands.push(CommandListCommand::DrawIndexed {
            index_start,
            index_count,
            base_vertex,
            instance_start,
            instance_count,
        });
    }

    fn dispatch_compute(&mut self, x: u32, y: u32, z: u32) {
        self.commands
            .push(CommandListCommand::DispatchCompute { x, y, z });
    }
}
