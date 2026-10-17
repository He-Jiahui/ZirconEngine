//! Frame-local adapter from the existing GPU visibility/authority ABI to indexed draws.
//! Author runs describe admitted topology, never visibility. Main-pass integration must
//! supply topology from the same immutable admission as the bound mesh and a real
//! GPUScene instance span; this module does not infer either from cluster ordinals.
use std::{num::NonZeroU64, ops::Range};

use zircon_runtime::{
    core::framework::render::RenderVirtualGeometryExecutionIdentity,
    graphics::{RenderPassGpuRecordingContext, RenderPassGpuResourceFactory},
};

const IDENTITY_WORDS: usize = 14;
const RUN_WORDS: usize = 24;
const SIDECAR_WORDS: usize = 32;
const MAX_RUNS: usize = 65_536;

/// The binding range is explicit so a pooled allocation's unused tail is not a source.
#[derive(Clone, Copy)]
pub(super) struct GpuBufferSlice<'a> {
    buffer: &'a wgpu::Buffer,
    offset: u64,
    size: NonZeroU64,
}

impl<'a> GpuBufferSlice<'a> {
    pub(super) fn new(buffer: &'a wgpu::Buffer, offset: u64, size: u64) -> Result<Self, String> {
        let size = NonZeroU64::new(size).ok_or("indexed adapter binding is empty")?;
        if offset % 256 != 0 || size.get() % 4 != 0 {
            return Err("indexed adapter binding alignment is invalid".into());
        }
        if offset
            .checked_add(size.get())
            .is_none_or(|end| end > buffer.size())
        {
            return Err("indexed adapter binding exceeds its actual buffer".into());
        }
        if !buffer.usage().contains(wgpu::BufferUsages::STORAGE) {
            return Err("indexed adapter binding is not a storage buffer".into());
        }
        Ok(Self {
            buffer,
            offset,
            size,
        })
    }

    fn entry(&self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: self.buffer,
                offset: self.offset,
                size: Some(self.size),
            }),
        }
    }

    fn require(&self, bytes: u64, usage: wgpu::BufferUsages) -> Result<(), String> {
        if self.size.get() < bytes || !self.buffer.usage().contains(usage) {
            return Err("indexed adapter buffer capacity or usage is insufficient".into());
        }
        Ok(())
    }
}

pub(super) struct IndexedAuthorRunInput {
    pub source_row: u32,
    pub candidate_index: u32,
    pub cluster_id: u32,
    pub cluster_ordinal: u32,
    pub index_range: Range<u32>,
    pub source_index_count: u32,
    pub base_vertex: i32,
    pub gpu_scene_instance_span: Range<u32>,
    pub run_ordinal: u32,
    pub identity: RenderVirtualGeometryExecutionIdentity,
}

/// Validated author topology. Fields cannot be overwritten after construction.
/// Repeated source rows and disjoint ranges remain separate commands in input order.
pub(super) struct IndexedAuthorRun {
    input: IndexedAuthorRunInput,
}

impl IndexedAuthorRun {
    pub(super) fn try_new(input: IndexedAuthorRunInput) -> Result<Self, String> {
        if input.index_range.start >= input.index_range.end
            || input.index_range.start % 3 != 0
            || (input.index_range.end - input.index_range.start) % 3 != 0
            || input.index_range.end > input.source_index_count
        {
            return Err("indexed author run is outside admitted triangle topology".into());
        }
        if input
            .gpu_scene_instance_span
            .end
            .checked_sub(input.gpu_scene_instance_span.start)
            != Some(1)
        {
            return Err("indexed author run requires one actual GPUScene instance".into());
        }
        let identity = input.identity;
        let cluster_end = identity
            .cluster_start_ordinal
            .checked_add(identity.cluster_span_count);
        // Canonical source-row construction already rejects instance-less
        // rows when the actual instance table is nonempty. Keep that legacy
        // identity here; GPUScene's draw instance is a separate allocation.
        let legal_legacy = identity.stable_instance_key
            == zircon_runtime::core::framework::render::render_mesh_stable_instance_key(
                identity.entity,
                0,
            );
        if (identity.instance_index.is_none() && !legal_legacy)
            || identity.submission_index.is_none()
            || identity.draw_ref_rank.is_none()
            || identity.cluster_span_count == 0
            || cluster_end.is_none_or(|end| input.cluster_ordinal >= end)
            || input.cluster_ordinal < identity.cluster_start_ordinal
        {
            return Err("indexed author run has incomplete execution identity".into());
        }
        Ok(Self { input })
    }
}

pub(super) struct IndexedAdapterInputs<'a> {
    /// Existing GPU-produced count prefix followed by fixed four-word source rows.
    pub source_args: GpuBufferSlice<'a>,
    /// Existing GPU-produced fourteen-word authority rows, without a prefix.
    pub source_authority: GpuBufferSlice<'a>,
    pub source_identities: &'a [RenderVirtualGeometryExecutionIdentity],
    pub runs: &'a [IndexedAuthorRun],
    pub output_capacity: u32,
}

pub(super) struct IndexedAdapterOutputs<'a> {
    /// Fixed five-word DrawIndexedIndirectArgs per run; unused slots are GPU-cleared.
    pub args: GpuBufferSlice<'a>,
    /// Raw fourteen-word GPU authority + full fourteen-word identity + four row tags.
    pub sidecar: GpuBufferSlice<'a>,
    /// Per-slot status: zero is valid/unused, nonzero rejects that command.
    pub status: GpuBufferSlice<'a>,
}

pub(super) fn identity_words(
    identity: RenderVirtualGeometryExecutionIdentity,
) -> [u32; IDENTITY_WORDS] {
    let present = u32::from(identity.instance_index.is_some())
        | (u32::from(identity.submission_index.is_some()) << 1)
        | (u32::from(identity.draw_ref_rank.is_some()) << 2)
        | (u32::from(identity.submission_slot.is_some()) << 3);
    [
        identity.original_index,
        identity.draw_ref_index,
        identity.stable_instance_key as u32,
        (identity.stable_instance_key >> 32) as u32,
        present,
        identity.instance_index.unwrap_or(u32::MAX),
        identity.entity as u32,
        (identity.entity >> 32) as u32,
        identity.page_id,
        identity.submission_index.unwrap_or(u32::MAX),
        identity.draw_ref_rank.unwrap_or(u32::MAX),
        identity.cluster_start_ordinal,
        identity.cluster_span_count,
        identity.submission_slot.unwrap_or(u32::MAX),
    ]
}

/// Encodes only into the caller's encoder; allocation, submission and lifetime remain
/// with the existing frame owner. A failed admission records no adapter commands.
pub(super) fn encode<C: RenderPassGpuRecordingContext + ?Sized>(
    recording: &mut C,
    inputs: IndexedAdapterInputs<'_>,
    outputs: IndexedAdapterOutputs<'_>,
) -> Result<(), String> {
    let sources = inputs.source_identities.len();
    let capacity = inputs.output_capacity as usize;
    if sources == 0
        || sources > super::MAX_RECORDS
        || capacity == 0
        || capacity > MAX_RUNS
        || inputs.runs.len() > capacity
    {
        return Err("indexed adapter source/run capacity is invalid".into());
    }
    // The caller binds exactly the active rows, not a pooled allocation tail.
    // This also rejects truncated/extra words that could change either source ABI.
    if inputs.source_args.size.get() != ((1 + sources * 4) * 4) as u64
        || inputs.source_authority.size.get() != (sources * 14 * 4) as u64
        || outputs.args.size.get() != (capacity * 5 * 4) as u64
        || outputs.sidecar.size.get() != (capacity * SIDECAR_WORDS * 4) as u64
        || outputs.status.size.get() != (capacity * 4) as u64
    {
        return Err("indexed adapter binding row stride/capacity does not match its ABI".into());
    }
    if inputs
        .runs
        .iter()
        .any(|run| run.input.source_row as usize >= sources)
    {
        return Err("indexed author run source row is out of bounds".into());
    }
    inputs
        .source_args
        .require(((1 + sources * 4) * 4) as u64, wgpu::BufferUsages::STORAGE)?;
    inputs
        .source_authority
        .require((sources * 14 * 4) as u64, wgpu::BufferUsages::STORAGE)?;
    outputs.args.require(
        (capacity * 5 * 4) as u64,
        wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC,
    )?;
    outputs.sidecar.require(
        (capacity * SIDECAR_WORDS * 4) as u64,
        wgpu::BufferUsages::COPY_SRC,
    )?;
    outputs
        .status
        .require((capacity * 4) as u64, wgpu::BufferUsages::COPY_SRC)?;
    let reads = [inputs.source_args, inputs.source_authority];
    let writes = [outputs.args, outputs.sidecar, outputs.status];
    for (index, write) in writes.iter().enumerate() {
        if reads.iter().any(|read| read.buffer == write.buffer)
            || writes[..index]
                .iter()
                .any(|other| other.buffer == write.buffer)
        {
            return Err("indexed adapter inputs and outputs must be independent buffers".into());
        }
    }
    let mut words =
        Vec::with_capacity(4 + sources * IDENTITY_WORDS + inputs.runs.len() * RUN_WORDS);
    words.extend_from_slice(&[
        inputs.runs.len() as u32,
        sources as u32,
        inputs.output_capacity,
        0,
    ]);
    for identity in inputs.source_identities {
        words.extend_from_slice(&identity_words(*identity));
    }
    for run in inputs.runs {
        let input = &run.input;
        words.extend_from_slice(&[
            input.source_row,
            input.candidate_index,
            input.cluster_id,
            input.cluster_ordinal,
            input.index_range.start,
            input.index_range.end - input.index_range.start,
            input.base_vertex as u32,
            input.gpu_scene_instance_span.start,
            input.run_ordinal,
            input.source_index_count,
        ]);
        words.extend_from_slice(&identity_words(input.identity));
    }
    let factory = recording.resource_factory();
    let author = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("virtual-geometry-indexed-author-runs"),
        contents: bytemuck::cast_slice(&words),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let pipeline = create_pipeline(factory);
    let layout = pipeline.get_bind_group_layout(0);
    let bind_group = factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("virtual-geometry-indexed-adapter"),
        layout: &layout,
        entries: &[
            inputs.source_args.entry(0),
            inputs.source_authority.entry(1),
            wgpu::BindGroupEntry {
                binding: 2,
                resource: author.as_entire_binding(),
            },
            outputs.args.entry(3),
            outputs.sidecar.entry(4),
            outputs.status.entry(5),
        ],
    });
    let mut pass = recording
        .command_encoder()
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("virtual-geometry-indexed-adapter"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(inputs.output_capacity.div_ceil(64), 1, 1);
    Ok(())
}

fn create_pipeline<F: RenderPassGpuResourceFactory + ?Sized>(factory: &F) -> wgpu::ComputePipeline {
    let entries: Vec<_> = (0..6)
        .map(|binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: binding < 3,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
        .collect();
    let layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("virtual-geometry-indexed-layout"),
        entries: &entries,
    });
    let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("virtual-geometry-indexed-layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("virtual-geometry-indexed-adapter"),
        source: wgpu::ShaderSource::Wgsl(include_str!("indexed_args.wgsl").into()),
    });
    factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("virtual-geometry-indexed-adapter"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

#[cfg(test)]
#[path = "indexed_tests.rs"]
mod tests;
