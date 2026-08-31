use zr_rhi::{
    CommandListCommand, DiagnosticPassQueryScope, PipelineKind, RenderPassColorAttachmentDesc,
    RenderPassColorLoadOp, RenderPassDepthLoadOp, RenderPassDepthStencilAttachmentDesc,
    RenderPassStencilLoadOp, RenderPassStoreOp, RenderQueueClass, RhiError,
};

use crate::bind_group_validation::validate_bind_group_dynamic_offsets;
use crate::indirect_validation::{
    validate_indirect_arguments, validate_indirect_count_buffer, IndirectArgumentKind,
};
use crate::render_pass_validation::{validate_render_pass_attachments, ActiveRenderPass};

use super::super::diagnostics::WgpuDiagnosticQueryFrame;
use super::super::registry::WgpuResourceRegistry;
use super::state::{
    bind_render_state, ensure_non_zero_draw_counts, pop_debug_group, require_graphics_queue,
    validate_bind_group_slot, validate_debug_label, validate_draw_bindings, validate_index_buffer,
    validate_index_range, validate_required_vertex_buffers, validate_vertex_buffer,
    validate_vertex_ranges, BufferBinding, DebugGroupScope, EncoderState, IndexBufferBinding,
};

/// Encodes a complete neutral render pass and its pass-local state.
#[allow(clippy::too_many_arguments)]
pub(crate) fn encode_render_pass(
    encoder: &mut wgpu::CommandEncoder,
    registry: &WgpuResourceRegistry,
    commands: &[CommandListCommand],
    begin_index: usize,
    label: &str,
    color_attachments: &[RenderPassColorAttachmentDesc],
    depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
    state: &mut EncoderState,
    debug_groups: &mut Vec<DebugGroupScope>,
    queue_class: RenderQueueClass,
    diagnostic_scope: Option<(&WgpuDiagnosticQueryFrame, DiagnosticPassQueryScope)>,
    limits: &zr_rhi::RenderDeviceLimits,
) -> Result<usize, RhiError> {
    let attachment_info =
        validate_render_pass_attachments(registry, color_attachments, depth_stencil_attachment)?;
    let active_render_pass =
        ActiveRenderPass::new(color_attachments, depth_stencil_attachment, attachment_info);
    let color_views = color_attachments
        .iter()
        .map(|attachment| {
            Ok((
                create_attachment_view(registry, attachment.view)?,
                attachment
                    .resolve_target
                    .map(|resolve| create_attachment_view(registry, resolve))
                    .transpose()?,
                color_operations(attachment.load, attachment.store),
            ))
        })
        .collect::<Result<Vec<_>, RhiError>>()?;
    let native_color_attachments = color_views
        .iter()
        .map(|(view, resolve_target, ops)| {
            Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: resolve_target.as_ref(),
                ops: *ops,
            })
        })
        .collect::<Vec<_>>();
    let depth_view = depth_stencil_attachment
        .map(|attachment| -> Result<_, RhiError> {
            Ok((
                create_attachment_view(registry, attachment.view)?,
                attachment,
            ))
        })
        .transpose()?;
    let native_depth_attachment =
        depth_view.as_ref().map(
            |(view, attachment)| wgpu::RenderPassDepthStencilAttachment {
                view,
                depth_ops: Some(depth_operations(
                    attachment.depth_load,
                    attachment.depth_store,
                )),
                stencil_ops: attachment
                    .stencil_load
                    .zip(attachment.stencil_store)
                    .map(|(load, store)| stencil_operations(load, store)),
            },
        );
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &native_color_attachments,
        depth_stencil_attachment: native_depth_attachment,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    let pipeline_statistics_scope =
        diagnostic_scope.and_then(|(_, scope)| scope.pipeline_statistics());
    if let Some(scope) = pipeline_statistics_scope {
        let query_set = diagnostic_scope
            .and_then(|(frame, _)| frame.pipeline_statistics_query_set())
            .ok_or(RhiError::DiagnosticQueryPlanRequired)?;
        pass.begin_pipeline_statistics_query(query_set, scope.query_index());
    }

    let mut command_index = begin_index + 1;
    while command_index < commands.len() {
        match &commands[command_index] {
            CommandListCommand::EndRenderPass => {
                if debug_groups
                    .last()
                    .is_some_and(|scope| *scope == DebugGroupScope::RenderPass)
                {
                    return Err(RhiError::InvalidDebugMarker {
                        reason: "render pass ended with an active debug group".to_string(),
                    });
                }
                if pipeline_statistics_scope.is_some() {
                    pass.end_pipeline_statistics_query();
                }
                return Ok(command_index + 1);
            }
            CommandListCommand::BeginRenderPass { .. }
            | CommandListCommand::BeginRenderPassWithDiagnostics { .. } => {
                return Err(RhiError::InvalidRenderPass {
                    reason: "render pass is already active".to_string(),
                });
            }
            CommandListCommand::BeginComputePass { .. }
            | CommandListCommand::BeginComputePassWithDiagnostics { .. }
            | CommandListCommand::EndComputePass => {
                return Err(RhiError::InvalidRenderPass {
                    reason: "compute pass commands cannot be recorded inside an active render pass"
                        .to_string(),
                });
            }
            CommandListCommand::DebugMarker { label } => {
                validate_debug_label(label, "debug marker")?;
                pass.insert_debug_marker(label);
            }
            CommandListCommand::PushDebugGroup { label } => {
                validate_debug_label(label, "debug group")?;
                pass.push_debug_group(label);
                debug_groups.push(DebugGroupScope::RenderPass);
            }
            CommandListCommand::PopDebugGroup => {
                pop_debug_group(debug_groups, DebugGroupScope::RenderPass)?;
                pass.pop_debug_group();
            }
            CommandListCommand::SetPipeline { pipeline } => {
                state.set_pipeline(*pipeline, registry.pipeline_desc(*pipeline)?);
            }
            CommandListCommand::SetBindGroup {
                slot,
                bind_group,
                dynamic_offsets,
            } => {
                let bind_group_desc = registry.bind_group_desc(*bind_group)?;
                validate_bind_group_dynamic_offsets(
                    registry,
                    *bind_group,
                    &bind_group_desc,
                    dynamic_offsets,
                    limits,
                )?;
                if let Some((_, pipeline)) = state.current_pipeline.as_ref() {
                    validate_bind_group_slot(registry, pipeline, *slot, *bind_group)?;
                }
                state.set_bind_group(*slot, *bind_group, dynamic_offsets.clone());
            }
            CommandListCommand::SetViewport { viewport } => {
                active_render_pass.validate_viewport(*viewport)?;
                pass.set_viewport(
                    viewport.x,
                    viewport.y,
                    viewport.width,
                    viewport.height,
                    viewport.min_depth,
                    viewport.max_depth,
                );
            }
            CommandListCommand::SetScissorRect { rect } => {
                active_render_pass.validate_scissor_rect(*rect)?;
                pass.set_scissor_rect(rect.x, rect.y, rect.width, rect.height);
            }
            CommandListCommand::SetVertexBuffer {
                slot,
                buffer,
                offset,
                size,
            } => {
                validate_vertex_buffer(registry, *buffer, *offset, *size)?;
                state.vertex_buffers.insert(
                    *slot,
                    BufferBinding {
                        handle: *buffer,
                        offset: *offset,
                        size: *size,
                    },
                );
            }
            CommandListCommand::SetIndexBuffer {
                buffer,
                offset,
                size,
                format,
            } => {
                validate_index_buffer(registry, *buffer, *offset, *size, *format)?;
                state.index_buffer = Some(IndexBufferBinding {
                    handle: *buffer,
                    offset: *offset,
                    size: *size,
                    format: *format,
                });
            }
            CommandListCommand::Draw {
                vertex_start,
                vertex_count,
                instance_start,
                instance_count,
            } => {
                require_graphics_queue(queue_class, "draw")?;
                ensure_non_zero_draw_counts(*vertex_count, *instance_count)?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(registry, state, pipeline, "draw")?;
                validate_vertex_ranges(
                    state,
                    pipeline,
                    *vertex_start,
                    *vertex_count,
                    *instance_start,
                    *instance_count,
                    true,
                )?;
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.draw(
                    *vertex_start..vertex_start.saturating_add(*vertex_count),
                    *instance_start..instance_start.saturating_add(*instance_count),
                );
            }
            CommandListCommand::DrawIndexed {
                index_start,
                index_count,
                base_vertex,
                instance_start,
                instance_count,
            } => {
                require_graphics_queue(queue_class, "draw_indexed")?;
                ensure_non_zero_draw_counts(*index_count, *instance_count)?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(registry, state, pipeline, "draw_indexed")?;
                let index_binding =
                    state
                        .index_buffer
                        .ok_or_else(|| RhiError::InvalidRasterDraw {
                            reason: "draw_indexed requires a bound index buffer".to_string(),
                        })?;
                validate_index_range(index_binding, *index_start, *index_count)?;
                validate_vertex_ranges(
                    state,
                    pipeline,
                    0,
                    0,
                    *instance_start,
                    *instance_count,
                    false,
                )?;
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.draw_indexed(
                    *index_start..index_start.saturating_add(*index_count),
                    *base_vertex,
                    *instance_start..instance_start.saturating_add(*instance_count),
                );
            }
            CommandListCommand::DrawIndirect { arguments, offset } => {
                require_graphics_queue(queue_class, "draw_indirect")?;
                validate_indirect_arguments(
                    *arguments,
                    &registry.buffer_desc(*arguments)?,
                    *offset,
                    1,
                    IndirectArgumentKind::Draw,
                )?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(registry, state, pipeline, "draw_indirect")?;
                validate_required_vertex_buffers(state, pipeline)?;
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.draw_indirect(registry.buffer(*arguments)?, *offset);
            }
            CommandListCommand::DrawIndexedIndirect { arguments, offset } => {
                require_graphics_queue(queue_class, "draw_indexed_indirect")?;
                validate_indirect_arguments(
                    *arguments,
                    &registry.buffer_desc(*arguments)?,
                    *offset,
                    1,
                    IndirectArgumentKind::IndexedDraw,
                )?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(registry, state, pipeline, "draw_indexed_indirect")?;
                validate_required_vertex_buffers(state, pipeline)?;
                if state.index_buffer.is_none() {
                    return Err(RhiError::InvalidRasterDraw {
                        reason: "draw_indexed_indirect requires a bound index buffer".to_string(),
                    });
                }
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.draw_indexed_indirect(registry.buffer(*arguments)?, *offset);
            }
            CommandListCommand::MultiDrawIndirect {
                arguments,
                offset,
                count,
            } => {
                require_graphics_queue(queue_class, "multi_draw_indirect")?;
                validate_indirect_arguments(
                    *arguments,
                    &registry.buffer_desc(*arguments)?,
                    *offset,
                    *count,
                    IndirectArgumentKind::Draw,
                )?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(registry, state, pipeline, "multi_draw_indirect")?;
                validate_required_vertex_buffers(state, pipeline)?;
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.multi_draw_indirect(registry.buffer(*arguments)?, *offset, *count);
            }
            CommandListCommand::MultiDrawIndexedIndirect {
                arguments,
                offset,
                count,
            } => {
                require_graphics_queue(queue_class, "multi_draw_indexed_indirect")?;
                validate_indirect_arguments(
                    *arguments,
                    &registry.buffer_desc(*arguments)?,
                    *offset,
                    *count,
                    IndirectArgumentKind::IndexedDraw,
                )?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(registry, state, pipeline, "multi_draw_indexed_indirect")?;
                validate_required_vertex_buffers(state, pipeline)?;
                if state.index_buffer.is_none() {
                    return Err(RhiError::InvalidRasterDraw {
                        reason: "multi_draw_indexed_indirect requires a bound index buffer"
                            .to_string(),
                    });
                }
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.multi_draw_indexed_indirect(registry.buffer(*arguments)?, *offset, *count);
            }
            CommandListCommand::MultiDrawIndirectCount {
                arguments,
                offset,
                count_buffer,
                count_offset,
                max_count,
            } => {
                require_graphics_queue(queue_class, "multi_draw_indirect_count")?;
                validate_indirect_arguments(
                    *arguments,
                    &registry.buffer_desc(*arguments)?,
                    *offset,
                    *max_count,
                    IndirectArgumentKind::Draw,
                )?;
                validate_indirect_count_buffer(
                    *count_buffer,
                    &registry.buffer_desc(*count_buffer)?,
                    *count_offset,
                )?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(registry, state, pipeline, "multi_draw_indirect_count")?;
                validate_required_vertex_buffers(state, pipeline)?;
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.multi_draw_indirect_count(
                    registry.buffer(*arguments)?,
                    *offset,
                    registry.buffer(*count_buffer)?,
                    *count_offset,
                    *max_count,
                );
            }
            CommandListCommand::MultiDrawIndexedIndirectCount {
                arguments,
                offset,
                count_buffer,
                count_offset,
                max_count,
            } => {
                require_graphics_queue(queue_class, "multi_draw_indexed_indirect_count")?;
                validate_indirect_arguments(
                    *arguments,
                    &registry.buffer_desc(*arguments)?,
                    *offset,
                    *max_count,
                    IndirectArgumentKind::IndexedDraw,
                )?;
                validate_indirect_count_buffer(
                    *count_buffer,
                    &registry.buffer_desc(*count_buffer)?,
                    *count_offset,
                )?;
                let (_, pipeline) = state.require_pipeline(PipelineKind::Raster)?;
                active_render_pass.validate_pipeline_attachments(registry, pipeline)?;
                validate_draw_bindings(
                    registry,
                    state,
                    pipeline,
                    "multi_draw_indexed_indirect_count",
                )?;
                validate_required_vertex_buffers(state, pipeline)?;
                if state.index_buffer.is_none() {
                    return Err(RhiError::InvalidRasterDraw {
                        reason: "multi_draw_indexed_indirect_count requires a bound index buffer"
                            .to_string(),
                    });
                }
                bind_render_state(&mut pass, registry, state, pipeline)?;
                pass.multi_draw_indexed_indirect_count(
                    registry.buffer(*arguments)?,
                    *offset,
                    registry.buffer(*count_buffer)?,
                    *count_offset,
                    *max_count,
                );
            }
            CommandListCommand::CopyBufferToBuffer { .. }
            | CommandListCommand::CopyBufferToTexture { .. }
            | CommandListCommand::CopyTextureToBuffer { .. }
            | CommandListCommand::CopyTextureToTexture { .. }
            | CommandListCommand::DispatchCompute { .. }
            | CommandListCommand::DispatchComputeIndirect { .. } => {
                return Err(RhiError::InvalidRenderPass {
                    reason:
                        "copy and compute commands cannot be recorded inside an active render pass"
                            .to_string(),
                });
            }
        }
        command_index += 1;
    }

    Err(RhiError::InvalidRenderPass {
        reason: "command list ended with an active render pass".to_string(),
    })
}

fn create_attachment_view(
    registry: &WgpuResourceRegistry,
    view: zr_rhi::RenderPassTextureViewDesc,
) -> Result<wgpu::TextureView, RhiError> {
    if let Some(registered_view) = view.registered_view {
        return Ok(registry.texture_view(registered_view)?.clone());
    }
    Ok(registry
        .texture(view.texture)?
        .create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_mip_level: view.mip_level,
            mip_level_count: Some(1),
            base_array_layer: view.array_layer,
            array_layer_count: Some(1),
            ..Default::default()
        }))
}

fn color_operations(
    load: RenderPassColorLoadOp,
    store: RenderPassStoreOp,
) -> wgpu::Operations<wgpu::Color> {
    wgpu::Operations {
        load: match load {
            RenderPassColorLoadOp::Load => wgpu::LoadOp::Load,
            RenderPassColorLoadOp::Clear(color) => wgpu::LoadOp::Clear(wgpu::Color {
                r: f64::from(color.r),
                g: f64::from(color.g),
                b: f64::from(color.b),
                a: f64::from(color.a),
            }),
        },
        store: wgpu_store_op(store),
    }
}

fn depth_operations(
    load: RenderPassDepthLoadOp,
    store: RenderPassStoreOp,
) -> wgpu::Operations<f32> {
    wgpu::Operations {
        load: match load {
            RenderPassDepthLoadOp::Load => wgpu::LoadOp::Load,
            RenderPassDepthLoadOp::Clear(value) => wgpu::LoadOp::Clear(value),
        },
        store: wgpu_store_op(store),
    }
}

fn stencil_operations(
    load: RenderPassStencilLoadOp,
    store: RenderPassStoreOp,
) -> wgpu::Operations<u32> {
    wgpu::Operations {
        load: match load {
            RenderPassStencilLoadOp::Load => wgpu::LoadOp::Load,
            RenderPassStencilLoadOp::Clear(value) => wgpu::LoadOp::Clear(value),
        },
        store: wgpu_store_op(store),
    }
}

const fn wgpu_store_op(store: RenderPassStoreOp) -> wgpu::StoreOp {
    match store {
        RenderPassStoreOp::Store => wgpu::StoreOp::Store,
        RenderPassStoreOp::Discard => wgpu::StoreOp::Discard,
    }
}
