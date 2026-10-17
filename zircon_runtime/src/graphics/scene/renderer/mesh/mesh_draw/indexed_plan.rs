use super::MeshDraw;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::{
    virtual_geometry_indexed_source_rows, VirtualGeometryIndexedAuthorRun,
    VirtualGeometryIndexedSourceAdmission,
};
use std::sync::Arc;

pub(crate) const INDEXED_ARGS_RESOURCE: &str = "virtual-geometry-indexed-args";
pub(crate) const INDEXED_CAPACITY: u32 = 65536;
#[derive(Debug)]
pub(crate) struct VirtualGeometryIndexedPlan {
    args: Arc<wgpu::Buffer>,
    authors: Vec<VirtualGeometryIndexedAuthorRun>,
}
impl VirtualGeometryIndexedPlan {
    pub(crate) fn args(&self) -> &Arc<wgpu::Buffer> {
        &self.args
    }
    pub(crate) fn authors(&self) -> &[VirtualGeometryIndexedAuthorRun] {
        &self.authors
    }
}
#[derive(Clone, Debug)]
pub(crate) struct VirtualGeometryIndexedRunRef {
    plan: Arc<VirtualGeometryIndexedPlan>,
    slot: u32,
}
impl VirtualGeometryIndexedRunRef {
    pub(crate) fn plan(&self) -> &Arc<VirtualGeometryIndexedPlan> {
        &self.plan
    }
    pub(crate) fn slot(&self) -> u32 {
        self.slot
    }
    pub(crate) fn author(&self) -> &VirtualGeometryIndexedAuthorRun {
        &self.plan.authors[self.slot as usize]
    }
}
fn global_id(mapping: &[(u32, u32)], local: u32) -> Result<u32, String> {
    let values = mapping
        .iter()
        .filter(|(key, _)| *key == local)
        .collect::<Vec<_>>();
    if values.len() != 1 {
        return Err("admitted topology has an ambiguous or absent remapped ID".into());
    }
    Ok(values[0].1)
}
fn captured<'a>(
    sources: &'a [VirtualGeometryIndexedSourceAdmission],
    stable_key: u64,
    bound: &crate::graphics::VirtualGeometryPrimitiveAdmission,
) -> Result<&'a VirtualGeometryIndexedSourceAdmission, String> {
    let matches = sources
        .iter()
        .filter(|source| {
            source.execution_stable_instance_key() == stable_key
                && source.primitive().same_primitive(bound)
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("main indexed draw has no unique captured source admission".into());
    }
    Ok(matches[0])
}
pub(crate) fn expand_indexed_draws(
    device: &wgpu::Device,
    frame: &ViewportRenderFrame,
    draws: Vec<MeshDraw>,
) -> Result<Vec<MeshDraw>, String> {
    let author_inputs = frame
        .virtual_geometry_execution_data
        .as_deref()
        .and_then(|data| data.prepared_cpu_frame.as_ref())
        .map(|prepared| &prepared.execution_inputs);
    let explicit_empty = author_inputs.is_some_and(|inputs| {
        inputs.selections.as_ref().is_some_and(Vec::is_empty)
            || inputs.prepare.is_some()
                && frame
                    .virtual_geometry_execution_data
                    .as_deref()
                    .and_then(|data| data.prepared_cpu_frame.as_ref())
                    .and_then(|prepared| prepared.paired_execution_segments.as_ref())
                    .is_some_and(Vec::is_empty)
    });
    let mut admitted_draws = Vec::with_capacity(draws.len());
    for draw in draws {
        let captured = draw.mesh.source_admission().is_some_and(|bound| {
            frame.virtual_geometry_indexed_sources.iter().any(|source| {
                source.stable_instance_key() == draw.stable_instance_key
                    && source.primitive().same_primitive(bound)
            })
        });
        if captured && explicit_empty {
            continue;
        }
        if captured && draw.virtual_geometry_submission_detail.is_none() {
            return Err(
                "captured VG draw has no exact execution segment; whole-mesh replay is forbidden"
                    .into(),
            );
        }
        admitted_draws.push(draw);
    }
    let draws = admitted_draws;
    if !draws
        .iter()
        .any(|draw| draw.virtual_geometry_submission_detail.is_some())
    {
        return Ok(draws);
    }
    let data = frame
        .virtual_geometry_execution_data
        .as_deref()
        .ok_or("indexed main draw has no actual execution data")?;
    let rows = virtual_geometry_indexed_source_rows(data)?;
    let mut groups = Vec::with_capacity(draws.len());
    let mut authors = Vec::new();
    for draw in draws {
        let Some(detail) = draw.virtual_geometry_submission_detail else {
            groups.push((draw, None));
            continue;
        };
        let identity = detail
            .execution_identity()
            .ok_or("main indexed draw lost its full execution identity")?;
        let bound = draw
            .mesh
            .source_admission()
            .ok_or("main indexed draw is bound to a mesh without canonical provenance")?;
        let source = captured(
            &frame.virtual_geometry_indexed_sources,
            identity.stable_instance_key,
            bound,
        )?;
        if !source.primitive().same_primitive(bound) {
            return Err(
                "captured indexed source and actual bound GPU mesh are different admissions".into(),
            );
        }
        let count = u32::try_from(bound.primitive().indices.len())
            .map_err(|_| "admitted index count exceeds indexed u32 ABI")?;
        if draw.mesh.index_count != count {
            return Err("actual bound GPU index count differs from admission".into());
        }
        let (first_instance, instance_count) = draw
            .gpu_scene_instance_span
            .ok_or("main indexed draw has no actual GPUScene span")?;
        if instance_count != 1 {
            return Err(
                "indexed source requires the actual single-instance GPUScene allocation".into(),
            );
        }
        first_instance
            .checked_add(instance_count)
            .ok_or("GPUScene span overflow")?;
        let topology = bound.indexed_topology().map_err(str::to_owned)?;
        let mut slots = Vec::new();
        for row in rows.iter().filter(|row| {
            row.identity() == identity && source.owns_resident_payload(row.page_index())
        }) {
            let mut matching = Vec::new();
            for run in topology {
                if global_id(source.page_ids(), run.page_id)? == identity.page_id
                    && global_id(source.cluster_ids(), run.cluster_id)? == row.cluster_id()
                    && usize::try_from(run.range_index)
                        .map_err(|_| "admitted range ordinal exceeds usize")?
                        == row.range_index()
                {
                    matching.push(run);
                }
            }
            if matching.len() != 1 {
                return Err(
                    "source row does not identify exactly one admitted original index range".into(),
                );
            }
            if authors.len() >= INDEXED_CAPACITY as usize {
                return Err("main indexed run capacity exceeded".into());
            }
            let slot = u32::try_from(authors.len()).map_err(|_| "indexed slot exceeds u32")?;
            let run = matching[0];
            if run.indices.start >= run.indices.end || run.indices.end > count {
                return Err("admitted indexed range exceeds actual bound mesh".into());
            }
            authors.push(VirtualGeometryIndexedAuthorRun::new(
                if source.has_authored_execution_binding() {
                    row.clone().with_draw_reference_identity(
                        detail.draw_ref_index(),
                        detail.draw_ref_rank(),
                    )?
                } else {
                    row.clone()
                },
                run.indices.clone(),
                count,
                first_instance,
            ));
            slots.push(slot);
        }
        // A resident draw with no matching native source must never fall back to
        // whole-segment CPU args. The failed frame cannot publish a completion.
        if slots.is_empty() {
            return Err("resident indexed draw has no canonical source rows".into());
        }
        groups.push((draw, Some(slots)));
    }
    if authors.is_empty() {
        return Err("indexed main frame has no admitted runs".into());
    }
    // A fresh physical allocation is exclusive to this frame. No persistent
    // cache or transient pool can rewrite it after native capture retains it.
    let args = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indexed-args-frame"),
        size: u64::from(INDEXED_CAPACITY) * 20,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::INDIRECT
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    }));
    let plan = Arc::new(VirtualGeometryIndexedPlan { args, authors });
    let mut expanded = Vec::new();
    for (draw, slots) in groups {
        match slots {
            None => expanded.push(draw),
            Some(slots) => {
                for slot in slots {
                    let mut command_draw = draw.clone();
                    let author = &plan.authors[slot as usize];
                    command_draw.first_index = author.index_range().start;
                    command_draw.draw_index_count =
                        author.index_range().end - author.index_range().start;
                    command_draw.indirect_args_buffer = Some(Arc::clone(&plan.args));
                    command_draw.indirect_args_offset = u64::from(slot) * 20;
                    command_draw.virtual_geometry_indexed_run =
                        Some(VirtualGeometryIndexedRunRef {
                            plan: Arc::clone(&plan),
                            slot,
                        });
                    expanded.push(command_draw);
                }
            }
        }
    }
    Ok(expanded)
}

pub(crate) const INDEXED_JOURNAL_RESOURCE: &str = "virtual-geometry-admitted-run-slots";
impl VirtualGeometryIndexedPlan {
    pub(crate) fn empty(device: &wgpu::Device) -> Arc<Self> {
        Arc::new(Self {
            args: Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("zircon-vg-indexed-args-empty-frame"),
                size: u64::from(INDEXED_CAPACITY) * 20,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })),
            authors: Vec::new(),
        })
    }
    pub(crate) fn append_issued(
        self: &Arc<Self>,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        journal: &wgpu::Buffer,
        issued: &[VirtualGeometryIndexedRunRef],
    ) -> Result<(), String> {
        use wgpu::util::DeviceExt;
        if issued.is_empty() {
            return Ok(());
        }
        if issued.len() > INDEXED_CAPACITY as usize {
            return Err("actual indexed admission count exceeds journal capacity".into());
        }
        if journal.size() < (u64::from(INDEXED_CAPACITY) + 2) * 4
            || !journal.usage().contains(wgpu::BufferUsages::STORAGE)
        {
            return Err("actual indexed journal buffer violates physical capacity or usage".into());
        }
        let mut words = Vec::with_capacity(issued.len() + 1);
        words.push(u32::try_from(issued.len()).map_err(|_| "issued indexed count exceeds u32")?);
        for run in issued {
            if !Arc::ptr_eq(self, run.plan()) || run.slot() as usize >= self.authors.len() {
                return Err("actual recorded indexed command belongs to another frame plan".into());
            }
            words.push(run.slot());
        }
        let upload = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-actual-issued-slots"),
            contents: bytemuck::cast_slice(&words),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-vg-append-actual-issued"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(INDEXED_JOURNAL_SHADER)),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-vg-append-actual-issued"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-vg-append-actual-issued"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: upload.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: journal.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("zircon-vg-append-actual-issued"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.dispatch_workgroups(1, 1, 1);
        Ok(())
    }
}
const INDEXED_JOURNAL_SHADER: &str = r#"
@group(0) @binding(0) var<storage,read> issued:array<u32>;
@group(0) @binding(1) var<storage,read_write> journal:array<atomic<u32>>;
@compute @workgroup_size(1)
fn main() {
    let count=issued[0];
    let base=atomicLoad(&journal[0]);
    if (count>65536u || base>65536u-count) {
        atomicStore(&journal[1],1u); return;
    }
    for(var i=0u;i<count;i=i+1u) { atomicStore(&journal[2u+base+i],issued[1u+i]); }
    atomicStore(&journal[0],base+count);
}
"#;
