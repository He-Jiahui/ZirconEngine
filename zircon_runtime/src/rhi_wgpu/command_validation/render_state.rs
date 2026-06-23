use std::collections::BTreeMap;

use crate::rhi::{
    BindGroupDesc, BindGroupHandle, BufferDesc, BufferHandle, IndexFormat, PipelineDesc,
    PipelineHandle, PipelineKind, RhiError, VertexBufferLayoutDesc,
};

use super::super::device::WgpuRenderDeviceState;

#[derive(Clone, Copy, Debug)]
pub(super) struct IndexBufferBinding {
    pub(super) size_bytes: u64,
    pub(super) format: IndexFormat,
}

#[derive(Clone, Copy, Debug)]
struct VertexBufferBinding {
    size_bytes: u64,
}

#[derive(Default)]
pub(super) struct RecordedRenderState<'a> {
    pub(super) current_pipeline: Option<(PipelineHandle, PipelineKind, &'a PipelineDesc)>,
    bind_groups: BTreeMap<u32, BindGroupHandle>,
    vertex_buffers: BTreeMap<u32, VertexBufferBinding>,
    pub(super) index_buffer: Option<IndexBufferBinding>,
}

impl<'a> RecordedRenderState<'a> {
    pub(super) fn set_pipeline(&mut self, pipeline: PipelineHandle, desc: &'a PipelineDesc) {
        self.current_pipeline = Some((pipeline, desc.kind, desc));
    }

    pub(super) fn set_bind_group(&mut self, slot: u32, bind_group: BindGroupHandle) {
        self.bind_groups.insert(slot, bind_group);
    }

    pub(super) fn set_vertex_buffer(&mut self, slot: u32, size_bytes: u64) {
        self.vertex_buffers
            .insert(slot, VertexBufferBinding { size_bytes });
    }

    pub(super) fn require_raster_pipeline(&self) -> Result<&PipelineDesc, RhiError> {
        match self.current_pipeline {
            Some((_pipeline, PipelineKind::Raster, desc)) => Ok(desc),
            Some((pipeline, actual, _)) => Err(RhiError::InvalidPipelineUsage {
                pipeline: pipeline.raw(),
                required: PipelineKind::Raster,
                actual,
            }),
            None => Err(RhiError::InvalidRasterDraw {
                reason: "raster draw requires a bound raster pipeline".to_string(),
            }),
        }
    }

    pub(super) fn require_compute_pipeline(&self) -> Result<&PipelineDesc, RhiError> {
        match self.current_pipeline {
            Some((_pipeline, PipelineKind::Compute, desc)) => Ok(desc),
            Some((pipeline, actual, _)) => Err(RhiError::InvalidPipelineUsage {
                pipeline: pipeline.raw(),
                required: PipelineKind::Compute,
                actual,
            }),
            None => Err(RhiError::InvalidComputeDispatch {
                reason: "compute dispatch requires a bound compute pipeline".to_string(),
            }),
        }
    }

    pub(super) fn ensure_required_bind_groups(
        &self,
        state: &WgpuRenderDeviceState,
        pipeline: &PipelineDesc,
        command: &str,
    ) -> Result<(), RhiError> {
        let pipeline_layout = pipeline_layout_for_command(state, pipeline)?;
        for (slot, expected_layout) in pipeline_layout.bind_group_layouts.iter().enumerate() {
            let slot = slot as u32;
            let bind_group =
                self.bind_groups
                    .get(&slot)
                    .ok_or_else(|| RhiError::InvalidBindGroupUsage {
                        reason: format!("{command} requires bind group slot {slot} to be bound"),
                    })?;
            let bind_group_desc = state.bind_group_desc_ref(*bind_group)?;
            if bind_group_desc.layout != *expected_layout {
                return Err(RhiError::InvalidBindGroupUsage {
                    reason: format!(
                        "bind group slot {slot} layout `{}` does not match pipeline layout `{}`",
                        bind_group_desc.layout.raw(),
                        expected_layout.raw()
                    ),
                });
            }
        }
        Ok(())
    }

    pub(super) fn ensure_required_vertex_buffers(
        &self,
        pipeline: &PipelineDesc,
    ) -> Result<(), RhiError> {
        let Some(raster_state) = pipeline.raster_state.as_ref() else {
            return Err(RhiError::InvalidRasterDraw {
                reason: "bound raster pipeline has no raster state".to_string(),
            });
        };
        for slot in 0..raster_state.vertex_input.buffers.len() as u32 {
            if !self.vertex_buffers.contains_key(&slot) {
                return Err(RhiError::InvalidRasterDraw {
                    reason: format!("draw requires vertex buffer slot {slot} to be bound"),
                });
            }
        }
        Ok(())
    }

    pub(super) fn validate_vertex_ranges(
        &self,
        pipeline: &PipelineDesc,
        vertex_start: u32,
        vertex_count: u32,
        instance_start: u32,
        instance_count: u32,
        validate_vertex_buffers: bool,
    ) -> Result<(), RhiError> {
        if pipeline_vertex_input_is_empty(pipeline) {
            if validate_vertex_buffers && vertex_start != 0 {
                return Err(RhiError::InvalidRasterDraw {
                    reason: "generated-vertex draws must start at vertex 0".to_string(),
                });
            }
            return Ok(());
        }

        let raster_state =
            pipeline
                .raster_state
                .as_ref()
                .ok_or_else(|| RhiError::InvalidRasterDraw {
                    reason: "bound raster pipeline has no raster state".to_string(),
                })?;
        for (slot, layout) in raster_state.vertex_input.buffers.iter().enumerate() {
            let slot = slot as u32;
            let binding =
                self.vertex_buffers
                    .get(&slot)
                    .ok_or_else(|| RhiError::InvalidRasterDraw {
                        reason: format!("draw requires vertex buffer slot {slot} to be bound"),
                    })?;
            let (start, count, label) = match layout.step_mode {
                crate::rhi::VertexStepMode::Vertex if validate_vertex_buffers => {
                    (vertex_start, vertex_count, "vertex")
                }
                crate::rhi::VertexStepMode::Instance => {
                    (instance_start, instance_count, "instance")
                }
                crate::rhi::VertexStepMode::Vertex => continue,
            };
            validate_strided_binding_range(slot, label, binding.size_bytes, layout, start, count)?;
        }
        Ok(())
    }
}

pub(super) fn validate_bind_group_slot(
    state: &WgpuRenderDeviceState,
    pipeline: &PipelineDesc,
    slot: u32,
    bind_group: BindGroupHandle,
    bind_group_desc: &BindGroupDesc,
) -> Result<(), RhiError> {
    let pipeline_layout = pipeline_layout_for_command(state, pipeline)?;
    let Some(expected_layout) = pipeline_layout.bind_group_layouts.get(slot as usize) else {
        return Err(RhiError::InvalidBindGroupUsage {
            reason: format!("bind group slot {slot} is not declared by the active pipeline layout"),
        });
    };
    if bind_group_desc.layout != *expected_layout {
        return Err(RhiError::InvalidBindGroupUsage {
            reason: format!(
                "bind group `{}` layout `{}` does not match pipeline layout slot {slot} `{}`",
                bind_group.raw(),
                bind_group_desc.layout.raw(),
                expected_layout.raw()
            ),
        });
    }
    Ok(())
}

fn pipeline_layout_for_command<'a>(
    state: &'a WgpuRenderDeviceState,
    pipeline: &PipelineDesc,
) -> Result<&'a crate::rhi::PipelineLayoutDesc, RhiError> {
    let layout = pipeline
        .layout
        .ok_or_else(|| RhiError::InvalidBindGroupUsage {
            reason: "bound pipeline has no pipeline layout".to_string(),
        })?;
    state.pipeline_layout_desc_ref(layout)
}

pub(super) trait CommandResourceLookup {
    fn buffer_desc(&self, handle: BufferHandle) -> Result<&BufferDesc, RhiError>;
}

impl CommandResourceLookup for WgpuRenderDeviceState {
    fn buffer_desc(&self, handle: BufferHandle) -> Result<&BufferDesc, RhiError> {
        self.buffers
            .get(&handle)
            .map(|buffer| &buffer.desc)
            .ok_or(RhiError::UnknownBuffer(handle.raw()))
    }
}

pub(super) fn ensure_binding_range(
    buffer: BufferHandle,
    desc: &BufferDesc,
    offset: u64,
    size: u64,
) -> Result<(), RhiError> {
    if size == 0 || offset.saturating_add(size) > desc.size_bytes {
        Err(RhiError::BufferBindingOutOfRange {
            buffer: buffer.raw(),
            offset,
            size,
        })
    } else {
        Ok(())
    }
}

fn pipeline_vertex_input_is_empty(pipeline: &PipelineDesc) -> bool {
    pipeline
        .raster_state
        .as_ref()
        .map(|raster_state| raster_state.vertex_input.buffers.is_empty())
        .unwrap_or(true)
}

pub(super) fn validate_index_range(
    index_buffer: IndexBufferBinding,
    index_start: u32,
    index_count: u32,
) -> Result<(), RhiError> {
    let first_index_byte = u64::from(index_start).saturating_mul(index_buffer.format.size_bytes());
    let index_bytes = u64::from(index_count).saturating_mul(index_buffer.format.size_bytes());
    if first_index_byte.saturating_add(index_bytes) > index_buffer.size_bytes {
        Err(RhiError::InvalidRasterDraw {
            reason: "indexed draw range exceeds the bound index buffer".to_string(),
        })
    } else {
        Ok(())
    }
}

fn validate_strided_binding_range(
    slot: u32,
    label: &str,
    size_bytes: u64,
    layout: &VertexBufferLayoutDesc,
    start: u32,
    count: u32,
) -> Result<(), RhiError> {
    let Some(first_element_offset) = u64::from(start).checked_mul(layout.array_stride) else {
        return Err(RhiError::InvalidRasterDraw {
            reason: format!("{label} range overflows for vertex buffer slot {slot}"),
        });
    };
    let Some(tail_stride_bytes) =
        u64::from(count.saturating_sub(1)).checked_mul(layout.array_stride)
    else {
        return Err(RhiError::InvalidRasterDraw {
            reason: format!("{label} range overflows for vertex buffer slot {slot}"),
        });
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
        return Err(RhiError::InvalidRasterDraw {
            reason: format!("{label} range overflows for vertex buffer slot {slot}"),
        });
    };
    if required_bytes > size_bytes {
        Err(RhiError::InvalidRasterDraw {
            reason: format!("{label} draw range exceeds vertex buffer slot {slot}"),
        })
    } else {
        Ok(())
    }
}
