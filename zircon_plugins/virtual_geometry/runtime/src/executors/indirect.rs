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
                if instance.is_none()
                    && data
                        .instances
                        .iter()
                        .any(|instance| instance.entity == segment.entity)
                {
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
    // Keep the source lease and pipeline lock while ending the factory capability borrow.
    let (_source, cache, bind_group) = {
        let native = gpu.native_context();
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
        (source, cache, bind_group)
    };
    gpu.buffer_upload_recorder().write_buffer(
        &args_buffer,
        args_offset,
        bytemuck::bytes_of(&0_u32),
    );
    let groups = u32::try_from(records.len().div_ceil(WORKGROUP_SIZE as usize).max(1))
        .map_err(|_| "virtual geometry indirect dispatch count exceeds u32".to_string())?;
    let pipeline = &cache.as_ref().expect("pipeline initialized").1;
    {
        let native = gpu.native_context();
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
    }
    drop(cache);
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
    let (vertices, ranges) = super::gpu::cluster_vertices_with_ranges(gpu)?;
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
    let admitted = admitted_source_rows(gpu, &rows)?;
    let mut records = Vec::with_capacity(rows.len());
    for (index, (row, range)) in rows.iter().zip(ranges).enumerate() {
        let identity = row.identity();
        let authorized = admitted.as_ref().is_none_or(|mask| mask[index]);
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
            vertex_count: if authorized { count } else { 0 },
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

fn admitted_source_rows(
    gpu: &RenderPassGpuExecutionContext<'_>,
    rows: &[zircon_runtime::graphics::VirtualGeometryIndexedSourceRow],
) -> Result<Option<Vec<bool>>, String> {
    let mut projected = vec![false; rows.len()];
    for (index, row) in rows.iter().enumerate() {
        if usize::try_from(row.source_row()).map_err(|_| "source row index overflow")? != index {
            return Err("canonical source rows are not in their actual fixed slots".into());
        }
    }
    if let Some(authors) = gpu.virtual_geometry_indexed_author_runs() {
        let mut physical_runs = std::collections::HashSet::new();
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
                || source.has_authored_topology_binding() != base.has_authored_topology_binding()
            {
                return Err(
                    "actual author source lost its original candidate/payload provenance".into(),
                );
            }
            let actual = source.identity();
            let range = author.index_range();
            if !physical_runs.insert((actual, range.start, range.end, author.gpu_scene_instance()))
            {
                return Err("duplicate complete physical author run".into());
            }
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
                && (!source.has_authored_topology_binding()
                    || original.submission_index.is_none()
                    || actual
                        .draw_ref_rank
                        .is_none_or(|rank| rank > u32::from(u16::MAX)))
            {
                return Err(
                    "changed draw reference is not a checked legacy author identity".into(),
                );
            }
            projected[index] = true;
        }
    }
    Ok(gpu
        .virtual_geometry_indexed_author_runs()
        .map(|_| projected))
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
    admitted_source_rows(gpu, &rows)?;
    if authors.is_empty()
        && !rows.is_empty()
        && !gpu.virtual_geometry_indexed_empty_admission_is_proven()
    {
        return Err("visible-source frame has no proven bound-mesh indexed admission".into());
    }
    let mut runs = Vec::with_capacity(authors.len());
    for (ordinal, author) in authors.iter().enumerate() {
        let source = author.source();
        let first = author.gpu_scene_instance();
        let canonical = rows
            .get(source.source_row() as usize)
            .ok_or("indexed author source row exceeds canonical catalog")?;
        runs.push(indexed::IndexedAuthorRun::try_new_projected(
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
            canonical.identity(),
            source.has_authored_topology_binding(),
        )?);
    }
    let identities = rows.iter().map(|row| row.identity()).collect::<Vec<_>>();
    let author_words = terminal_author_metadata(gpu)?;
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
        drop(native);
        return super::selection::pre_raster_gate(gpu, &author_words);
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
    )?;
    drop(native);
    super::selection::pre_raster_gate(gpu, &author_words)
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
    execution_args.values[0]=0xffffffffu;
    execution_submission.values[0]=0xffffffffu;
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
        let token=(min(sidecar.values[sidecar_offset+8u],65535u)<<16u)|min(sidecar.values[sidecar_offset+9u],65535u);
        let submission=sidecar.values[sidecar_offset+8u];
        let rank=sidecar.values[sidecar_offset+9u];
        if source.values[source_offset]==0u || source.values[source_offset+1u]!=1u ||
            token!=((min(submission,65535u)<<16u)|min(rank,65535u)) {return;}
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
        execution_args.values[1u+output*4u+3u]=token;
        execution_submission.values[1u+output*17u+15u]=token;
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
    let inputs = gpu.virtual_geometry_execution_inputs();
    let seed_driven = inputs.prepare.is_none()
        && inputs.selections.is_none()
        && gpu
            .virtual_geometry_execution_data()
            .is_some_and(|data| !data.instances.is_empty());
    let metadata = encode_terminal_metadata(gpu, &buffers)?;
    // Preserve physical-run metadata before seed projection reuses its graph output.
    let physical_metadata = preserve_physical_terminal_metadata(gpu, &metadata)?;
    let (metadata, selected) = if seed_driven {
        super::selection::encode_seed_decision_projection(gpu, &buffers)?
    } else {
        (metadata, encode_terminal_selected(gpu)?)
    };
    super::selection::final_publication_gate(gpu)?;
    record_terminal_completion(gpu, &buffers, &metadata, &selected, &physical_metadata)?;
    Ok(())
}

const METADATA_SHADER: &str = r###"
struct Words { values:array<u32> }
@group(0) @binding(0) var<storage,read> authority:Words;
@group(0) @binding(1) var<storage,read> submission:Words;
@group(0) @binding(2) var<storage,read> authors:Words;
@group(0) @binding(3) var<storage,read_write> output:Words;
@group(0) @binding(4) var<storage,read> canonical_source:Words;
@compute @workgroup_size(1)
fn cs_main(){
    output.values[0]=0xffffffffu;
    let count=submission.values[0];
    let capacity=(arrayLength(&output.values)-1u)/18u;
    if count>capacity || count>(arrayLength(&submission.values)-1u)/17u {return;}
    for(var row=0u;row<count;row++){
        let input=1u+row*17u;
        let source=submission.values[input];
        if source>=authors.values[0] || source>=(arrayLength(&authors.values)-1u)/32u {return;}
        let author=1u+source*32u;
        let projected=(authors.values[author+4u]&16u)!=0u;
        if (authors.values[author+4u]&0xffffffe0u)!=0u {return;}
        for(var word=0u;word<14u;word++){
            var expected=authors.values[author+word];
            if word==4u {expected=expected&15u;}
            if projected && (word==1u || word==10u) {continue;}
            if submission.values[input+1u+word]!=expected {return;}
        }
        let canonical_token=(min(authors.values[author+9u],65535u)<<16u)|min(authors.values[author+10u],65535u);
        if source>=(arrayLength(&canonical_source.values)-1u)/4u ||
            canonical_source.values[1u+source*4u+3u]!=canonical_token {return;}
        let record=author+14u;
        let actual=row*14u;
        // Validate every identity/ordinal field represented by raw authority.
        if authority.values[actual]!=authors.values[record] ||
            authority.values[actual+1u]!=authors.values[record+1u] ||
            authority.values[actual+2u]!=authors.values[record+2u] ||
            authority.values[actual+3u]!=authors.values[record+5u] ||
            authority.values[actual+4u]!=authors.values[record+3u] ||
            authority.values[actual+5u]!=authors.values[record+4u] ||
            authority.values[actual+6u]!=submission.values[input+2u] ||
            authority.values[actual+7u]!=authors.values[author] ||
            authority.values[actual+8u]!=authors.values[record+7u] ||
            authority.values[actual+9u]!=submission.values[input+11u] ||
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
    let identities = rows.iter().map(|row| row.identity()).collect::<Vec<_>>();
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
    let admitted = admitted_source_rows(gpu, &rows)?;
    let mut words = Vec::with_capacity(1 + rows.len() * 32);
    words.push(u32::try_from(rows.len()).map_err(|_| "terminal author count overflow")?);
    for (row_index, (row, identity)) in rows.iter().zip(identities).enumerate() {
        if admitted.as_ref().is_some_and(|mask| !mask[row_index]) {
            words.extend_from_slice(&indexed::identity_words(identity));
            words.extend_from_slice(&[u32::MAX; 18]);
            continue;
        }
        let raw_domain = zircon_runtime::core::framework::render::RenderVirtualGeometryExtract::checked_cluster_row_indices(
            &data.instances, &data.author_cluster_catalog, identity.entity,
            identity.stable_instance_key, identity.instance_index).map_err(str::to_owned)?;
        let ordinal = row.cluster_ordinal();
        let total = row.normal_cluster_total();
        if ordinal >= total {
            return Err("author ordinal exceeds normal policy total".into());
        }
        let index = row.author_array_index();
        if !raw_domain.contains(&(index as usize)) {
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
        let mut identity_words = indexed::identity_words(identity);
        if row.has_authored_topology_binding() {
            identity_words[4] |= 16;
        }
        words.extend_from_slice(&identity_words);
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
    let entries = (0..5)
        .map(|binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: binding != 3,
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
            wgpu::BindGroupEntry {
                binding: 4,
                resource: view(4),
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

pub(super) const SELECTED_SHADER: &str = r###"
struct Words { values:array<u32> }
@group(0) @binding(0) var<storage,read> indices:Words;
@group(0) @binding(1) var<storage,read> clusters:Words;
@group(0) @binding(2) var<storage,read> authors:Words;
@group(0) @binding(3) var<storage,read_write> selected:Words;
@group(0) @binding(4) var<storage,read> vis:Words;
@compute @workgroup_size(1)
fn cs_main(){
    selected.values[0]=0xffffffffu;
    let count=indices.values[0];
    if count!=clusters.values[0] || count!=vis.values[0] || count>(arrayLength(&vis.values)-1u)/2u || count>arrayLength(&indices.values)-1u ||
        count>arrayLength(&clusters.values)-1u || count>(arrayLength(&selected.values)-1u)/8u{return;}
    for(var row=0u;row<count;row++){
        let candidate=indices.values[1u+row];
        if candidate>=authors.values[0] || candidate>=(arrayLength(&authors.values)-1u)/8u{return;}
        let input=1u+candidate*8u;
        let cluster=authors.values[input+3u];let page=authors.values[input+5u];
        let instance=authors.values[input];let lod=authors.values[input+6u];let state=authors.values[input+7u];
        if cluster!=clusters.values[1u+row] || state>2u ||
           vis.values[1u+row*2u]!=((cluster&0xfffffu)|((page&0xfffu)<<20u)) ||
           vis.values[2u+row*2u]!=(((page>>12u)&0xffu)|((instance&0xffffu)<<8u)|(lod<<24u)|(state<<30u)){return;}
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
        (
            "virtual-geometry-visbuffer64",
            RenderGraphResourceAccessKind::Read,
            (1 + 4096 * 2) * 4,
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
    let entries = (0..5)
        .map(|binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: binding != 3,
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
            wgpu::BindGroupEntry {
                binding: 4,
                resource: view(3),
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

fn preserve_physical_terminal_metadata(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    source: &(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64),
) -> Result<(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64), String> {
    let bytes = (1 + 65536 * 18) * 4_u64;
    let end = source
        .1
        .checked_add(bytes)
        .ok_or("physical metadata range overflow")?;
    if source.3 != bytes
        || source.2.is_some_and(|size| size.get() != bytes)
        || source.1 % 4 != 0
        || end > source.0.size()
        || !source
            .0
            .usage()
            .contains(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC)
    {
        return Err("physical terminal metadata has an invalid copy range or usage".into());
    }
    let length =
        usize::try_from(bytes).map_err(|_| "physical metadata allocation exceeds usize")?;
    let contents = vec![0_u8; length];
    let native = gpu.native_context();
    let copy = native.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("virtual-geometry-physical-terminal-metadata"),
        contents: &contents,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });
    // Ordered after METADATA_SHADER dispatch, before any seed-selected overwrite.
    native
        .encoder
        .copy_buffer_to_buffer(&source.0, source.1, &copy, 0, bytes);
    Ok((copy, 0, std::num::NonZeroU64::new(bytes), bytes))
}

fn physical_terminal_row_matches(
    row: &[u32],
    authority: &[u32],
    sidecar: &[u32],
    hw: &zircon_runtime::core::framework::render::RenderVirtualGeometryHardwareRasterizationRecord,
    args: &[u32],
) -> bool {
    use zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState;
    if row.len() != 17 || authority.len() != 14 || sidecar.len() != 32 || args.len() != 4 {
        return false;
    }
    let identity = &row[1..15];
    let flags = identity[4];
    let token = (identity[9].min(0xffff) << 16) | identity[10].min(0xffff);
    !(flags != 15
        || identity[5] == u32::MAX
        || identity[9] == u32::MAX
        || identity[10] == u32::MAX
        || identity[13] == u32::MAX
        || sidecar[31] != 1
        || sidecar[28] != row[0]
        || sidecar[..14] != *authority
        || sidecar[14..28] != *identity
        || row[15] != token
        || row[16] != 1
        || args[3] != token
        || args[0] == 0
        || args[1] != 1
        || authority[0] != identity[5]
        || authority[1..3] != identity[6..8]
        || authority[3] != identity[8]
        || authority[6] != identity[1]
        || authority[7] != identity[0]
        || authority[8] != identity[9]
        || authority[9] != identity[10]
        || authority[10] != identity[13]
        || hw.instance_index != Some(identity[5])
        || hw.entity != (u64::from(identity[6]) | (u64::from(identity[7]) << 32))
        || hw.page_id != identity[8]
        || hw.submission_page_id != identity[8]
        || hw.submission_index != identity[9]
        || hw.entity_cluster_start_ordinal != identity[11]
        || hw.entity_cluster_span_count != identity[12]
        || hw.submission_slot != Some(identity[13])
        || hw.resident_slot != Some(identity[13])
        || hw.cluster_id != authority[4]
        || hw.cluster_ordinal != authority[5]
        || hw.lineage_depth != authority[12]
        || hw.frontier_rank != authority[13]
        || hw.state != RenderVirtualGeometryExecutionState::Resident)
}

fn record_terminal_completion(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    buffers: &[(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64)],
    metadata: &(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64),
    selected: &(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64),
    physical_metadata: &(wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64),
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
    let decision = gpu.require_buffer_binding(
        "virtual-geometry-cull-decisions",
        RenderGraphResourceAccessKind::Write,
    )?;
    if binding_size(&decision) != (8 + 4096 * 33) * 4 {
        return Err("VG terminal decision buffer has a different physical ABI".into());
    }
    sources.push((
        decision.buffer.clone(),
        decision.offset,
        decision.size,
        binding_size(&decision),
    ));
    // Preserve the same live-slot sidecar for full terminal identity verification.
    sources.push(buffers[1].clone());
    // Stream 12 is the immutable physical18 copy; stream 6 remains selected diagnostics.
    sources.push(physical_metadata.clone());
    let original_source_count = sources.len();
    let inspection = gpu.take_virtual_geometry_native_inspection()?;
    let mut inspection_bytes = 0_u64;
    if let Some((sections, _)) = &inspection {
        for (_, buffer, range) in sections {
            let size = range
                .end
                .checked_sub(range.start)
                .ok_or("reversed native inspection range")?;
            inspection_bytes = inspection_bytes
                .checked_add(size)
                .ok_or("native inspection length overflow")?;
            sources.push((
                buffer.clone(),
                range.start,
                std::num::NonZeroU64::new(size),
                size,
            ));
        }
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
    let inspection_start = bytes
        .checked_sub(inspection_bytes)
        .ok_or("native inspection range underflow")?;
    let inspection_decoder = inspection.map(|(sections, decode)| {
        Box::new(move |mapped: &[u8], outputs: &zircon_runtime::core::framework::render::RenderVirtualGeometryReadbackOutputs| {
            let _keepalive = &sections;
            let start = usize::try_from(inspection_start).map_err(|_| "native inspection offset overflow")?;
            decode(mapped.get(start..).ok_or("native inspection exceeds full mapped staging")?, outputs)
        }) as Box<dyn FnOnce(&[u8], &zircon_runtime::core::framework::render::RenderVirtualGeometryReadbackOutputs)
            -> Result<zircon_runtime::core::framework::render::RenderVirtualGeometryNativeReadbackInspection, String> + Send>
    });
    gpu.enqueue_reserved_virtual_geometry_readback_with_inspection(
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
            for &(start, end) in &ranges[..original_source_count] {
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
            let physical_metadata_expected = streams.get(10).and_then(|words| words.get(3)).copied() != Some(2);
            let mut live = 0usize;
            let mut live_slots = Vec::new();
            let mut journal_slots = std::collections::HashSet::new();
            for slot in &journal[2..2 + issued] {
                let slot = usize::try_from(*slot).map_err(|_| "GPU journal slot overflow")?;
                if !journal_slots.insert(slot) {
                    return Err("GPU journal repeats a physical run slot".into());
                }
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
                live_slots.push(slot);
                live += 1;
            }
            let executed = count(&streams[3], 4)?;
            if executed != live
                || count(&streams[5], 17)? != live
                || count(&streams[12], 18)? != live
                || (physical_metadata_expected && count(&streams[6], 18)? != live)
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
            let decode_hardware = |row: &[u32]| -> Result<RenderVirtualGeometryHardwareRasterizationRecord, String> {
                Ok(RenderVirtualGeometryHardwareRasterizationRecord {
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
                    })
            };
            let hardware_count = count(&streams[6], 18)?;
            for row in streams[6][1..1 + hardware_count * 18].chunks_exact(18) {
                outputs.hardware_rasterization_records.push(decode_hardware(row)?);
            }
            let mut physical = Vec::with_capacity(executed);
            for (output, slot) in live_slots.into_iter().enumerate() {
                let row = &streams[5][1 + output * 17..1 + (output + 1) * 17];
                let authority = &streams[4][output * 14..(output + 1) * 14];
                let sidecar_start = slot.checked_mul(32).ok_or("GPU sidecar offset overflow")?;
                let sidecar = streams[11].get(sidecar_start..sidecar_start + 32)
                    .ok_or("GPU journal exceeds mapped sidecar")?;
                let physical_row = &streams[12][1 + output * 18..1 + (output + 1) * 18];
                let hw = decode_hardware(physical_row)?;
                let identity = &row[1..15];
                let args = &streams[3][1 + output * 4..1 + (output + 1) * 4];
                if !physical_terminal_row_matches(row, authority, sidecar, &hw, args) {
                    return Err("GPU physical terminal identity/authority/token disagrees with its live journal slot".into());
                }
                physical.push(zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionSegment {
                    original_index: identity[0],
                    draw_ref_index: identity[1],
                    stable_instance_key: u64::from(identity[2]) | (u64::from(identity[3]) << 32),
                    instance_index: Some(identity[5]),
                    entity: hw.entity,
                    page_id: identity[8],
                    submission_index: Some(identity[9]),
                    draw_ref_rank: Some(identity[10]),
                    cluster_start_ordinal: identity[11],
                    cluster_span_count: identity[12],
                    cluster_total_count: hw.entity_cluster_total_count,
                    submission_slot: Some(identity[13]),
                    state: hw.state,
                    lineage_depth: hw.lineage_depth,
                    lod_level: hw.submission_lod_level,
                    frontier_rank: hw.frontier_rank,
                });
            }
            outputs.physical_submission_segments = Some(physical);
            let visible = count(&streams[7], 8)?;
            let decision = &streams[10];
            if decision.len() != 8 + 4096 * 33 || decision[0] != 0x5647_5331
                || decision[1] != 0 || decision[2] as usize != visible
                || !matches!(decision[3], 0 | 2) || decision[6] != 1 || decision[7] != 1 {
                return Err("GPU hierarchy selection did not publish a complete valid decision".into());
            }
            for row in 0..visible {
                let start = 8 + row * 28;
                if streams[7][1 + row * 8..1 + (row + 1) * 8] != decision[start..start + 8] {
                    return Err("selected metadata differs from the actual GPU decision".into());
                }
            }
            if decision[3] == 2 {
                if count(&streams[6], 18)? != visible {
                    return Err("seed hardware startup and GPU selection counts disagree".into());
                }
                for row in 0..visible {
                    let start = 8 + row * 28 + 8;
                    if streams[6][1 + row * 18..1 + (row + 1) * 18] != decision[start..start + 18] {
                        return Err("hardware startup differs from the actual GPU seed decision".into());
                    }
                }
            }
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
        inspection_decoder,
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

#[cfg(test)]
mod physical_terminal_tests {
    use super::physical_terminal_row_matches;
    use zircon_runtime::core::framework::render::{
        RenderVirtualGeometryExecutionState, RenderVirtualGeometryHardwareRasterizationRecord,
    };

    #[test]
    fn physical_terminal_row_requires_all_seventeen_words_and_full_authority() {
        let identity = [4, 6, 13, 0, 15, 2, 41, 0, 7, 0, 1, 3, 2, 9];
        let authority = [2, 41, 0, 7, 11, 3, 6, 4, 0, 1, 9, 8, 2, 5];
        let mut row = vec![12];
        row.extend(identity);
        row.extend([1, 1]);
        let mut sidecar = authority.to_vec();
        sidecar.extend(identity);
        sidecar.extend([12, 0, 0, 1]);
        let hw = RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(2),
            entity: 41,
            cluster_id: 11,
            cluster_ordinal: 3,
            page_id: 7,
            lod_level: 1,
            submission_index: 0,
            submission_page_id: 7,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 3,
            entity_cluster_span_count: 2,
            entity_cluster_total_count: 8,
            lineage_depth: 2,
            frontier_rank: 5,
            resident_slot: Some(9),
            submission_slot: Some(9),
            state: RenderVirtualGeometryExecutionState::Resident,
        };
        let args = [3, 1, 0, 1];
        assert!(physical_terminal_row_matches(
            &row, &authority, &sidecar, &hw, &args
        ));
        for index in 0..17 {
            let mut bad = row.clone();
            bad[index] ^= 1;
            assert!(
                !physical_terminal_row_matches(&bad, &authority, &sidecar, &hw, &args),
                "terminal word {index}"
            );
        }
        for index in 0..14 {
            let mut bad = authority;
            bad[index] ^= 1;
            assert!(
                !physical_terminal_row_matches(&row, &bad, &sidecar, &hw, &args),
                "authority word {index}"
            );
        }
        let mut wrong_args = args;
        wrong_args[3] ^= 1;
        assert!(!physical_terminal_row_matches(
            &row,
            &authority,
            &sidecar,
            &hw,
            &wrong_args
        ));
        assert!(!physical_terminal_row_matches(
            &row[..16],
            &authority,
            &sidecar,
            &hw,
            &args
        ));
    }
}
