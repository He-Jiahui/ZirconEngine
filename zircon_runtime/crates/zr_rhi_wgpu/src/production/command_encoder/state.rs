use std::collections::BTreeMap;

use zr_rhi::{
    BindGroupHandle, BufferHandle, BufferUsage, IndexFormat, PipelineDesc, PipelineHandle,
    PipelineKind, RenderQueueClass, RhiError,
};

use crate::resource_validation::ensure_buffer_usage;

use super::super::registry::WgpuResourceRegistry;

/// Neutral state recorded before native WGPU passes materialize it.
#[derive(Clone, Debug, Default)]
pub(crate) struct EncoderState {
    pub(crate) current_pipeline: Option<(PipelineHandle, PipelineDesc)>,
    pub(crate) bind_groups: BTreeMap<u32, BoundBindGroup>,
    pub(crate) vertex_buffers: BTreeMap<u32, BufferBinding>,
    pub(crate) index_buffer: Option<IndexBufferBinding>,
}

impl EncoderState {
    pub(crate) fn set_pipeline(&mut self, handle: PipelineHandle, desc: PipelineDesc) {
        self.current_pipeline = Some((handle, desc));
    }

    pub(crate) fn set_bind_group(
        &mut self,
        slot: u32,
        handle: BindGroupHandle,
        dynamic_offsets: Vec<u32>,
    ) {
        self.bind_groups.insert(
            slot,
            BoundBindGroup {
                handle,
                dynamic_offsets,
            },
        );
    }

    pub(crate) fn require_pipeline(
        &self,
        required: PipelineKind,
    ) -> Result<(PipelineHandle, &PipelineDesc), RhiError> {
        match self.current_pipeline.as_ref() {
            Some((handle, desc)) if desc.kind == required => Ok((*handle, desc)),
            Some((handle, desc)) => Err(RhiError::InvalidPipelineUsage {
                pipeline: handle.diagnostic_id(),
                required,
                actual: desc.kind,
            }),
            None if required == PipelineKind::Raster => Err(RhiError::InvalidRasterDraw {
                reason: "raster draw requires a bound raster pipeline".to_string(),
            }),
            None => Err(RhiError::InvalidComputeDispatch {
                reason: "compute dispatch requires a bound compute pipeline".to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BoundBindGroup {
    pub(crate) handle: BindGroupHandle,
    pub(crate) dynamic_offsets: Vec<u32>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BufferBinding {
    pub(crate) handle: BufferHandle,
    pub(crate) offset: u64,
    pub(crate) size: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IndexBufferBinding {
    pub(crate) handle: BufferHandle,
    pub(crate) offset: u64,
    pub(crate) size: u64,
    pub(crate) format: IndexFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DebugGroupScope {
    CommandEncoder,
    RenderPass,
    ComputePass,
}

pub(crate) fn bind_render_state(
    pass: &mut wgpu::RenderPass<'_>,
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    pipeline: &PipelineDesc,
) -> Result<(), RhiError> {
    let (pipeline_handle, _) = state.require_pipeline(PipelineKind::Raster)?;
    pass.set_pipeline(registry.render_pipeline(pipeline_handle)?);
    bind_groups_render(pass, registry, state, pipeline)?;
    for (slot, binding) in &state.vertex_buffers {
        pass.set_vertex_buffer(
            *slot,
            registry
                .buffer(binding.handle)?
                .slice(binding.offset..binding.offset.saturating_add(binding.size)),
        );
    }
    if let Some(binding) = state.index_buffer {
        pass.set_index_buffer(
            registry
                .buffer(binding.handle)?
                .slice(binding.offset..binding.offset.saturating_add(binding.size)),
            wgpu_index_format(binding.format),
        );
    }
    Ok(())
}

fn bind_groups_render(
    pass: &mut wgpu::RenderPass<'_>,
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    pipeline: &PipelineDesc,
) -> Result<(), RhiError> {
    for slot in required_bind_group_slots(registry, pipeline)? {
        let bind_group =
            state
                .bind_groups
                .get(&slot)
                .ok_or_else(|| RhiError::InvalidBindGroupUsage {
                    reason: format!("draw requires bind group slot {slot} to be bound"),
                })?;
        pass.set_bind_group(
            slot,
            registry.bind_group(bind_group.handle)?,
            &bind_group.dynamic_offsets,
        );
    }
    Ok(())
}

pub(crate) fn bind_groups_compute(
    pass: &mut wgpu::ComputePass<'_>,
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    pipeline: &PipelineDesc,
) -> Result<(), RhiError> {
    for slot in required_bind_group_slots(registry, pipeline)? {
        let bind_group =
            state
                .bind_groups
                .get(&slot)
                .ok_or_else(|| RhiError::InvalidBindGroupUsage {
                    reason: format!("dispatch_compute requires bind group slot {slot} to be bound"),
                })?;
        pass.set_bind_group(
            slot,
            registry.bind_group(bind_group.handle)?,
            &bind_group.dynamic_offsets,
        );
    }
    Ok(())
}

pub(crate) fn validate_draw_bindings(
    registry: &WgpuResourceRegistry,
    state: &EncoderState,
    pipeline: &PipelineDesc,
    command: &str,
) -> Result<(), RhiError> {
    for slot in required_bind_group_slots(registry, pipeline)? {
        let bind_group =
            state
                .bind_groups
                .get(&slot)
                .ok_or_else(|| RhiError::InvalidBindGroupUsage {
                    reason: format!("{command} requires bind group slot {slot} to be bound"),
                })?;
        validate_bind_group_slot(registry, pipeline, slot, bind_group.handle)?;
    }
    Ok(())
}

fn required_bind_group_slots(
    registry: &WgpuResourceRegistry,
    pipeline: &PipelineDesc,
) -> Result<std::ops::Range<u32>, RhiError> {
    let layout = pipeline
        .layout
        .ok_or_else(|| RhiError::InvalidBindGroupUsage {
            reason: "bound pipeline has no pipeline layout".to_string(),
        })?;
    let layout_desc = registry.pipeline_layout_desc(layout)?;
    Ok(0..layout_desc.bind_group_layouts.len() as u32)
}

pub(crate) fn validate_bind_group_slot(
    registry: &WgpuResourceRegistry,
    pipeline: &PipelineDesc,
    slot: u32,
    bind_group: BindGroupHandle,
) -> Result<(), RhiError> {
    let layout = pipeline
        .layout
        .ok_or_else(|| RhiError::InvalidBindGroupUsage {
            reason: "bound pipeline has no pipeline layout".to_string(),
        })?;
    let pipeline_layout = registry.pipeline_layout_desc(layout)?;
    let expected_layout = pipeline_layout
        .bind_group_layouts
        .get(slot as usize)
        .ok_or_else(|| RhiError::InvalidBindGroupUsage {
            reason: format!("bind group slot {slot} is not declared by the active pipeline layout"),
        })?;
    let bind_group_desc = registry.bind_group_desc(bind_group)?;
    if bind_group_desc.layout != *expected_layout {
        return Err(RhiError::InvalidBindGroupUsage {
            reason: format!(
                "bind group `{}` layout `{}` does not match pipeline layout slot {slot} `{}`",
                bind_group.diagnostic_id(),
                bind_group_desc.layout.diagnostic_id(),
                expected_layout.diagnostic_id(),
            ),
        });
    }
    Ok(())
}

pub(crate) fn validate_vertex_buffer(
    registry: &WgpuResourceRegistry,
    buffer: BufferHandle,
    offset: u64,
    size: u64,
) -> Result<(), RhiError> {
    let desc = registry.buffer_desc(buffer)?;
    ensure_buffer_usage(buffer.diagnostic_id(), &desc, BufferUsage::VERTEX)?;
    ensure_binding_range(buffer, desc.size_bytes, offset, size)
}

pub(crate) fn validate_index_buffer(
    registry: &WgpuResourceRegistry,
    buffer: BufferHandle,
    offset: u64,
    size: u64,
    format: IndexFormat,
) -> Result<(), RhiError> {
    let desc = registry.buffer_desc(buffer)?;
    ensure_buffer_usage(buffer.diagnostic_id(), &desc, BufferUsage::INDEX)?;
    ensure_binding_range(buffer, desc.size_bytes, offset, size)?;
    if offset % format.size_bytes() != 0 || size % format.size_bytes() != 0 {
        return Err(RhiError::InvalidRasterDraw {
            reason: format!("index buffer binding must be aligned to {format:?}"),
        });
    }
    Ok(())
}

fn ensure_binding_range(
    buffer: BufferHandle,
    buffer_size: u64,
    offset: u64,
    size: u64,
) -> Result<(), RhiError> {
    if size == 0 || offset.saturating_add(size) > buffer_size {
        Err(RhiError::BufferBindingOutOfRange {
            buffer: buffer.diagnostic_id(),
            offset,
            size,
        })
    } else {
        Ok(())
    }
}

pub(crate) fn validate_vertex_ranges(
    state: &EncoderState,
    pipeline: &PipelineDesc,
    vertex_start: u32,
    vertex_count: u32,
    instance_start: u32,
    instance_count: u32,
    validate_vertex_buffers: bool,
) -> Result<(), RhiError> {
    let raster_state =
        pipeline
            .raster_state
            .as_ref()
            .ok_or_else(|| RhiError::InvalidRasterDraw {
                reason: "bound raster pipeline has no raster state".to_string(),
            })?;
    if raster_state.vertex_input.buffers.is_empty() {
        if validate_vertex_buffers && vertex_start != 0 {
            return Err(RhiError::InvalidRasterDraw {
                reason: "generated-vertex draws must start at vertex 0".to_string(),
            });
        }
        return Ok(());
    }

    for (slot, layout) in raster_state.vertex_input.buffers.iter().enumerate() {
        let slot = slot as u32;
        let binding =
            state
                .vertex_buffers
                .get(&slot)
                .ok_or_else(|| RhiError::InvalidRasterDraw {
                    reason: format!("draw requires vertex buffer slot {slot} to be bound"),
                })?;
        let (start, count, role) = match layout.step_mode {
            zr_rhi::VertexStepMode::Vertex if validate_vertex_buffers => {
                (vertex_start, vertex_count, "vertex")
            }
            zr_rhi::VertexStepMode::Instance => (instance_start, instance_count, "instance"),
            zr_rhi::VertexStepMode::Vertex => continue,
        };
        validate_strided_range(slot, role, binding.size, layout, start, count)?;
    }
    Ok(())
}

pub(crate) fn validate_required_vertex_buffers(
    state: &EncoderState,
    pipeline: &PipelineDesc,
) -> Result<(), RhiError> {
    let raster_state =
        pipeline
            .raster_state
            .as_ref()
            .ok_or_else(|| RhiError::InvalidRasterDraw {
                reason: "bound raster pipeline has no raster state".to_string(),
            })?;
    for slot in 0..raster_state.vertex_input.buffers.len() as u32 {
        if !state.vertex_buffers.contains_key(&slot) {
            return Err(RhiError::InvalidRasterDraw {
                reason: format!("draw requires vertex buffer slot {slot} to be bound"),
            });
        }
    }
    Ok(())
}

fn validate_strided_range(
    slot: u32,
    role: &str,
    size_bytes: u64,
    layout: &zr_rhi::VertexBufferLayoutDesc,
    start: u32,
    count: u32,
) -> Result<(), RhiError> {
    let Some(first_element_offset) = u64::from(start).checked_mul(layout.array_stride) else {
        return invalid_vertex_range(slot, role);
    };
    let Some(tail_stride_bytes) =
        u64::from(count.saturating_sub(1)).checked_mul(layout.array_stride)
    else {
        return invalid_vertex_range(slot, role);
    };
    let max_attribute_end = layout
        .attributes
        .iter()
        .filter_map(|attribute| attribute.offset.checked_add(attribute.format.size_bytes()))
        .max()
        .unwrap_or(0);
    let Some(required_bytes) = first_element_offset
        .checked_add(tail_stride_bytes)
        .and_then(|value| value.checked_add(max_attribute_end))
    else {
        return invalid_vertex_range(slot, role);
    };
    if required_bytes > size_bytes {
        return Err(RhiError::InvalidRasterDraw {
            reason: format!("{role} draw range exceeds vertex buffer slot {slot}"),
        });
    }
    Ok(())
}

fn invalid_vertex_range(slot: u32, role: &str) -> Result<(), RhiError> {
    Err(RhiError::InvalidRasterDraw {
        reason: format!("{role} range overflows for vertex buffer slot {slot}"),
    })
}

pub(crate) fn validate_index_range(
    binding: IndexBufferBinding,
    index_start: u32,
    index_count: u32,
) -> Result<(), RhiError> {
    let first_index_byte = u64::from(index_start).saturating_mul(binding.format.size_bytes());
    let index_bytes = u64::from(index_count).saturating_mul(binding.format.size_bytes());
    if first_index_byte.saturating_add(index_bytes) > binding.size {
        Err(RhiError::InvalidRasterDraw {
            reason: "indexed draw range exceeds the bound index buffer".to_string(),
        })
    } else {
        Ok(())
    }
}

const fn wgpu_index_format(format: IndexFormat) -> wgpu::IndexFormat {
    match format {
        IndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
        IndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
    }
}

pub(crate) fn validate_debug_label(label: &str, role: &str) -> Result<(), RhiError> {
    if label.is_empty() {
        Err(RhiError::InvalidDebugMarker {
            reason: format!("{role} label must not be empty"),
        })
    } else {
        Ok(())
    }
}

pub(crate) fn pop_debug_group(
    groups: &mut Vec<DebugGroupScope>,
    expected: DebugGroupScope,
) -> Result<(), RhiError> {
    match groups.last().copied() {
        Some(scope) if scope == expected => {
            groups.pop();
            Ok(())
        }
        Some(DebugGroupScope::CommandEncoder) => Err(RhiError::InvalidDebugMarker {
            reason:
                "pop_debug_group must close a command-encoder debug group outside a render pass"
                    .to_string(),
        }),
        Some(DebugGroupScope::RenderPass) => Err(RhiError::InvalidDebugMarker {
            reason:
                "pop_debug_group must close a render-pass debug group inside the active render pass"
                    .to_string(),
        }),
        Some(DebugGroupScope::ComputePass) => Err(RhiError::InvalidDebugMarker {
            reason:
                "pop_debug_group must close a compute-pass debug group inside the active compute pass"
                    .to_string(),
        }),
        None => Err(RhiError::InvalidDebugMarker {
            reason: "pop_debug_group requires an active debug group".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{pop_debug_group, DebugGroupScope};
    use zr_rhi::RhiError;

    #[test]
    fn debug_group_pop_reports_compute_pass_scope_mismatch() {
        let mut groups = vec![DebugGroupScope::ComputePass];

        assert_eq!(
            pop_debug_group(&mut groups, DebugGroupScope::CommandEncoder).unwrap_err(),
            RhiError::InvalidDebugMarker {
                reason:
                    "pop_debug_group must close a compute-pass debug group inside the active compute pass"
                        .to_string(),
            }
        );
        assert_eq!(groups, vec![DebugGroupScope::ComputePass]);
    }
}

pub(crate) fn require_graphics_queue(
    queue: RenderQueueClass,
    command: &str,
) -> Result<(), RhiError> {
    if queue == RenderQueueClass::Graphics {
        Ok(())
    } else {
        Err(RhiError::InvalidCommandQueue {
            queue,
            command: command.to_string(),
        })
    }
}

pub(crate) fn ensure_non_zero_draw_counts(
    draw_count: u32,
    instance_count: u32,
) -> Result<(), RhiError> {
    if draw_count == 0 || instance_count == 0 {
        Err(RhiError::InvalidRasterDraw {
            reason: "draw and instance counts must be greater than zero".to_string(),
        })
    } else {
        Ok(())
    }
}
