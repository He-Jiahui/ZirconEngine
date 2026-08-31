use zr_rhi::{
    BindGroupEntryResource, BindGroupHandle, BindGroupLayoutHandle, BufferHandle,
    CommandListCommand, PipelineHandle, PipelineLayoutHandle, RenderPassTextureViewDesc, RhiError,
    SamplerHandle, ShaderModuleHandle, SubmissionTicket, SurfaceFrameId, TextureHandle,
    TextureViewHandle,
};

use super::WgpuResourceRegistry;

impl WgpuResourceRegistry {
    /// Records all direct and descriptor-transitive resources retained by one
    /// encoded packet before that packet can reach the native queue.
    pub(crate) fn mark_command_list_use(
        &mut self,
        ticket: SubmissionTicket,
        commands: &[CommandListCommand],
    ) -> Result<(), RhiError> {
        for command in commands {
            match command {
                CommandListCommand::CopyBufferToBuffer {
                    source,
                    destination,
                    ..
                } => {
                    self.mark_buffer_use(*source, ticket)?;
                    self.mark_buffer_use(*destination, ticket)?;
                }
                CommandListCommand::CopyBufferToTexture {
                    source,
                    destination,
                    ..
                } => {
                    self.mark_buffer_use(*source, ticket)?;
                    self.mark_texture_use(*destination, ticket)?;
                }
                CommandListCommand::CopyTextureToBuffer {
                    source,
                    destination,
                    ..
                } => {
                    self.mark_texture_use(*source, ticket)?;
                    self.mark_buffer_use(*destination, ticket)?;
                }
                CommandListCommand::CopyTextureToTexture {
                    source,
                    destination,
                    ..
                } => {
                    self.mark_texture_use(*source, ticket)?;
                    self.mark_texture_use(*destination, ticket)?;
                }
                CommandListCommand::BeginRenderPass {
                    color_attachments,
                    depth_stencil_attachment,
                    ..
                }
                | CommandListCommand::BeginRenderPassWithDiagnostics {
                    color_attachments,
                    depth_stencil_attachment,
                    ..
                } => {
                    for attachment in color_attachments {
                        self.mark_render_pass_attachment_use(attachment.view, ticket)?;
                        if let Some(resolve_target) = attachment.resolve_target {
                            self.mark_render_pass_attachment_use(resolve_target, ticket)?;
                        }
                    }
                    if let Some(depth_stencil_attachment) = depth_stencil_attachment {
                        self.mark_render_pass_attachment_use(
                            depth_stencil_attachment.view,
                            ticket,
                        )?;
                    }
                }
                CommandListCommand::SetPipeline { pipeline } => {
                    self.mark_pipeline_use(*pipeline, ticket)?;
                }
                CommandListCommand::SetBindGroup { bind_group, .. } => {
                    self.mark_bind_group_use(*bind_group, ticket)?;
                }
                CommandListCommand::SetVertexBuffer { buffer, .. }
                | CommandListCommand::SetIndexBuffer { buffer, .. } => {
                    self.mark_buffer_use(*buffer, ticket)?;
                }
                CommandListCommand::DrawIndirect { arguments, .. }
                | CommandListCommand::DrawIndexedIndirect { arguments, .. }
                | CommandListCommand::MultiDrawIndirect { arguments, .. }
                | CommandListCommand::MultiDrawIndexedIndirect { arguments, .. }
                | CommandListCommand::DispatchComputeIndirect { arguments, .. } => {
                    self.mark_buffer_use(*arguments, ticket)?;
                }
                CommandListCommand::MultiDrawIndirectCount {
                    arguments,
                    count_buffer,
                    ..
                }
                | CommandListCommand::MultiDrawIndexedIndirectCount {
                    arguments,
                    count_buffer,
                    ..
                } => {
                    self.mark_buffer_use(*arguments, ticket)?;
                    self.mark_buffer_use(*count_buffer, ticket)?;
                }
                CommandListCommand::DebugMarker { .. }
                | CommandListCommand::PushDebugGroup { .. }
                | CommandListCommand::PopDebugGroup
                | CommandListCommand::EndRenderPass
                | CommandListCommand::BeginComputePass { .. }
                | CommandListCommand::BeginComputePassWithDiagnostics { .. }
                | CommandListCommand::EndComputePass
                | CommandListCommand::SetViewport { .. }
                | CommandListCommand::SetScissorRect { .. }
                | CommandListCommand::Draw { .. }
                | CommandListCommand::DrawIndexed { .. }
                | CommandListCommand::DispatchCompute { .. } => {}
            }
        }
        Ok(())
    }

    fn mark_render_pass_attachment_use(
        &mut self,
        attachment: RenderPassTextureViewDesc,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        match attachment.registered_view {
            Some(view) => self.mark_texture_view_use(view, ticket),
            None => self.mark_texture_use(attachment.texture, ticket),
        }
    }

    /// Records a queued CPU-to-GPU upload as the buffer's final use until the
    /// upload receipt reaches a terminal state.
    pub(crate) fn mark_buffer_upload_use(
        &mut self,
        handle: BufferHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.mark_buffer_use(handle, ticket)
    }

    /// Records a queued CPU-to-GPU texture update as the texture's final use
    /// until its submission receipt reaches a terminal state.
    pub(crate) fn mark_texture_upload_use(
        &mut self,
        handle: TextureHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.mark_texture_use(handle, ticket)
    }

    /// Retains a buffer source used by a diagnostic copy until the exact
    /// submission-qualified map lifecycle reaches a terminal state.
    pub(crate) fn mark_buffer_diagnostic_readback_use(
        &mut self,
        handle: BufferHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.mark_buffer_use(handle, ticket)
    }

    /// Retains a texture source used by a diagnostic subresource copy until
    /// the exact submission-qualified map lifecycle reaches a terminal state.
    pub(crate) fn mark_texture_diagnostic_readback_use(
        &mut self,
        handle: TextureHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.mark_texture_use(handle, ticket)
    }

    pub(crate) fn mark_native_surface_frame_use(
        &mut self,
        frame: SurfaceFrameId,
        target: TextureHandle,
        default_view: TextureViewHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_texture(target)?;
        self.handles.validate_texture_view(default_view)?;
        let view_matches_target = self
            .texture_views
            .get(&default_view)
            .is_some_and(|resource| resource.desc.texture == target);
        if !self.surface_owned_textures.contains(&target)
            || !self.surface_owned_texture_views.contains(&default_view)
            || !view_matches_target
        {
            return Err(RhiError::SurfaceFrameLeaseMismatch { frame });
        }
        self.mark_texture_view_use(default_view, ticket)
    }

    fn mark_buffer_use(
        &mut self,
        handle: BufferHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_buffer(handle)?;
        let resource = self
            .buffers
            .get_mut(&handle)
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        Ok(())
    }

    fn mark_texture_use(
        &mut self,
        handle: TextureHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_texture(handle)?;
        let resource = self
            .textures
            .get_mut(&handle)
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        if self.surface_owned_textures.contains(&handle) {
            self.surface_frame_submissions
                .entry(handle)
                .or_default()
                .insert(ticket);
        }
        Ok(())
    }

    fn mark_texture_view_use(
        &mut self,
        handle: TextureViewHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_texture_view(handle)?;
        let texture = self
            .texture_views
            .get(&handle)
            .map(|resource| resource.desc.texture)
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))?;
        let resource = self
            .texture_views
            .get_mut(&handle)
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        self.mark_texture_use(texture, ticket)
    }

    fn mark_sampler_use(
        &mut self,
        handle: SamplerHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_sampler(handle)?;
        let resource = self
            .samplers
            .get_mut(&handle)
            .ok_or(RhiError::UnknownSampler(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        Ok(())
    }

    fn mark_bind_group_layout_use(
        &mut self,
        handle: BindGroupLayoutHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_bind_group_layout(handle)?;
        let resource = self
            .bind_group_layouts
            .get_mut(&handle)
            .ok_or(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        Ok(())
    }

    fn mark_bind_group_use(
        &mut self,
        handle: BindGroupHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_bind_group(handle)?;
        let desc = self
            .bind_groups
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownBindGroup(handle.diagnostic_id()))?;
        let resource = self
            .bind_groups
            .get_mut(&handle)
            .ok_or(RhiError::UnknownBindGroup(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        self.mark_bind_group_layout_use(desc.layout, ticket)?;
        for entry in desc.entries {
            match entry.resource {
                BindGroupEntryResource::Buffer(binding) => {
                    self.mark_buffer_use(binding.buffer, ticket)?
                }
                BindGroupEntryResource::TextureView(view) => {
                    self.mark_texture_view_use(view, ticket)?
                }
                BindGroupEntryResource::Sampler(sampler) => {
                    self.mark_sampler_use(sampler, ticket)?
                }
            }
        }
        Ok(())
    }

    fn mark_shader_module_use(
        &mut self,
        handle: ShaderModuleHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_shader_module(handle)?;
        let resource = self
            .shader_modules
            .get_mut(&handle)
            .ok_or(RhiError::UnknownShaderModule(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        Ok(())
    }

    fn mark_pipeline_layout_use(
        &mut self,
        handle: PipelineLayoutHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_pipeline_layout(handle)?;
        let desc = self
            .pipeline_layouts
            .get(&handle)
            .map(|resource| resource.desc.clone())
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))?;
        let resource = self
            .pipeline_layouts
            .get_mut(&handle)
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))?;
        record_use(&mut resource.last_uses, ticket);
        for bind_group_layout in desc.bind_group_layouts {
            self.mark_bind_group_layout_use(bind_group_layout, ticket)?;
        }
        Ok(())
    }

    fn mark_pipeline_use(
        &mut self,
        handle: PipelineHandle,
        ticket: SubmissionTicket,
    ) -> Result<(), RhiError> {
        self.handles.validate_pipeline(handle)?;
        let desc = self
            .pipelines
            .get(&handle)
            .map(|resource| resource.desc().clone())
            .ok_or(RhiError::UnknownPipeline(handle.diagnostic_id()))?;
        let resource = self
            .pipelines
            .get_mut(&handle)
            .ok_or(RhiError::UnknownPipeline(handle.diagnostic_id()))?;
        record_use(resource.last_uses_mut(), ticket);
        if let Some(layout) = desc.layout {
            self.mark_pipeline_layout_use(layout, ticket)?;
        }
        for shader in [
            desc.vertex_shader,
            desc.fragment_shader,
            desc.compute_shader,
        ]
        .into_iter()
        .flatten()
        {
            self.mark_shader_module_use(shader, ticket)?;
        }
        Ok(())
    }
}

fn record_use(uses: &mut Vec<SubmissionTicket>, ticket: SubmissionTicket) {
    if !uses.contains(&ticket) {
        uses.push(ticket);
    }
}
