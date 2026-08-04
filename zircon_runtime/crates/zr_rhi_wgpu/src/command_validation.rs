use zr_rhi::{BufferUsage, CommandListCommand, RenderQueueClass, RhiError, TextureUsage};

mod render_state;

use self::render_state::{
    ensure_binding_range, validate_bind_group_slot, validate_index_range, CommandResourceLookup,
    IndexBufferBinding, RecordedRenderState,
};
use super::bind_group_validation::validate_bind_group_desc;
use super::device::DeterministicRhiContractDeviceState;
use super::render_pass_validation::{validate_render_pass_attachments, ActiveRenderPass};
use super::resource_validation::{ensure_buffer_usage, ensure_texture_usage};
use super::texture_copy::texture_copy_layout;

pub(super) fn validate_recorded_commands(
    state: &DeterministicRhiContractDeviceState,
    commands: &[CommandListCommand],
    queue_class: RenderQueueClass,
) -> Result<(), RhiError> {
    let mut render_state = RecordedRenderState::default();
    let mut active_render_pass: Option<ActiveRenderPass> = None;
    let mut debug_group_stack = Vec::new();
    for command in commands {
        match command {
            CommandListCommand::DebugMarker { label } => {
                validate_debug_label(label, "debug marker")?;
            }
            CommandListCommand::PushDebugGroup { label } => {
                validate_debug_label(label, "debug group")?;
                debug_group_stack.push(DebugGroupScope::from_render_pass_state(
                    active_render_pass.is_some(),
                ));
            }
            CommandListCommand::PopDebugGroup => {
                let expected_scope =
                    DebugGroupScope::from_render_pass_state(active_render_pass.is_some());
                match debug_group_stack.last().copied() {
                    Some(scope) if scope == expected_scope => {
                        debug_group_stack.pop();
                    }
                    Some(DebugGroupScope::CommandEncoder) => {
                        return Err(RhiError::InvalidDebugMarker {
                            reason:
                                "pop_debug_group must close a debug group recorded outside the active render pass"
                                    .to_string(),
                        });
                    }
                    Some(DebugGroupScope::RenderPass) => {
                        return Err(RhiError::InvalidDebugMarker {
                            reason:
                                "pop_debug_group must close a debug group recorded in the active render pass"
                                    .to_string(),
                        });
                    }
                    None => {
                        return Err(RhiError::InvalidDebugMarker {
                            reason: "pop_debug_group requires an active debug group".to_string(),
                        });
                    }
                }
            }
            CommandListCommand::CopyBufferToBuffer {
                source,
                destination,
                source_offset,
                destination_offset,
                size,
            } => {
                ensure_no_active_render_pass(&active_render_pass, "copy_buffer_to_buffer")?;
                let source_buffer = state
                    .buffers
                    .get(source)
                    .ok_or(RhiError::UnknownBuffer(source.raw()))?;
                let destination_buffer = state
                    .buffers
                    .get(destination)
                    .ok_or(RhiError::UnknownBuffer(destination.raw()))?;
                ensure_buffer_usage(source.raw(), &source_buffer.desc, BufferUsage::COPY_SRC)?;
                ensure_buffer_usage(
                    destination.raw(),
                    &destination_buffer.desc,
                    BufferUsage::COPY_DST,
                )?;
                let source_end = source_offset.saturating_add(*size);
                let destination_end = destination_offset.saturating_add(*size);
                if source_end > source_buffer.desc.size_bytes
                    || destination_end > destination_buffer.desc.size_bytes
                {
                    return Err(RhiError::BufferCopyOutOfRange {
                        source_buffer: source.raw(),
                        destination_buffer: destination.raw(),
                        source_offset: *source_offset,
                        destination_offset: *destination_offset,
                        size: *size,
                    });
                }
            }
            CommandListCommand::CopyBufferToTexture {
                source,
                destination,
                source_offset,
                bytes_per_row,
                region,
            } => {
                ensure_no_active_render_pass(&active_render_pass, "copy_buffer_to_texture")?;
                let source_buffer = state
                    .buffers
                    .get(source)
                    .ok_or(RhiError::UnknownBuffer(source.raw()))?;
                let destination_texture = state
                    .textures
                    .get(destination)
                    .ok_or(RhiError::UnknownTexture(destination.raw()))?;
                ensure_buffer_usage(source.raw(), &source_buffer.desc, BufferUsage::COPY_SRC)?;
                ensure_texture_usage(
                    destination.raw(),
                    &destination_texture.desc,
                    TextureUsage::COPY_DST,
                )?;
                let Some(layout) = texture_copy_layout(&destination_texture.desc, *region) else {
                    return Err(RhiError::BufferToTextureCopyOutOfRange {
                        source_buffer: source.raw(),
                        destination_texture: destination.raw(),
                        source_offset: *source_offset,
                        bytes_per_row: *bytes_per_row,
                        mip_level: region.mip_level,
                        origin_x: region.origin_x,
                        origin_y: region.origin_y,
                        origin_z: region.origin_z,
                        width: region.width,
                        height: region.height,
                    });
                };
                let row_size = layout.copy_row_bytes;
                let copy_size =
                    buffer_to_texture_copy_size(region.height, *bytes_per_row, row_size);
                if *bytes_per_row < row_size
                    || source_offset.saturating_add(copy_size) > source_buffer.desc.size_bytes
                    || layout.last_row_end > destination_texture.contents.len() as u64
                {
                    return Err(RhiError::BufferToTextureCopyOutOfRange {
                        source_buffer: source.raw(),
                        destination_texture: destination.raw(),
                        source_offset: *source_offset,
                        bytes_per_row: *bytes_per_row,
                        mip_level: region.mip_level,
                        origin_x: region.origin_x,
                        origin_y: region.origin_y,
                        origin_z: region.origin_z,
                        width: region.width,
                        height: region.height,
                    });
                }
            }
            CommandListCommand::CopyTextureToBuffer {
                source,
                destination,
                destination_offset,
                bytes_per_row,
                region,
            } => {
                ensure_no_active_render_pass(&active_render_pass, "copy_texture_to_buffer")?;
                let source_texture = state
                    .textures
                    .get(source)
                    .ok_or(RhiError::UnknownTexture(source.raw()))?;
                let destination_buffer = state
                    .buffers
                    .get(destination)
                    .ok_or(RhiError::UnknownBuffer(destination.raw()))?;
                ensure_texture_usage(source.raw(), &source_texture.desc, TextureUsage::COPY_SRC)?;
                ensure_buffer_usage(
                    destination.raw(),
                    &destination_buffer.desc,
                    BufferUsage::COPY_DST,
                )?;
                let Some(layout) = texture_copy_layout(&source_texture.desc, *region) else {
                    return Err(RhiError::TextureToBufferCopyOutOfRange {
                        source_texture: source.raw(),
                        destination_buffer: destination.raw(),
                        destination_offset: *destination_offset,
                        bytes_per_row: *bytes_per_row,
                        mip_level: region.mip_level,
                        origin_x: region.origin_x,
                        origin_y: region.origin_y,
                        origin_z: region.origin_z,
                        width: region.width,
                        height: region.height,
                    });
                };
                let row_size = layout.copy_row_bytes;
                let copy_size =
                    buffer_to_texture_copy_size(region.height, *bytes_per_row, row_size);
                if *bytes_per_row < row_size
                    || destination_offset.saturating_add(copy_size)
                        > destination_buffer.desc.size_bytes
                    || layout.last_row_end > source_texture.contents.len() as u64
                {
                    return Err(RhiError::TextureToBufferCopyOutOfRange {
                        source_texture: source.raw(),
                        destination_buffer: destination.raw(),
                        destination_offset: *destination_offset,
                        bytes_per_row: *bytes_per_row,
                        mip_level: region.mip_level,
                        origin_x: region.origin_x,
                        origin_y: region.origin_y,
                        origin_z: region.origin_z,
                        width: region.width,
                        height: region.height,
                    });
                }
            }
            CommandListCommand::BeginRenderPass {
                label: _,
                color_attachments,
                depth_stencil_attachment,
            } => {
                validate_raster_queue(queue_class, "begin_render_pass")?;
                if active_render_pass.is_some() {
                    return Err(RhiError::InvalidRenderPass {
                        reason: "render pass is already active".to_string(),
                    });
                }
                let attachment_info = validate_render_pass_attachments(
                    state,
                    color_attachments,
                    *depth_stencil_attachment,
                )?;
                active_render_pass = Some(ActiveRenderPass::new(
                    color_attachments,
                    *depth_stencil_attachment,
                    attachment_info,
                ));
            }
            CommandListCommand::EndRenderPass => {
                if debug_group_stack
                    .last()
                    .is_some_and(|scope| *scope == DebugGroupScope::RenderPass)
                {
                    return Err(RhiError::InvalidDebugMarker {
                        reason: "render pass ended with an active debug group".to_string(),
                    });
                }
                if active_render_pass.take().is_none() {
                    return Err(RhiError::InvalidRenderPass {
                        reason: "end_render_pass requires an active render pass".to_string(),
                    });
                }
            }
            CommandListCommand::SetPipeline { pipeline } => {
                let pipeline_desc = state
                    .pipelines
                    .get(pipeline)
                    .ok_or(RhiError::UnknownPipeline(pipeline.raw()))?;
                render_state.set_pipeline(*pipeline, pipeline_desc);
            }
            CommandListCommand::SetBindGroup { slot, bind_group } => {
                let bind_group_desc = state.bind_group_desc_ref(*bind_group)?;
                validate_bind_group_desc(state, bind_group_desc)?;
                if let Some((_pipeline, _kind, pipeline_desc)) = render_state.current_pipeline {
                    validate_bind_group_slot(
                        state,
                        pipeline_desc,
                        *slot,
                        *bind_group,
                        bind_group_desc,
                    )?;
                }
                render_state.set_bind_group(*slot, *bind_group);
            }
            CommandListCommand::SetViewport { viewport } => {
                let render_pass = require_active_render_pass(&active_render_pass, "set_viewport")?;
                render_pass.validate_viewport(*viewport)?;
            }
            CommandListCommand::SetScissorRect { rect } => {
                let render_pass =
                    require_active_render_pass(&active_render_pass, "set_scissor_rect")?;
                render_pass.validate_scissor_rect(*rect)?;
            }
            CommandListCommand::SetVertexBuffer {
                slot,
                buffer,
                offset,
                size,
            } => {
                let desc = state.buffer_desc(*buffer)?;
                ensure_buffer_usage(buffer.raw(), desc, BufferUsage::VERTEX)?;
                ensure_binding_range(*buffer, desc, *offset, *size)?;
                render_state.set_vertex_buffer(*slot, *size);
            }
            CommandListCommand::SetIndexBuffer {
                buffer,
                offset,
                size,
                format,
            } => {
                let desc = state.buffer_desc(*buffer)?;
                ensure_buffer_usage(buffer.raw(), desc, BufferUsage::INDEX)?;
                ensure_binding_range(*buffer, desc, *offset, *size)?;
                if *size < format.size_bytes() {
                    return Err(RhiError::InvalidRasterDraw {
                        reason: "index buffer binding must contain at least one index".to_string(),
                    });
                }
                if *size % format.size_bytes() != 0 {
                    return Err(RhiError::InvalidRasterDraw {
                        reason: format!(
                            "index buffer binding size must be aligned to {:?}",
                            format
                        ),
                    });
                }
                render_state.index_buffer = Some(IndexBufferBinding {
                    size_bytes: *size,
                    format: *format,
                });
            }
            CommandListCommand::Draw {
                vertex_start,
                vertex_count,
                instance_start,
                instance_count,
            } => {
                validate_raster_queue(queue_class, "draw")?;
                ensure_non_zero_draw_counts(*vertex_count, *instance_count)?;
                let pipeline = render_state.require_raster_pipeline()?;
                let render_pass = require_active_render_pass(&active_render_pass, "draw")?;
                render_pass.validate_pipeline_attachments(state, pipeline)?;
                render_state.ensure_required_bind_groups(state, pipeline, "draw")?;
                render_state.ensure_required_vertex_buffers(pipeline)?;
                render_state.validate_vertex_ranges(
                    pipeline,
                    *vertex_start,
                    *vertex_count,
                    *instance_start,
                    *instance_count,
                    true,
                )?;
            }
            CommandListCommand::DrawIndexed {
                index_start,
                index_count,
                base_vertex: _,
                instance_start,
                instance_count,
            } => {
                validate_raster_queue(queue_class, "draw_indexed")?;
                ensure_non_zero_draw_counts(*index_count, *instance_count)?;
                let pipeline = render_state.require_raster_pipeline()?;
                let render_pass = require_active_render_pass(&active_render_pass, "draw_indexed")?;
                render_pass.validate_pipeline_attachments(state, pipeline)?;
                render_state.ensure_required_bind_groups(state, pipeline, "draw_indexed")?;
                render_state.ensure_required_vertex_buffers(pipeline)?;
                let index_buffer =
                    render_state
                        .index_buffer
                        .ok_or_else(|| RhiError::InvalidRasterDraw {
                            reason: "draw_indexed requires a bound index buffer".to_string(),
                        })?;
                validate_index_range(index_buffer, *index_start, *index_count)?;
                render_state.validate_vertex_ranges(
                    pipeline,
                    0,
                    0,
                    *instance_start,
                    *instance_count,
                    false,
                )?;
            }
            CommandListCommand::DispatchCompute { x, y, z } => {
                ensure_no_active_render_pass(&active_render_pass, "dispatch_compute")?;
                if queue_class == RenderQueueClass::Copy {
                    return Err(RhiError::InvalidCommandQueue {
                        queue: queue_class,
                        command: "dispatch_compute".to_string(),
                    });
                }
                if *x == 0 || *y == 0 || *z == 0 {
                    return Err(RhiError::InvalidComputeDispatch {
                        reason: "workgroup counts must be greater than zero".to_string(),
                    });
                }
                let pipeline = render_state.require_compute_pipeline()?;
                render_state.ensure_required_bind_groups(state, pipeline, "dispatch_compute")?;
            }
        }
    }
    if active_render_pass.is_some() {
        return Err(RhiError::InvalidRenderPass {
            reason: "command list ended with an active render pass".to_string(),
        });
    }
    if !debug_group_stack.is_empty() {
        return Err(RhiError::InvalidDebugMarker {
            reason: "command list ended with an active debug group".to_string(),
        });
    }
    Ok(())
}

pub(super) fn execute_recorded_commands(
    state: &mut DeterministicRhiContractDeviceState,
    commands: &[CommandListCommand],
) -> Result<(), RhiError> {
    for command in commands {
        match command {
            CommandListCommand::DebugMarker { .. }
            | CommandListCommand::PushDebugGroup { .. }
            | CommandListCommand::PopDebugGroup => {}
            CommandListCommand::CopyBufferToBuffer {
                source,
                destination,
                source_offset,
                destination_offset,
                size,
            } => {
                let source_start = *source_offset as usize;
                let source_end = source_start + *size as usize;
                let destination_start = *destination_offset as usize;
                let destination_end = destination_start + *size as usize;
                if source == destination {
                    state
                        .buffers
                        .get_mut(source)
                        .ok_or(RhiError::UnknownBuffer(source.raw()))?
                        .contents
                        .copy_within(source_start..source_end, destination_start);
                } else {
                    let [source_buffer, destination_buffer] =
                        state.buffers.get_disjoint_mut([source, destination]);
                    let source_buffer =
                        source_buffer.ok_or(RhiError::UnknownBuffer(source.raw()))?;
                    let destination_buffer =
                        destination_buffer.ok_or(RhiError::UnknownBuffer(destination.raw()))?;
                    destination_buffer.contents[destination_start..destination_end]
                        .copy_from_slice(&source_buffer.contents[source_start..source_end]);
                }
            }
            CommandListCommand::CopyBufferToTexture {
                source,
                destination,
                source_offset,
                bytes_per_row,
                region,
            } => {
                let (buffers, textures) = (&state.buffers, &mut state.textures);
                let source_contents = &buffers
                    .get(source)
                    .ok_or(RhiError::UnknownBuffer(source.raw()))?
                    .contents;
                let destination_texture = textures
                    .get_mut(destination)
                    .ok_or(RhiError::UnknownTexture(destination.raw()))?;
                let layout =
                    texture_copy_layout(&destination_texture.desc, *region).ok_or_else(|| {
                        RhiError::BufferToTextureCopyOutOfRange {
                            source_buffer: source.raw(),
                            destination_texture: destination.raw(),
                            source_offset: *source_offset,
                            bytes_per_row: *bytes_per_row,
                            mip_level: region.mip_level,
                            origin_x: region.origin_x,
                            origin_y: region.origin_y,
                            origin_z: region.origin_z,
                            width: region.width,
                            height: region.height,
                        }
                    })?;
                let row_size = layout.copy_row_bytes as usize;
                let source_offset = *source_offset as usize;
                let bytes_per_row = *bytes_per_row as usize;
                for row in 0..region.height as usize {
                    let source_start = source_offset + row * bytes_per_row;
                    let source_end = source_start + row_size;
                    let destination_start =
                        layout.offset as usize + row * layout.row_stride as usize;
                    let destination_end = destination_start + row_size;
                    destination_texture.contents[destination_start..destination_end]
                        .copy_from_slice(&source_contents[source_start..source_end]);
                }
            }
            CommandListCommand::CopyTextureToBuffer {
                source,
                destination,
                destination_offset,
                bytes_per_row,
                region,
            } => {
                let (textures, buffers) = (&state.textures, &mut state.buffers);
                let source_texture = textures
                    .get(source)
                    .ok_or(RhiError::UnknownTexture(source.raw()))?;
                let layout =
                    texture_copy_layout(&source_texture.desc, *region).ok_or_else(|| {
                        RhiError::TextureToBufferCopyOutOfRange {
                            source_texture: source.raw(),
                            destination_buffer: destination.raw(),
                            destination_offset: *destination_offset,
                            bytes_per_row: *bytes_per_row,
                            mip_level: region.mip_level,
                            origin_x: region.origin_x,
                            origin_y: region.origin_y,
                            origin_z: region.origin_z,
                            width: region.width,
                            height: region.height,
                        }
                    })?;
                let row_size = layout.copy_row_bytes as usize;
                let destination_offset = *destination_offset as usize;
                let bytes_per_row = *bytes_per_row as usize;
                let destination_buffer = buffers
                    .get_mut(destination)
                    .ok_or(RhiError::UnknownBuffer(destination.raw()))?;
                for row in 0..region.height as usize {
                    let source_start = layout.offset as usize + row * layout.row_stride as usize;
                    let source_end = source_start + row_size;
                    let destination_start = destination_offset + row * bytes_per_row;
                    let destination_end = destination_start + row_size;
                    destination_buffer.contents[destination_start..destination_end]
                        .copy_from_slice(&source_texture.contents[source_start..source_end]);
                }
            }
            CommandListCommand::SetPipeline { .. }
            | CommandListCommand::BeginRenderPass { .. }
            | CommandListCommand::EndRenderPass
            | CommandListCommand::SetBindGroup { .. }
            | CommandListCommand::SetViewport { .. }
            | CommandListCommand::SetScissorRect { .. }
            | CommandListCommand::SetVertexBuffer { .. }
            | CommandListCommand::SetIndexBuffer { .. }
            | CommandListCommand::Draw { .. }
            | CommandListCommand::DrawIndexed { .. }
            | CommandListCommand::DispatchCompute { .. } => {}
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DebugGroupScope {
    CommandEncoder,
    RenderPass,
}

impl DebugGroupScope {
    const fn from_render_pass_state(in_render_pass: bool) -> Self {
        if in_render_pass {
            Self::RenderPass
        } else {
            Self::CommandEncoder
        }
    }
}

fn validate_debug_label(label: &str, role: &str) -> Result<(), RhiError> {
    if label.is_empty() {
        return Err(RhiError::InvalidDebugMarker {
            reason: format!("{role} label must not be empty"),
        });
    }
    Ok(())
}

fn validate_raster_queue(queue_class: RenderQueueClass, command: &str) -> Result<(), RhiError> {
    if queue_class == RenderQueueClass::Graphics {
        Ok(())
    } else {
        Err(RhiError::InvalidCommandQueue {
            queue: queue_class,
            command: command.to_string(),
        })
    }
}

fn require_active_render_pass<'a>(
    active_render_pass: &'a Option<ActiveRenderPass>,
    command: &str,
) -> Result<&'a ActiveRenderPass, RhiError> {
    active_render_pass
        .as_ref()
        .ok_or_else(|| RhiError::InvalidRenderPass {
            reason: format!("{command} requires an active render pass"),
        })
}

fn ensure_no_active_render_pass(
    active_render_pass: &Option<ActiveRenderPass>,
    command: &str,
) -> Result<(), RhiError> {
    if active_render_pass.is_some() {
        Err(RhiError::InvalidRenderPass {
            reason: format!("{command} cannot be recorded inside an active render pass"),
        })
    } else {
        Ok(())
    }
}

fn ensure_non_zero_draw_counts(draw_count: u32, instance_count: u32) -> Result<(), RhiError> {
    if draw_count == 0 || instance_count == 0 {
        Err(RhiError::InvalidRasterDraw {
            reason: "draw and instance counts must be greater than zero".to_string(),
        })
    } else {
        Ok(())
    }
}

fn buffer_to_texture_copy_size(height: u32, bytes_per_row: u64, row_size: u64) -> u64 {
    if height == 0 {
        0
    } else {
        u64::from(height - 1)
            .saturating_mul(bytes_per_row)
            .saturating_add(row_size)
    }
}
