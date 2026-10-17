use std::{
    collections::HashMap,
    mem::size_of,
    sync::{Mutex, OnceLock},
};

use bytemuck::{Pod, Zeroable};
use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RenderPassDeviceEpoch, RenderPassGpuExecutionContext,
    RenderPassGpuResourceFactory,
};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

use super::gpu::PackedClusterRange;

#[path = "indexed.rs"]
pub(super) mod indexed;

const PASS_NAME: &str = "virtual-geometry-indirect-args";
const EXECUTOR_ID: &str = "virtual-geometry.indirect-args";
const PIPELINE_LABEL: &str = "zircon-virtual-geometry-indirect-args";
const VISIBLE_CANDIDATES: &str = "virtual-geometry-visible-candidate-indices";
const INDIRECT_ARGS: &str = "virtual-geometry-indirect-args";
const INDIRECT_AUTHORITY: &str = "virtual-geometry-indirect-authority";
const INDIRECT_SEGMENTS: &str = "virtual-geometry-indirect-segments";
const SEGMENT_WORD_COUNT: usize = 12;
const SEGMENT_HEADER_WORDS: usize = 4;
const MAX_RECORDS: usize = 4096;
const WORKGROUP_SIZE: u32 = 64;
const SOURCE_WORD_COUNT: usize = 19;
const AUTHORITY_WORD_COUNT: usize = 14;

#[cfg(test)]
#[derive(Default)]
pub(crate) struct IndirectReadbackCollector {
    pending: Mutex<HashMap<u64, PendingIndirectReadback>>,
}

#[cfg(test)]
struct PendingIndirectReadback {
    args: wgpu::Buffer,
    authority: wgpu::Buffer,
    wait_for_submission: Box<dyn FnOnce() -> Result<(), String> + Send>,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IndirectReadbackWords {
    pub args: Vec<u32>,
    pub authority: Vec<u32>,
}

#[cfg(test)]
impl IndirectReadbackCollector {
    pub(crate) fn read_words(&self, frame_id: u64) -> Result<IndirectReadbackWords, String> {
        let pending = self
            .pending
            .lock()
            .map_err(|_| "virtual geometry indirect readback collector is poisoned".to_string())?
            .remove(&frame_id)
            .ok_or_else(|| format!("no indirect GPU readback is pending for frame {frame_id}"))?;
        let args_slice = pending.args.slice(..);
        let authority_slice = pending.authority.slice(..);
        let (args_sender, args_receiver) = std::sync::mpsc::channel();
        let (authority_sender, authority_receiver) = std::sync::mpsc::channel();
        args_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = args_sender.send(result.map_err(|error| error.to_string()));
        });
        authority_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = authority_sender.send(result.map_err(|error| error.to_string()));
        });
        (pending.wait_for_submission)()?;
        let timeout = std::time::Duration::from_secs(30);
        args_receiver
            .recv_timeout(timeout)
            .map_err(|error| format!("indirect args map callback did not finish: {error}"))??;
        authority_receiver.recv_timeout(timeout).map_err(|error| {
            format!("indirect authority map callback did not finish: {error}")
        })??;
        let args = bytemuck::try_cast_slice::<u8, u32>(&args_slice.get_mapped_range())
            .map_err(|error| format!("indirect args readback is not u32-aligned: {error:?}"))?
            .to_vec();
        let authority = bytemuck::try_cast_slice::<u8, u32>(&authority_slice.get_mapped_range())
            .map_err(|error| format!("indirect authority readback is not u32-aligned: {error:?}"))?
            .to_vec();
        drop(args_slice.get_mapped_range());
        drop(authority_slice.get_mapped_range());
        pending.args.unmap();
        pending.authority.unmap();
        Ok(IndirectReadbackWords { args, authority })
    }
}

static PIPELINE: OnceLock<Mutex<Option<(RenderPassDeviceEpoch, wgpu::ComputePipeline)>>> =
    OnceLock::new();

/// The fixed-slot source layout is `[cluster, candidate, first_vertex, vertex_count,
/// visibility_token, authority[14]]`. Slot `i` always writes args at byte offset `4 + i * 16`
/// and authority at `i * 14 * 4`; visibility only zeros the draw's vertex/instance counts.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SourceRecord {
    cluster_id: u32,
    candidate_index: u32,
    first_vertex: u32,
    vertex_count: u32,
    first_instance: u32,
    authority: [u32; AUTHORITY_WORD_COUNT],
}

/// Frozen author rows, not CPU visibility or decoded GPU results. Header:
/// [row_count, resident_fixed_rows, paired_prepare_present, request_count].
/// Rows: [instance/MAX, entity_lo, entity_hi, start, span, total, page,
///        execution_submission_slot/MAX, state, lineage, lod, author_frontier].
fn author_segment_words(
    gpu: &RenderPassGpuExecutionContext<'_>,
    resident_count: usize,
) -> Result<Vec<u32>, String> {
    use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState as State;
    let inputs = gpu.virtual_geometry_execution_inputs();
    let data = gpu.virtual_geometry_execution_data();
    let mut rows = Vec::<[u32; SEGMENT_WORD_COUNT]>::new();
    let state_word = |state| match state {
        State::Resident => 0,
        State::PendingUpload => 1,
        State::Missing => 2,
    };
    let mut requests = Vec::<(u32, u32)>::new();
    if let Some(prepare) = &inputs.prepare {
        for request in &prepare.pending_page_requests {
            requests.push((request.page_id, request.frontier_rank));
        }
        for segment in &prepare.cluster_draw_segments {
            let mut instance = None;
            let mut execution_slot = None;
            if let Some(data) = data {
                let execution_segments = data
                    .prepared_cpu_frame
                    .as_ref()
                    .and_then(|prepared| prepared.paired_execution_segments.as_deref())
                    .unwrap_or(&data.execution_segments);
                for candidate in execution_segments {
                    if candidate.entity == segment.entity
                        && candidate.page_id == segment.page_id
                        && candidate.cluster_start_ordinal == segment.cluster_ordinal
                        && candidate.cluster_span_count == segment.cluster_span_count
                    {
                        if instance.is_some_and(|previous| previous != candidate.instance_index) {
                            return Err(
                                "author segment has ambiguous actual instance provenance".into()
                            );
                        }
                        if execution_slot
                            .is_some_and(|previous| previous != candidate.submission_slot)
                        {
                            return Err(
                                "author segment has ambiguous actual submission slots".into()
                            );
                        }
                        execution_slot = Some(candidate.submission_slot);
                        instance = Some(candidate.instance_index);
                    }
                }
                if instance.is_none() && !data.instances.is_empty() {
                    return Err("author segment has no actual instance provenance".into());
                }
            }
            let mut resident_slot = segment.resident_slot;
            for cluster in &prepare.visible_clusters {
                if cluster.entity == segment.entity
                    && cluster.cluster_id == segment.cluster_id
                    && cluster.page_id == segment.page_id
                {
                    if resident_slot.is_some()
                        && cluster.resident_slot.is_some()
                        && resident_slot != cluster.resident_slot
                    {
                        return Err("author resident slot facts conflict".into());
                    }
                    resident_slot = resident_slot.or(cluster.resident_slot);
                }
            }
            let mut request_slot = None;
            for request in &prepare.pending_page_requests {
                if request.page_id != segment.page_id {
                    continue;
                }
                let mut assigned = request.assigned_slot;
                if assigned.is_none() {
                    if let Some(recycled) = request.recycled_page_id {
                        for page in prepare
                            .resident_pages
                            .iter()
                            .chain(&prepare.evictable_pages)
                        {
                            if page.page_id != recycled {
                                continue;
                            }
                            if assigned.is_some_and(|slot| slot != page.slot) {
                                return Err(
                                    "recycled pending page has ambiguous physical slots".into()
                                );
                            }
                            assigned = Some(page.slot);
                        }
                    }
                }
                if request_slot.is_some_and(|previous| previous != assigned) {
                    return Err("pending request has ambiguous submission slot facts".into());
                }
                request_slot = Some(assigned);
            }
            let declared_slot = resident_slot.or(request_slot.flatten());
            if execution_slot.is_some_and(|slot| {
                slot.is_some() && declared_slot.is_some() && slot != declared_slot
            }) {
                return Err(
                    "actual execution submission slot conflicts with prepare ownership".into(),
                );
            }
            let submission_slot = execution_slot.flatten().or(declared_slot);
            rows.push([
                instance.flatten().unwrap_or(u32::MAX),
                segment.entity as u32,
                (segment.entity >> 32) as u32,
                segment.cluster_ordinal,
                segment.cluster_span_count,
                segment.cluster_count,
                segment.page_id,
                submission_slot.unwrap_or(u32::MAX),
                state_word(segment.state),
                segment.lineage_depth,
                u32::from(segment.lod_level),
                0,
            ]);
        }
    } else if let Some(data) = data {
        if let Some(prepared) = data.prepared_cpu_frame.as_ref() {
            requests.extend(
                prepared
                    .author_pending_page_requests
                    .iter()
                    .map(|&(page, rank, _, _)| (page, rank)),
            );
        } else if data
            .execution_segments
            .iter()
            .any(|s| s.state == State::PendingUpload)
        {
            return Err("pending author segments have no actual prepare request owner".into());
        }
        for segment in &data.execution_segments {
            rows.push([
                segment.instance_index.unwrap_or(u32::MAX),
                segment.entity as u32,
                (segment.entity >> 32) as u32,
                segment.cluster_start_ordinal,
                segment.cluster_span_count,
                segment.cluster_total_count,
                segment.page_id,
                segment.submission_slot.unwrap_or(u32::MAX),
                state_word(segment.state),
                segment.lineage_depth,
                u32::from(segment.lod_level),
                segment.frontier_rank,
            ]);
        }
    }
    if rows.len() > MAX_RECORDS || requests.len() > MAX_RECORDS {
        return Err("author segments or pending request table exceed graph capacity".into());
    }
    for (index, &(page, rank)) in requests.iter().enumerate() {
        if requests[..index]
            .iter()
            .any(|&(other_page, other_rank)| other_page == page && other_rank != rank)
        {
            return Err("same page has ambiguous pending frontier rank".into());
        }
    }
    let mut pending = 0usize;
    for row in &rows {
        if row[4] == 0 || row[3].checked_add(row[4]).is_none_or(|end| end > row[5]) {
            return Err("author segment has an invalid cluster span".into());
        }
        if row[8] == 1 {
            pending = pending
                .checked_add(1)
                .ok_or("pending reservation count overflow")?;
            if !requests.iter().any(|&(page, _)| page == row[6]) {
                return Err("pending author segment has no actual pending request".into());
            }
        }
    }
    if resident_count
        .checked_add(pending)
        .is_none_or(|count| count > MAX_RECORDS)
    {
        return Err("resident rows plus pending reservations exceed args4 capacity".into());
    }
    let mut words = Vec::with_capacity(4 + rows.len() * SEGMENT_WORD_COUNT + requests.len() * 2);
    words.extend_from_slice(&[
        u32::try_from(rows.len()).map_err(|_| "author count overflow")?,
        u32::try_from(resident_count).map_err(|_| "resident count overflow")?,
        u32::from(inputs.prepare.is_some()),
        u32::try_from(requests.len()).map_err(|_| "request count overflow")?,
    ]);
    for row in rows {
        words.extend_from_slice(&row);
    }
    for (page, rank) in requests {
        words.extend_from_slice(&[page, rank]);
    }
    Ok(words)
}

fn encode_author_segments(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    words: &[u32],
) -> Result<(), String> {
    let args = gpu.require_buffer_binding(INDIRECT_ARGS, RenderGraphResourceAccessKind::Write)?;
    let segments =
        gpu.require_buffer_binding(INDIRECT_SEGMENTS, RenderGraphResourceAccessKind::Write)?;
    let args_view = (args.buffer.clone(), args.offset, binding_size(&args));
    let segment_view = (
        segments.buffer.clone(),
        segments.offset,
        binding_size(&segments),
    );
    let expected = ((SEGMENT_HEADER_WORDS + MAX_RECORDS * SEGMENT_WORD_COUNT) * 4) as u64;
    if segment_view.2 != expected {
        return Err("author segment graph output has an incompatible ABI".into());
    }
    let native = gpu.native_context();
    let input = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("virtual-geometry-frozen-author-segments"),
        contents: bytemuck::cast_slice(words),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let entries = [
        storage_layout_entry(2, false),
        storage_layout_entry(4, false),
        storage_layout_entry(5, true),
    ];
    let layout = native.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("virtual-geometry-author-segments"),
        entries: &entries,
    });
    let pipeline_layout = native.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("virtual-geometry-author-segments"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let shader = native.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("virtual-geometry-author-segments"),
        source: wgpu::ShaderSource::Wgsl(include_str!("indirect_args.wgsl").into()),
    });
    let pipeline = native.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("virtual-geometry-author-segments"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_segments"),
        compilation_options: Default::default(),
        cache: None,
    });
    let group = native.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("virtual-geometry-author-segments"),
        layout: &layout,
        entries: &[
            buffer_entry(2, &args_view.0, args_view.1, args_view.2),
            buffer_entry(4, &segment_view.0, segment_view.1, segment_view.2),
            wgpu::BindGroupEntry {
                binding: 5,
                resource: input.as_entire_binding(),
            },
        ],
    });
    native
        .encoder
        .clear_buffer(&segment_view.0, segment_view.1, Some(segment_view.2));
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("virtual-geometry-author-segments"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
    drop(pass);
    drop(native);
    gpu.record_compute_dispatch(
        PASS_NAME,
        EXECUTOR_ID,
        "virtual-geometry-author-segments",
        [1, 1, 1],
        [1, 1, 1],
        vec![INDIRECT_SEGMENTS.into(), INDIRECT_ARGS.into()],
    );
    Ok(())
}

fn execute_source(gpu: &mut RenderPassGpuExecutionContext<'_>) -> Result<(), String> {
    let visible =
        gpu.require_buffer_binding(VISIBLE_CANDIDATES, RenderGraphResourceAccessKind::Read)?;
    let args = gpu.require_buffer_binding(INDIRECT_ARGS, RenderGraphResourceAccessKind::Write)?;
    let authority =
        gpu.require_buffer_binding(INDIRECT_AUTHORITY, RenderGraphResourceAccessKind::Write)?;
    let records = source_records(gpu)?;
    let author_segments = author_segment_words(gpu, records.len())?;
    if records.len() > MAX_RECORDS {
        return Err(format!(
            "virtual geometry indirect source has {} records; capacity is {MAX_RECORDS}",
            records.len()
        ));
    }
    let mut source_words = Vec::with_capacity(1 + records.len() * SOURCE_WORD_COUNT);
    source_words.push(u32::try_from(records.len()).map_err(|_| {
        "virtual geometry indirect source count exceeds the GPU u32 ABI".to_string()
    })?);
    for record in &records {
        source_words.extend_from_slice(bytemuck::cast_slice(std::slice::from_ref(record)));
    }
    if source_words.len() != 1 + records.len() * SOURCE_WORD_COUNT {
        return Err("virtual geometry indirect source record layout mismatch".to_string());
    }

    let visible_buffer = visible.buffer.clone();
    let visible_offset = visible.offset;
    let visible_size = binding_size(&visible);
    let args_buffer = args.buffer.clone();
    let args_offset = args.offset;
    let args_size = binding_size(&args);
    let authority_buffer = authority.buffer.clone();
    let authority_offset = authority.offset;
    let authority_size = binding_size(&authority);
    let required_args_bytes = size_of::<u32>() as u64 + (MAX_RECORDS * 4 * size_of::<u32>()) as u64;
    let required_authority_bytes = (MAX_RECORDS * AUTHORITY_WORD_COUNT * size_of::<u32>()) as u64;
    if args_size < required_args_bytes || authority_size < required_authority_bytes {
        return Err(
            "virtual geometry indirect graph outputs are below the declared capacity".into(),
        );
    }

    let device_epoch = gpu
        .device_epoch()
        .ok_or_else(|| "virtual geometry indirect pass requires a device epoch".to_string())?;
    let mut native = gpu.native_context();
    let factory = &native;
    let source = factory.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-indirect-source"),
        contents: bytemuck::cast_slice(&source_words),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let mut cache = PIPELINE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .map_err(|_| "virtual geometry indirect pipeline cache is poisoned".to_string())?;
    if cache
        .as_ref()
        .is_none_or(|(epoch, _)| *epoch != device_epoch)
    {
        *cache = Some((device_epoch, create_pipeline(factory)));
    }
    let pipeline = &cache.as_ref().expect("pipeline initialized").1;
    let bind_group = factory.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-virtual-geometry-indirect-bind-group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            buffer_entry(0, &visible_buffer, visible_offset, visible_size),
            wgpu::BindGroupEntry {
                binding: 1,
                resource: source.as_entire_binding(),
            },
            buffer_entry(2, &args_buffer, args_offset, args_size),
            buffer_entry(3, &authority_buffer, authority_offset, authority_size),
        ],
    });
    gpu.buffer_upload_recorder().write_buffer(
        &args_buffer,
        args_offset,
        bytemuck::bytes_of(&0_u32),
    );
    let groups = u32::try_from(records.len().div_ceil(WORKGROUP_SIZE as usize).max(1))
        .map_err(|_| "virtual geometry indirect dispatch count exceeds u32".to_string())?;
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-virtual-geometry-indirect-args"),
            timestamp_writes: None,
        });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(groups, 1, 1);
    drop(pass);
    drop(cache);
    drop(native);
    gpu.record_compute_dispatch(
        PASS_NAME,
        EXECUTOR_ID,
        PIPELINE_LABEL,
        [WORKGROUP_SIZE, 1, 1],
        [groups, 1, 1],
        vec![INDIRECT_ARGS.to_string(), INDIRECT_AUTHORITY.to_string()],
    );
    encode_author_segments(gpu, &author_segments)?;
    Ok(())
}

#[cfg(test)]
pub(super) fn record_readback(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    collector: &IndirectReadbackCollector,
    frame_id: u64,
) -> Result<(), String> {
    let args = gpu.require_buffer_binding(INDIRECT_ARGS, RenderGraphResourceAccessKind::Write)?;
    let authority =
        gpu.require_buffer_binding(INDIRECT_AUTHORITY, RenderGraphResourceAccessKind::Write)?;
    for (name, size) in [
        (INDIRECT_ARGS, binding_size(&args)),
        (INDIRECT_AUTHORITY, binding_size(&authority)),
    ] {
        if size == 0 {
            return Err(format!("{name} has no bytes to read back"));
        }
        usize::try_from(size).map_err(|_| format!("{name} readback exceeds usize"))?;
    }
    let args_buffer = args.buffer.clone();
    let args_offset = args.offset;
    let args_size = binding_size(&args);
    let authority_buffer = authority.buffer.clone();
    let authority_offset = authority.offset;
    let authority_size = binding_size(&authority);
    let mut native = gpu.native_context();
    let args_zeroes = vec![0_u8; usize::try_from(args_size).expect("args size was checked")];
    let authority_zeroes =
        vec![0_u8; usize::try_from(authority_size).expect("authority size was checked")];
    let args_staging = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-indirect-args-readback"),
        contents: &args_zeroes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    });
    let authority_staging = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-virtual-geometry-indirect-authority-readback"),
        contents: &authority_zeroes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    });
    native
        .encoder
        .copy_buffer_to_buffer(&args_buffer, args_offset, &args_staging, 0, args_size);
    native.encoder.copy_buffer_to_buffer(
        &authority_buffer,
        authority_offset,
        &authority_staging,
        0,
        authority_size,
    );
    let wait_for_submission = Box::new(native.readback_completion_waiter());
    drop(native);
    let mut pending = collector
        .pending
        .lock()
        .map_err(|_| "virtual geometry indirect readback collector is poisoned".to_string())?;
    if pending.contains_key(&frame_id) {
        return Err(format!(
            "virtual geometry indirect readback frame id {frame_id} was reused before consumption"
        ));
    }
    pending.insert(
        frame_id,
        PendingIndirectReadback {
            args: args_staging,
            authority: authority_staging,
            wait_for_submission,
        },
    );
    Ok(())
}

fn source_records(gpu: &RenderPassGpuExecutionContext<'_>) -> Result<Vec<SourceRecord>, String> {
    let (vertices, ranges) = super::gpu::cluster_vertices_with_ranges(gpu);
    let Some(data) = gpu.virtual_geometry_execution_data() else {
        if !ranges.is_empty() {
            return Err("packed source has no prepared execution data".into());
        }
        return Ok(Vec::new());
    };
    let rows = zircon_runtime::graphics::virtual_geometry_indexed_source_rows(data)?;
    if rows.len() != ranges.len() {
        return Err("canonical source rows and actual packed geometry differ".into());
    }
    for (row, range) in rows.iter().zip(&ranges) {
        let identity = row.identity();
        if range.authority_index != row.source_row()
            || range.candidate_index != row.candidate_index()
            || range.cluster_id != row.cluster_id()
            || range.cluster_ordinal != row.cluster_ordinal()
            || range.entity != identity.entity
            || range.instance_index != identity.instance_index.unwrap_or(u32::MAX)
            || range.page_id != identity.page_id
            || range.draw_ref_index != identity.draw_ref_index
            || range.original_index != identity.original_index
            || range.submission_index != identity.submission_index
            || range.draw_ref_rank != identity.draw_ref_rank
            || range.submission_slot != identity.submission_slot
        {
            return Err("packed source row lost its complete canonical identity".into());
        }
        range
            .vertices
            .end
            .checked_sub(range.vertices.start)
            .ok_or("inverted packed source range")?;
        let end = usize::try_from(range.vertices.end).map_err(|_| "packed range exceeds usize")?;
        if end > vertices.len() {
            return Err("packed source exceeds actual vertex stream".into());
        }
    }
    let identities = projected_source_identities(gpu, &rows)?;
    let mut records = Vec::with_capacity(rows.len());
    for ((row, range), identity) in rows.iter().zip(ranges).zip(identities) {
        let count = range
            .vertices
            .end
            .checked_sub(range.vertices.start)
            .ok_or("inverted packed source range")?;
        let submission = identity
            .submission_index
            .ok_or("source submission index missing")?;
        let rank = identity
            .draw_ref_rank
            .ok_or("source draw-ref rank missing")?;
        records.push(SourceRecord {
            cluster_id: row.cluster_id(),
            candidate_index: row.candidate_index(),
            first_vertex: range.vertices.start,
            vertex_count: count,
            first_instance: (submission.min(0xffff) << 16) | rank.min(0xffff),
            authority: [
                range.instance_index,
                identity.entity as u32,
                (identity.entity >> 32) as u32,
                identity.page_id,
                row.cluster_id(),
                row.cluster_ordinal(),
                identity.draw_ref_index,
                identity.original_index,
                submission,
                rank,
                identity.submission_slot.unwrap_or(u32::MAX),
                row.candidate_index(),
                row.lineage_depth(),
                row.frontier_rank(),
            ],
        });
    }
    Ok(records)
}

fn projected_source_identities(
    gpu: &RenderPassGpuExecutionContext<'_>,
    rows: &[zircon_runtime::graphics::VirtualGeometryIndexedSourceRow],
) -> Result<
    Vec<zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionIdentity>,
    String,
> {
    let mut identities = rows.iter().map(|row| row.identity()).collect::<Vec<_>>();
    let mut projected = vec![false; rows.len()];
    for (index, row) in rows.iter().enumerate() {
        if usize::try_from(row.source_row()).map_err(|_| "source row index overflow")? != index {
            return Err("canonical source rows are not in their actual fixed slots".into());
        }
    }
    if let Some(authors) = gpu.virtual_geometry_indexed_author_runs() {
        for author in authors {
            let source = author.source();
            let index =
                usize::try_from(source.source_row()).map_err(|_| "author source row overflow")?;
            let base = rows
                .get(index)
                .ok_or("actual author source row exceeds the canonical table")?;
            if source.candidate_index() != base.candidate_index()
                || source.cluster_id() != base.cluster_id()
                || source.cluster_ordinal() != base.cluster_ordinal()
                || source.page_index() != base.page_index()
                || source.range_index() != base.range_index()
                || source.author_array_index() != base.author_array_index()
                || source.normal_cluster_total() != base.normal_cluster_total()
                || source.lineage_depth() != base.lineage_depth()
                || source.frontier_rank() != base.frontier_rank()
            {
                return Err(
                    "actual author source lost its original candidate/payload provenance".into(),
                );
            }
            let actual = source.identity();
            let original = base.identity();
            let mut permitted = original;
            permitted.draw_ref_index = actual.draw_ref_index;
            permitted.draw_ref_rank = actual.draw_ref_rank;
            if permitted != actual {
                return Err(
                    "actual author changed identity beyond its checked draw reference".into(),
                );
            }
            if actual != original
                && (original.instance_index.is_some()
                    || original.submission_index.is_none()
                    || actual
                        .draw_ref_rank
                        .is_none_or(|rank| rank > u32::from(u16::MAX)))
            {
                return Err(
                    "changed draw reference is not a checked legacy author identity".into(),
                );
            }
            if projected[index] && identities[index] != actual {
                return Err(
                    "same source row has incompatible actual draw-reference identities".into(),
                );
            }
            identities[index] = actual;
            projected[index] = true;
        }
    }
    Ok(identities)
}

fn binding_size(binding: &wgpu::BufferBinding<'_>) -> u64 {
    binding
        .size
        .map(|n| n.get())
        .unwrap_or_else(|| binding.buffer.size().saturating_sub(binding.offset))
}

fn buffer_entry(
    binding: u32,
    buffer: &wgpu::Buffer,
    offset: u64,
    size: u64,
) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset,
            size: std::num::NonZeroU64::new(size),
        }),
    }
}

fn create_pipeline(factory: &impl RenderPassGpuResourceFactory) -> wgpu::ComputePipeline {
    let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-virtual-geometry-indirect-shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("indirect_args.wgsl").into()),
    });
    let bind_group_layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-virtual-geometry-indirect-bind-group-layout"),
        entries: &[
            storage_layout_entry(0, true),
            storage_layout_entry(1, true),
            storage_layout_entry(2, false),
            storage_layout_entry(3, false),
        ],
    });
    let layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-virtual-geometry-indirect-pipeline-layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(PIPELINE_LABEL),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn storage_layout_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(super) fn execute(gpu: &mut RenderPassGpuExecutionContext<'_>) -> Result<(), String> {
    execute_source(gpu)?;
    encode_indexed_main(gpu)
}
fn encode_indexed_main(gpu: &mut RenderPassGpuExecutionContext<'_>) -> Result<(), String> {
    let Some(authors) = gpu
        .virtual_geometry_indexed_author_runs()
        .map(<[_]>::to_vec)
    else {
        // Isolated legacy four/fourteen-word producer tests have no main indexed contract.
        return Ok(());
    };
    let rows = match gpu.virtual_geometry_execution_data() {
        Some(data) => zircon_runtime::graphics::virtual_geometry_indexed_source_rows(data)?,
        None => Vec::new(),
    };
    if authors.is_empty() && !rows.is_empty() {
        return Err("visible-source frame has no proven bound-mesh indexed admission".into());
    }
    let mut runs = Vec::with_capacity(authors.len());
    for (ordinal, author) in authors.iter().enumerate() {
        let source = author.source();
        let first = author.gpu_scene_instance();
        runs.push(indexed::IndexedAuthorRun::try_new(
            indexed::IndexedAuthorRunInput {
                source_row: source.source_row(),
                candidate_index: source.candidate_index(),
                cluster_id: source.cluster_id(),
                cluster_ordinal: source.cluster_ordinal(),
                index_range: author.index_range(),
                source_index_count: author.source_index_count(),
                base_vertex: 0,
                gpu_scene_instance_span: first
                    ..first
                        .checked_add(1)
                        .ok_or("GPUScene instance span overflow")?,
                run_ordinal: u32::try_from(ordinal)
                    .map_err(|_| "indexed run ordinal exceeds u32")?,
                identity: source.identity(),
            },
        )?);
    }
    let identities = projected_source_identities(gpu, &rows)?;
    let read = RenderGraphResourceAccessKind::Read;
    let write = RenderGraphResourceAccessKind::Write;
    let specs = [
        (INDIRECT_ARGS, write),
        (INDIRECT_AUTHORITY, write),
        ("virtual-geometry-indexed-args", write),
        ("virtual-geometry-indexed-sidecar", write),
        ("virtual-geometry-indexed-status", write),
        ("virtual-geometry-admitted-run-slots", write),
    ];
    let mut bindings = Vec::new();
    for (name, access) in specs {
        let binding = gpu.require_buffer_binding(name, access)?;
        bindings.push((
            binding.buffer.clone(),
            binding.offset,
            binding_size(&binding),
        ));
    }
    let _ = read;
    let mut native = gpu.native_context();
    let (journal, offset, size) = &bindings[5];
    if *size != (65536_u64 + 2) * 4 {
        return Err("indexed journal has a different physical ABI".into());
    }
    native.encoder.clear_buffer(journal, *offset, Some(*size));
    if rows.is_empty() {
        for (buffer, offset, size) in &bindings[2..5] {
            native.encoder.clear_buffer(buffer, *offset, Some(*size));
        }
        return Ok(());
    }
    let slice = |index: usize, size: u64| {
        indexed::GpuBufferSlice::new(&bindings[index].0, bindings[index].1, size)
    };
    indexed::encode(
        &mut native,
        indexed::IndexedAdapterInputs {
            source_args: slice(0, ((1 + rows.len() * 4) * 4) as u64)?,
            source_authority: slice(1, (rows.len() * 14 * 4) as u64)?,
            source_identities: &identities,
            runs: &runs,
            output_capacity: 65536,
        },
        indexed::IndexedAdapterOutputs {
            args: slice(2, 65536 * 5 * 4)?,
            sidecar: slice(3, 65536 * 32 * 4)?,
            status: slice(4, 65536 * 4)?,
        },
    )
}

const TERMINAL_SHADER: &str = r###"
struct Words { values: array<u32> }
@group(0) @binding(0) var<storage,read> indexed: Words;
@group(0) @binding(1) var<storage,read> sidecar: Words;
@group(0) @binding(2) var<storage,read> status: Words;
@group(0) @binding(3) var<storage,read> journal: Words;
@group(0) @binding(4) var<storage,read> source: Words;
@group(0) @binding(5) var<storage,read_write> execution_args: Words;
@group(0) @binding(6) var<storage,read_write> execution_authority: Words;
@group(0) @binding(7) var<storage,read_write> execution_submission: Words;
@compute @workgroup_size(1)
fn cs_main() {
    execution_args.values[0]=0u;
    execution_submission.values[0]=0u;
    let capacity=arrayLength(&indexed.values)/5u;
    let source_capacity=(arrayLength(&source.values)-1u)/4u;
    let count=journal.values[0];
    if journal.values[1]!=0u || count>capacity || count>arrayLength(&journal.values)-2u {return;}
    var output=0u;
    for(var index=0u;index<count;index++) {
        let slot=journal.values[2u+index];
        if slot>=capacity {return;}
        let args_offset=slot*5u;
        let sidecar_offset=slot*32u;
        if status.values[slot]!=0u {return;}
        if indexed.values[args_offset]==0u || indexed.values[args_offset+1u]==0u {continue;}
        let row=sidecar.values[sidecar_offset+28u];
        if row>=source_capacity || indexed.values[args_offset+1u]!=1u ||
            sidecar.values[sidecar_offset+31u]!=1u {return;}
        let source_offset=1u+row*4u;
        let old_token=source.values[source_offset+3u];
        let submission=sidecar.values[sidecar_offset+8u];
        let rank=sidecar.values[sidecar_offset+9u];
        if source.values[source_offset]==0u || source.values[source_offset+1u]!=1u ||
            old_token!=((min(submission,65535u)<<16u)|min(rank,65535u)) {return;}
        // Keep one compact output row for EVERY actual admitted run. Repeated
        // candidates and noncontiguous geometry runs are intentionally retained.
        for(var word=0u;word<4u;word++) {
            execution_args.values[1u+output*4u+word]=source.values[source_offset+word];
        }
        for(var word=0u;word<14u;word++) {
            execution_authority.values[output*14u+word]=sidecar.values[sidecar_offset+word];
            execution_submission.values[1u+output*17u+1u+word]=sidecar.values[sidecar_offset+14u+word];
        }
        execution_submission.values[1u+output*17u]=row;
        execution_submission.values[1u+output*17u+15u]=old_token;
        execution_submission.values[1u+output*17u+16u]=indexed.values[args_offset+1u];
        output++;
    }
    // Publish only after validating the entire admitted stream: any malformed
    // row leaves both headers at zero. The completion decoder rejects errors.
    execution_args.values[0]=output;
    execution_submission.values[0]=output;
}
"###;

pub(super) fn complete_main(gpu: &mut RenderPassGpuExecutionContext<'_>) -> Result<(), String> {
    let kinds = [
        RenderGraphResourceAccessKind::Read,
        RenderGraphResourceAccessKind::Read,
        RenderGraphResourceAccessKind::Read,
        RenderGraphResourceAccessKind::Read,
        RenderGraphResourceAccessKind::Read,
        RenderGraphResourceAccessKind::Write,
        RenderGraphResourceAccessKind::Write,
        RenderGraphResourceAccessKind::Write,
    ];
    let names = [
        "virtual-geometry-indexed-args",
        "virtual-geometry-indexed-sidecar",
        "virtual-geometry-indexed-status",
        "virtual-geometry-admitted-run-slots",
        "virtual-geometry-indirect-args",
        "virtual-geometry-execution-args",
        "virtual-geometry-execution-authority",
        "virtual-geometry-execution-submission",
    ];
    let expected = [
        65536 * 5 * 4,
        65536 * 32 * 4,
        65536 * 4,
        (65536 + 2) * 4,
        (1 + 4096 * 4) * 4,
        (1 + 65536 * 4) * 4,
        65536 * 14 * 4,
        (1 + 65536 * 17) * 4,
    ];
    let mut buffers = Vec::new();
    for ((name, kind), size) in names.into_iter().zip(kinds).zip(expected) {
        let binding = gpu.require_buffer_binding(name, kind)?;
        let bytes = binding
            .size
            .map(|size| size.get())
            .unwrap_or(binding.buffer.size() - binding.offset);
        if bytes != size
            || !binding
                .buffer
                .usage()
                .contains(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC)
        {
            return Err(format!(
                "terminal GPU source {name} has an invalid physical contract"
            ));
        }
        if buffers
            .iter()
            .any(|(buffer, _, _, _)| buffer == binding.buffer)
        {
            return Err("terminal GPU sources must be independent physical allocations".into());
        }
        buffers.push((binding.buffer.clone(), binding.offset, binding.size, size));
    }
    {
        let native = gpu.native_context();
        let factory = &native;
        let entries = (0..8)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: binding < 5,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<_>>();
        let layout = factory.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("virtual-geometry-terminal-layout"),
            entries: &entries,
        });
        let pipeline_layout = factory.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("virtual-geometry-terminal-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = factory.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("virtual-geometry-terminal"),
            source: wgpu::ShaderSource::Wgsl(TERMINAL_SHADER.into()),
        });
        let pipeline = factory.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("virtual-geometry-terminal"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let bindings = buffers
            .iter()
            .enumerate()
            .map(|(index, (buffer, offset, size, _))| wgpu::BindGroupEntry {
                binding: index as u32,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: *offset,
                    size: *size,
                }),
            })
            .collect::<Vec<_>>();
        let group = factory.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("virtual-geometry-terminal"),
            layout: &layout,
            entries: &bindings,
        });
        for (buffer, offset, _, bytes) in &buffers[5..] {
            native.encoder.clear_buffer(buffer, *offset, Some(*bytes));
        }
        let mut pass = native
            .encoder
            .begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("virtual-geometry-terminal"),
                timestamp_writes: None,
            });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    let metadata = encode_terminal_metadata(gpu, &buffers)?;
    let selected = encode_terminal_selected(gpu)?;
    record_terminal_completion(gpu, &buffers, &metadata, &selected)?;
    Ok(())
}

const METADATA_SHADER: &str = r###"
struct Words { values:array<u32> }
@group(0) @binding(0) var<storage,read> authority:Words;
@group(0) @binding(1) var<storage,read> submission:Words;
@group(0) @binding(2) var<storage,read> authors:Words;
@group(0) @binding(3) var<storage,read_write> output:Words;
@compute @workgroup_size(1)
fn cs_main(){
    output.values[0]=0u;
    let count=submission.values[0];
    let capacity=(arrayLength(&output.values)-1u)/18u;
    if count>capacity || count>(arrayLength(&submission.values)-1u)/17u {return;}
    for(var row=0u;row<count;row++){
        let input=1u+row*17u;
        let source=submission.values[input];
        if source>=authors.values[0] || source>=(arrayLength(&authors.values)-1u)/32u {return;}
        let author=1u+source*32u;
        for(var word=0u;word<14u;word++){
            if submission.values[input+1u+word]!=authors.values[author+word] {return;}
        }
        let record=author+14u;
        let actual=row*14u;
        // Validate every identity/ordinal field represented by raw authority.
        if authority.values[actual]!=authors.values[record] ||
            authority.values[actual+1u]!=authors.values[record+1u] ||
            authority.values[actual+2u]!=authors.values[record+2u] ||
            authority.values[actual+3u]!=authors.values[record+5u] ||
            authority.values[actual+4u]!=authors.values[record+3u] ||
            authority.values[actual+5u]!=authors.values[record+4u] ||
            authority.values[actual+6u]!=authors.values[author+1u] ||
            authority.values[actual+7u]!=authors.values[author] ||
            authority.values[actual+8u]!=authors.values[record+7u] ||
            authority.values[actual+9u]!=authors.values[author+10u] ||
            authority.values[actual+10u]!=authors.values[record+16u] ||
            authority.values[actual+12u]!=authors.values[record+13u] ||
            authority.values[actual+13u]!=authors.values[record+14u] ||
            authors.values[record+10u]!=authors.values[author+11u] ||
            authors.values[record+11u]!=authors.values[author+12u] ||
            authors.values[record+17u]!=0u || submission.values[input+16u]!=1u {return;}
        let token=(min(authority.values[actual+8u],65535u)<<16u)|min(authority.values[actual+9u],65535u);
        if submission.values[input+15u]!=token {return;}
        for(var word=0u;word<18u;word++){
            output.values[1u+row*18u+word]=authors.values[record+word];
        }
    }
    output.values[0]=count;
}
"###;

fn terminal_author_metadata(gpu: &RenderPassGpuExecutionContext<'_>) -> Result<Vec<u32>, String> {
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryClusterSelectionInput, RenderVirtualGeometryExecutionState,
        RenderVirtualGeometryHardwareRasterizationRecord,
    };
    let data = gpu
        .virtual_geometry_execution_data()
        .ok_or("terminal author data is absent")?;
    let rows = zircon_runtime::graphics::virtual_geometry_indexed_source_rows(data)?;
    let identities = projected_source_identities(gpu, &rows)?;
    let prepared = data
        .prepared_cpu_frame
        .as_ref()
        .ok_or("terminal residency author frame is absent")?;
    // Duplicate page facts are retained. Conflicting facts are rejected, never
    // resolved by first-match, insertion overwrite or a renumbered slot.
    let page_slot = |page: u32| -> Result<Option<u32>, String> {
        let mut result = None;
        for &(candidate, slot) in &prepared.author_resident_page_slots {
            if candidate != page {
                continue;
            }
            if result.is_some_and(|previous| previous != slot) {
                return Err("same-frame resident page has conflicting physical slots".into());
            }
            result = Some(slot);
        }
        Ok(result)
    };
    let mut words = Vec::with_capacity(1 + rows.len() * 32);
    words.push(u32::try_from(rows.len()).map_err(|_| "terminal author count overflow")?);
    for (row, identity) in rows.iter().zip(identities) {
        let (raw_start, raw_end) = if let Some(instance_index) = identity.instance_index {
            let instance = data
                .instances
                .get(usize::try_from(instance_index).map_err(|_| "author instance overflow")?)
                .ok_or("author instance lies outside the original instance table")?;
            if instance.entity != identity.entity
                || instance.stable_instance_key_or_legacy() != identity.stable_instance_key
            {
                return Err("author instance disagrees with actual admitted run identity".into());
            }
            (
                instance.cluster_offset,
                instance
                    .cluster_offset
                    .checked_add(instance.cluster_count)
                    .ok_or("author raw span overflow")?,
            )
        } else {
            if !data.instances.is_empty()
                || zircon_runtime::core::framework::render::render_mesh_stable_instance_key(
                    identity.entity,
                    0,
                ) != identity.stable_instance_key
            {
                return Err("author legacy source identity is absent".into());
            }
            (
                0,
                u32::try_from(data.author_cluster_catalog.len())
                    .map_err(|_| "author raw catalog overflow")?,
            )
        };
        let ordinal = row.cluster_ordinal();
        let total = row.normal_cluster_total();
        if ordinal >= total {
            return Err("author ordinal exceeds normal policy total".into());
        }
        let index = row.author_array_index();
        if index < raw_start || index >= raw_end {
            return Err("author raw index lies outside its instance".into());
        }
        let cluster = data
            .author_cluster_catalog
            .get(usize::try_from(index).map_err(|_| "author array index overflow")?)
            .ok_or("author cluster array range is absent")?;
        if cluster.entity != identity.entity
            || cluster.cluster_id != row.cluster_id()
            || cluster.page_id != identity.page_id
        {
            return Err(
                "actual source row disagrees with original cluster array provenance".into(),
            );
        }
        let start = identity.cluster_start_ordinal;
        let span = identity.cluster_span_count;
        if start.checked_add(span).ok_or("author span overflow")? > total
            || ordinal < start
            || ordinal
                >= start
                    .checked_add(span)
                    .ok_or("author ordinal span overflow")?
        {
            return Err("actual admitted run exceeds its immutable source topology".into());
        }
        let actual_slot = page_slot(cluster.page_id)?;
        let mut request = None;
        for fact in &prepared.author_pending_page_requests {
            if fact.0 != cluster.page_id {
                continue;
            }
            if request.is_some_and(|previous| previous != *fact) {
                return Err("same-frame pending page has conflicting request provenance".into());
            }
            request = Some(*fact);
        }
        let frontier = request.map(|fact| fact.1).unwrap_or(0);
        let author = match gpu.virtual_geometry_execution_inputs().selections.as_ref() {
            Some(explicit) => {
                // candidate_index is the original GPU upload row, not an entity,
                // cluster ID, decoded result position or an ordinal guess.
                let candidate = usize::try_from(row.candidate_index())
                    .map_err(|_| "author candidate offset overflow")?;
                explicit
                    .get(candidate)
                    .ok_or("paired author input does not contain the actual GPU candidate row")?
                    .clone()
            }
            None => RenderVirtualGeometryClusterSelectionInput {
                submission_index: identity
                    .submission_index
                    .ok_or("author submission identity is absent")?,
                instance_index: identity.instance_index,
                entity: cluster.entity,
                cluster_id: cluster.cluster_id,
                cluster_ordinal: ordinal,
                page_id: cluster.page_id,
                lod_level: cluster.lod_level,
                submission_page_id: identity.page_id,
                submission_lod_level: cluster.lod_level,
                entity_cluster_start_ordinal: usize::try_from(start)
                    .map_err(|_| "author start overflow")?,
                entity_cluster_span_count: usize::try_from(span)
                    .map_err(|_| "author span overflow")?,
                entity_cluster_total_count: usize::try_from(total)
                    .map_err(|_| "author total overflow")?,
                lineage_depth: row.lineage_depth(),
                frontier_rank: frontier,
                resident_slot: actual_slot,
                submission_slot: actual_slot,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        };
        if author.instance_index != identity.instance_index
            || author.entity != identity.entity
            || author.cluster_id != cluster.cluster_id
            || author.cluster_ordinal != ordinal
            || author.page_id != cluster.page_id
            || author.lod_level != cluster.lod_level
            || Some(author.submission_index) != identity.submission_index
            || author.submission_page_id != identity.page_id
            || author.submission_slot != identity.submission_slot
            || author.entity_cluster_start_ordinal
                != usize::try_from(start).map_err(|_| "author start overflow")?
            || author.entity_cluster_span_count
                != usize::try_from(span).map_err(|_| "author span overflow")?
            || author.entity_cluster_total_count
                != usize::try_from(total).map_err(|_| "author total overflow")?
            || author.lineage_depth != row.lineage_depth()
            || author.frontier_rank != row.frontier_rank()
            || author.resident_slot != actual_slot
            || actual_slot.is_none()
            || author.state != RenderVirtualGeometryExecutionState::Resident
        {
            return Err(
                "author attributes do not match the actual source row and same-frame residency"
                    .into(),
            );
        }
        let record = RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: author.instance_index,
            entity: author.entity,
            cluster_id: author.cluster_id,
            cluster_ordinal: author.cluster_ordinal,
            page_id: author.page_id,
            lod_level: author.lod_level,
            submission_index: author.submission_index,
            submission_page_id: author.submission_page_id,
            submission_lod_level: author.submission_lod_level,
            entity_cluster_start_ordinal: u32::try_from(author.entity_cluster_start_ordinal)
                .map_err(|_| "author start exceeds GPU ABI")?,
            entity_cluster_span_count: u32::try_from(author.entity_cluster_span_count)
                .map_err(|_| "author span exceeds GPU ABI")?,
            entity_cluster_total_count: u32::try_from(author.entity_cluster_total_count)
                .map_err(|_| "author total exceeds GPU ABI")?,
            lineage_depth: author.lineage_depth,
            frontier_rank: author.frontier_rank,
            resident_slot: author.resident_slot,
            submission_slot: author.submission_slot,
            state: author.state,
        };
        words.extend_from_slice(&indexed::identity_words(identity));
        words.extend_from_slice(&record.packed_words());
    }
    Ok(words)
}

fn encode_terminal_metadata(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    buffers: &[(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64)],
) -> Result<(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64), String> {
    let words = terminal_author_metadata(gpu)?;
    let binding = gpu.require_buffer_binding(
        "virtual-geometry-execution-hardware-metadata",
        RenderGraphResourceAccessKind::Write,
    )?;
    let size = binding
        .size
        .map(|size| size.get())
        .unwrap_or(binding.buffer.size() - binding.offset);
    if size != (1 + 65536 * 18) * 4 {
        return Err("terminal metadata output has an invalid physical range".into());
    }
    let output = (binding.buffer.clone(), binding.offset, binding.size, size);
    let native = gpu.native_context();
    let authors = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("virtual-geometry-hardware-author-metadata"),
        contents: bytemuck::cast_slice(&words),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let entries = (0..4)
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
        .collect::<Vec<_>>();
    let layout = native.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("virtual-geometry-hardware-metadata"),
        entries: &entries,
    });
    let pipeline_layout = native.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("virtual-geometry-hardware-metadata"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let shader = native.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("virtual-geometry-hardware-metadata"),
        source: wgpu::ShaderSource::Wgsl(METADATA_SHADER.into()),
    });
    let pipeline = native.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("virtual-geometry-hardware-metadata"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let view = |index: usize| {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &buffers[index].0,
            offset: buffers[index].1,
            size: buffers[index].2,
        })
    };
    let group = native.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("virtual-geometry-hardware-metadata"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: view(6),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: view(7),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: authors.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &output.0,
                    offset: output.1,
                    size: output.2,
                }),
            },
        ],
    });
    native
        .encoder
        .clear_buffer(&output.0, output.1, Some(output.3));
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("virtual-geometry-hardware-metadata"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
    drop(pass);
    Ok(output)
}

const SELECTED_SHADER: &str = r###"
struct Words { values:array<u32> }
@group(0) @binding(0) var<storage,read> indices:Words;
@group(0) @binding(1) var<storage,read> clusters:Words;
@group(0) @binding(2) var<storage,read> authors:Words;
@group(0) @binding(3) var<storage,read_write> selected:Words;
@compute @workgroup_size(1)
fn cs_main(){
    selected.values[0]=0u;
    let count=indices.values[0];
    if count!=clusters.values[0] || count>arrayLength(&indices.values)-1u ||
        count>arrayLength(&clusters.values)-1u || count>(arrayLength(&selected.values)-1u)/8u{return;}
    for(var row=0u;row<count;row++){
        let candidate=indices.values[1u+row];
        if candidate>=authors.values[0] || candidate>=(arrayLength(&authors.values)-1u)/8u{return;}
        let input=1u+candidate*8u;
        if authors.values[input+3u]!=clusters.values[1u+row] ||
            authors.values[input+7u]!=0u{return;}
        for(var word=0u;word<8u;word++){
            selected.values[1u+row*8u+word]=authors.values[input+word];
        }
    }
    selected.values[0]=count;
}
"###;
fn encode_terminal_selected(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64), String> {
    use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState;
    let data = gpu
        .virtual_geometry_execution_data()
        .ok_or("GPU candidate author table is absent")?;
    if data.selected_clusters.len() > 4096 {
        return Err("GPU candidate author table exceeds graph capacity".into());
    }
    let mut words = Vec::with_capacity(1 + data.selected_clusters.len() * 8);
    words.push(
        u32::try_from(data.selected_clusters.len())
            .map_err(|_| "candidate author count overflow")?,
    );
    for cluster in &data.selected_clusters {
        words.extend_from_slice(&[
            cluster.instance_index.unwrap_or(u32::MAX),
            cluster.entity as u32,
            (cluster.entity >> 32) as u32,
            cluster.cluster_id,
            cluster.cluster_ordinal,
            cluster.page_id,
            u32::from(cluster.lod_level),
            match cluster.state {
                RenderVirtualGeometryExecutionState::Resident => 0,
                RenderVirtualGeometryExecutionState::PendingUpload => 1,
                RenderVirtualGeometryExecutionState::Missing => 2,
            },
        ]);
    }
    let mut views = Vec::new();
    for (name, kind, expected) in [
        (
            "virtual-geometry-visible-candidate-indices",
            RenderGraphResourceAccessKind::Read,
            (1 + 4096) * 4,
        ),
        (
            "virtual-geometry-visible-clusters",
            RenderGraphResourceAccessKind::Read,
            (1 + 4096) * 4,
        ),
        (
            "virtual-geometry-execution-selected-clusters",
            RenderGraphResourceAccessKind::Write,
            (1 + 4096 * 8) * 4,
        ),
    ] {
        let binding = gpu.require_buffer_binding(name, kind)?;
        let bytes = binding
            .size
            .map(|n| n.get())
            .unwrap_or(binding.buffer.size() - binding.offset);
        if bytes != expected {
            return Err(format!("invalid GPU selected-cluster range: {name}"));
        }
        views.push((binding.buffer.clone(), binding.offset, binding.size, bytes));
    }
    let output = views[2].clone();
    let native = gpu.native_context();
    let authors = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("virtual-geometry-candidate-author-metadata"),
        contents: bytemuck::cast_slice(&words),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let entries = (0..4)
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
        .collect::<Vec<_>>();
    let layout = native.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("virtual-geometry-selected-projection"),
        entries: &entries,
    });
    let pipeline_layout = native.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("virtual-geometry-selected-projection"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let shader = native.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("virtual-geometry-selected-projection"),
        source: wgpu::ShaderSource::Wgsl(SELECTED_SHADER.into()),
    });
    let pipeline = native.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("virtual-geometry-selected-projection"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let view = |index: usize| {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: &views[index].0,
            offset: views[index].1,
            size: views[index].2,
        })
    };
    let group = native.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("virtual-geometry-selected-projection"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: view(0),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: view(1),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: authors.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: view(2),
            },
        ],
    });
    native
        .encoder
        .clear_buffer(&output.0, output.1, Some(output.3));
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("virtual-geometry-selected-projection"),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
    drop(pass);
    Ok(output)
}

fn record_terminal_completion(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    buffers: &[(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64)],
    metadata: &(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64),
    selected: &(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64),
) -> Result<(), String> {
    // The only values captured by the decoder are physical byte ranges. No CPU
    // author record, expected result or planner visibility enters the callback.
    let mut sources = vec![
        buffers[0].clone(),
        buffers[2].clone(),
        buffers[3].clone(),
        buffers[5].clone(),
        buffers[6].clone(),
        buffers[7].clone(),
        metadata.clone(),
        selected.clone(),
    ];
    for (name, expected) in [
        ("virtual-geometry-visbuffer64", (1 + 4096 * 2) * 4),
        ("virtual-geometry-nonresident-page-requests", (1 + 4096) * 4),
    ] {
        let binding = gpu.require_buffer_binding(name, RenderGraphResourceAccessKind::Read)?;
        let bytes = binding
            .size
            .map(|n| n.get())
            .unwrap_or(binding.buffer.size() - binding.offset);
        if bytes != expected {
            return Err(format!("invalid native terminal source size: {name}"));
        }
        sources.push((binding.buffer.clone(), binding.offset, binding.size, bytes));
    }
    let mut ranges = Vec::with_capacity(sources.len());
    let mut bytes = 0u64;
    for (_, _, _, size) in &sources {
        let end = bytes
            .checked_add(*size)
            .ok_or("native terminal staging length overflow")?;
        ranges.push((bytes, end));
        bytes = end;
    }
    let Some(reservation) = gpu.reserve_virtual_geometry_readback(bytes)? else {
        return Ok(());
    };
    let staging = {
        let native = gpu.native_context();
        let staging_length = usize::try_from(bytes)
            .map_err(|_| "native terminal staging allocation exceeds usize")?;
        let staging_contents = vec![0_u8; staging_length];
        let staging = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("virtual-geometry-terminal-readback"),
            contents: &staging_contents,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        });
        for ((buffer, offset, _, size), (start, _)) in sources.iter().zip(&ranges) {
            native
                .encoder
                .copy_buffer_to_buffer(buffer, *offset, &staging, *start, *size);
        }
        staging
    };
    gpu.enqueue_reserved_virtual_geometry_readback(
        reservation,
        staging,
        0..bytes,
        Box::new(move |mapped| {
            use zircon_runtime::core::framework::render::{
                RenderVirtualGeometryExecutionState,
                RenderVirtualGeometryHardwareRasterizationRecord,
                RenderVirtualGeometryReadbackOutputs, RenderVirtualGeometrySelectedCluster,
                RenderVirtualGeometryVisBuffer64Entry,
            };
            if u64::try_from(mapped.len()).map_err(|_| "mapped length overflow")? != bytes {
                return Err(
                    "native terminal mapped length does not match the full allocation".into(),
                );
            }
            let mut streams = Vec::new();
            for &(start, end) in &ranges {
                let start = usize::try_from(start).map_err(|_| "native range overflow")?;
                let end = usize::try_from(end).map_err(|_| "native range overflow")?;
                let raw = mapped
                    .get(start..end)
                    .ok_or("native range exceeds actual mapped bytes")?;
                if raw.len() % 4 != 0 {
                    return Err("unaligned native word stream".into());
                }
                streams.push(
                    raw.chunks_exact(4)
                        .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
                        .collect::<Vec<_>>(),
                );
            }
            let count = |stream: &[u32], stride: usize| -> Result<usize, String> {
                let n = usize::try_from(*stream.first().ok_or("GPU count header is absent")?)
                    .map_err(|_| "GPU count does not fit the host")?;
                if n > stream.len().saturating_sub(1) / stride {
                    return Err("GPU count exceeds mapped rows".into());
                }
                Ok(n)
            };
            let journal = &streams[2];
            let issued = usize::try_from(journal[0]).map_err(|_| "GPU journal count overflow")?;
            if journal[1] != 0 || issued > journal.len() - 2 {
                return Err("GPU journal rejected admission".into());
            }
            let mut live = 0usize;
            for slot in &journal[2..2 + issued] {
                let slot = usize::try_from(*slot).map_err(|_| "GPU journal slot overflow")?;
                let arg = slot.checked_mul(5).ok_or("GPU argument offset overflow")?;
                if streams[1].get(slot) != Some(&0) {
                    return Err("GPU indexed adapter rejected a source".into());
                }
                let args = streams[0]
                    .get(arg..arg + 5)
                    .ok_or("GPU journal refers outside indexed arguments")?;
                if args[0] == 0 || args[1] == 0 {
                    continue;
                }
                if args[1] != 1 {
                    return Err("GPU indexed instance count violates the admitted span".into());
                }
                live += 1;
            }
            let executed = count(&streams[3], 4)?;
            if executed != live
                || count(&streams[5], 17)? != live
                || count(&streams[6], 18)? != live
            {
                return Err(
                    "GPU terminal output count disagrees with actual issued indexed arguments"
                        .into(),
                );
            }
            if executed > streams[4].len() / 14 {
                return Err("GPU authority exceeds mapped native range".into());
            }
            let state = |word: u32| -> Result<RenderVirtualGeometryExecutionState, String> {
                match word {
                    0 => Ok(RenderVirtualGeometryExecutionState::Resident),
                    1 => Ok(RenderVirtualGeometryExecutionState::PendingUpload),
                    2 => Ok(RenderVirtualGeometryExecutionState::Missing),
                    _ => Err("GPU execution state is invalid".into()),
                }
            };
            let optional = |word: u32| if word == u32::MAX { None } else { Some(word) };
            let mut outputs = RenderVirtualGeometryReadbackOutputs::default();
            for row in streams[6][1..1 + executed * 18].chunks_exact(18) {
                outputs.hardware_rasterization_records.push(
                    RenderVirtualGeometryHardwareRasterizationRecord {
                        instance_index: optional(row[0]),
                        entity: u64::from(row[1]) | (u64::from(row[2]) << 32),
                        cluster_id: row[3],
                        cluster_ordinal: row[4],
                        page_id: row[5],
                        lod_level: u8::try_from(row[6]).map_err(|_| "GPU LOD exceeds its ABI")?,
                        submission_index: row[7],
                        submission_page_id: row[8],
                        submission_lod_level: u8::try_from(row[9])
                            .map_err(|_| "GPU submission LOD exceeds its ABI")?,
                        entity_cluster_start_ordinal: row[10],
                        entity_cluster_span_count: row[11],
                        entity_cluster_total_count: row[12],
                        lineage_depth: row[13],
                        frontier_rank: row[14],
                        resident_slot: optional(row[15]),
                        submission_slot: optional(row[16]),
                        state: state(row[17])?,
                    },
                );
            }
            let visible = count(&streams[7], 8)?;
            if count(&streams[8], 2)? != visible {
                return Err("GPU selected metadata and VisBuffer64 counts disagree".into());
            }
            for (index, row) in streams[7][1..1 + visible * 8].chunks_exact(8).enumerate() {
                let cluster = RenderVirtualGeometrySelectedCluster {
                    instance_index: optional(row[0]),
                    entity: u64::from(row[1]) | (u64::from(row[2]) << 32),
                    cluster_id: row[3],
                    cluster_ordinal: row[4],
                    page_id: row[5],
                    lod_level: u8::try_from(row[6])
                        .map_err(|_| "GPU selected LOD exceeds its ABI")?,
                    state: state(row[7])?,
                };
                let packed = u64::from(streams[8][1 + index * 2])
                    | (u64::from(streams[8][2 + index * 2]) << 32);
                if packed
                    != RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
                        cluster.instance_index,
                        cluster.cluster_id,
                        cluster.page_id,
                        cluster.lod_level,
                        cluster.state,
                    )
                {
                    return Err(
                        "actual GPU VisBuffer64 words disagree with actual selected identity"
                            .into(),
                    );
                }
                outputs
                    .visbuffer64_entries
                    .push(RenderVirtualGeometryVisBuffer64Entry {
                        entry_index: u32::try_from(index)
                            .map_err(|_| "GPU VisBuffer index overflow")?,
                        packed_value: packed,
                        instance_index: cluster.instance_index,
                        entity: cluster.entity,
                        cluster_id: cluster.cluster_id,
                        page_id: cluster.page_id,
                        lod_level: cluster.lod_level,
                        state: cluster.state,
                    });
                outputs.selected_clusters.push(cluster);
            }
            let requested = count(&streams[9], 1)?;
            outputs
                .node_cluster_cull
                .page_request_ids
                .extend_from_slice(&streams[9][1..1 + requested]);
            Ok(outputs)
        }),
    )?;
    // Capture physical native sources only after enqueueing their own real
    // completion packet. Missing historical dedicated buffers remain absent.
    use RenderGraphResourceAccessKind::{Read, Write};
    let mut specs = [None; 14];
    specs[0] = Some(("virtual-geometry-indirect-args", Read, 4096, 4));
    specs[1] = Some(("virtual-geometry-indirect-authority", Read, 4096, 14));
    specs[4] = Some((INDIRECT_SEGMENTS, Read, 4096, 12));
    specs[5] = Some(("virtual-geometry-execution-args", Write, 65536, 4));
    specs[6] = Some(("virtual-geometry-execution-authority", Write, 65536, 14));
    specs[7] = Some(("virtual-geometry-execution-submission", Write, 65536, 17));
    specs[10] = Some(("virtual-geometry-indexed-args", Read, 65536, 5));
    specs[11] = Some(("virtual-geometry-indexed-sidecar", Read, 65536, 32));
    specs[12] = Some(("virtual-geometry-indexed-status", Read, 65536, 1));
    specs[13] = Some(("virtual-geometry-admitted-run-slots", Read, 65536, 1));
    let _ = super::native_probe::retain(gpu, specs, 65536)?;
    Ok(())
}
