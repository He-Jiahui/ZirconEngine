//! Same-frame GPU selection source upload, pre-raster admission, and terminal publication.
//! Graph-owned decision storage is production state; optional inspection never owns it.
use zircon_runtime::graphics::{RenderPassGpuExecutionContext, RenderPassGpuResourceFactory};
use zircon_runtime::render_graph::RenderGraphResourceAccessKind;

/// Uploads source facts only. Seed selection, hierarchy decisions and publication happen on GPU.
pub(super) fn node_selection_source_words(
    gpu: &RenderPassGpuExecutionContext<'_>,
) -> Result<Vec<u32>, String> {
    use zircon_runtime::core::framework::render::{
        ProjectionMode, RenderVirtualGeometryExecutionState,
    };
    let data = gpu
        .virtual_geometry_execution_data()
        .ok_or("VG execution data is absent")?;
    let extract = gpu
        .frame_extract()
        .geometry
        .virtual_geometry
        .as_ref()
        .ok_or("VG source extract is absent")?;
    let frame = data
        .prepared_cpu_frame
        .as_ref()
        .ok_or("VG prepared frame is absent")?;
    let bounded = |n: usize| -> Result<u32, String> {
        let n = u32::try_from(n).map_err(|_| "VG source length overflow")?;
        if n > 4096 {
            return Err("VG bounded selection source exceeds 4096 rows".into());
        }
        Ok(n)
    };
    let mut words = vec![0_u32; 64];
    words[..14].copy_from_slice(&data.cull_input.packed_words());
    words[14] = 0x5647_5331;
    let inputs = gpu.virtual_geometry_execution_inputs();
    words[15] =
        if inputs.prepare.is_none() && inputs.selections.is_none() && !extract.instances.is_empty()
        {
            2
        } else {
            0
        };
    words[16] = bounded(extract.clusters.len())?;
    words[17] = bounded(data.selected_clusters.len())?;
    words[18] = bounded(extract.hierarchy_nodes.len())?;
    words[19] = bounded(extract.hierarchy_child_ids.len())?;
    words[20] = bounded(extract.pages.len())?;
    words[21] = bounded(frame.inspection.instance_work_items.len())?;
    let camera = gpu.frame_extract().view.selected_effective_camera();
    if let Some(global) = &frame.inspection.global_state {
        words[27] = u32::from(global.child_frustum_culling_enabled);
        words[28] = global.child_split_screen_space_error_threshold.to_bits();
    }
    let near = camera.z_near.max(0.001);
    words[29] = near.to_bits();
    words[30] = camera.z_far.max(near).to_bits();
    words[31] = u32::from(camera.projection_mode == ProjectionMode::Perspective);
    words[32] = (camera.fov_y_radians * 0.5).tan().to_bits();
    words[33] = camera.ortho_size.max(0.01).to_bits();
    words[34] = camera.aspect_ratio.max(0.001).to_bits();
    for (target, value) in words[35..51]
        .iter_mut()
        .zip(zircon_runtime::core::math::view_matrix(camera.transform).to_cols_array())
    {
        *target = value.to_bits();
    }
    words[22] = u32::try_from(words.len()).map_err(|_| "VG cluster offset overflow")?;
    for row in &extract.clusters {
        words.extend_from_slice(&[
            row.entity as u32,
            (row.entity >> 32) as u32,
            row.cluster_id,
            row.page_id,
            u32::from(row.lod_level),
            row.parent_cluster_id.unwrap_or(u32::MAX),
            row.hierarchy_node_id.unwrap_or(u32::MAX),
            row.bounds_center.x.to_bits(),
            row.bounds_center.y.to_bits(),
            row.bounds_center.z.to_bits(),
            row.bounds_radius.to_bits(),
            row.screen_space_error.to_bits(),
        ]);
    }
    if data.candidate_author_indices.len() != data.selected_clusters.len() {
        return Err("VG candidate provenance count differs from source catalog".into());
    }
    words[23] = u32::try_from(words.len()).map_err(|_| "VG candidate offset overflow")?;
    for (row, &(_, raw, total)) in data
        .selected_clusters
        .iter()
        .zip(&data.candidate_author_indices)
    {
        let source = extract
            .clusters
            .get(raw as usize)
            .ok_or("VG candidate raw source is absent")?;
        if source.entity != row.entity
            || source.cluster_id != row.cluster_id
            || source.page_id != row.page_id
            || source.lod_level != row.lod_level
        {
            return Err("VG candidate identity differs from its raw source".into());
        }
        words.extend_from_slice(&[
            row.instance_index.unwrap_or(u32::MAX),
            row.entity as u32,
            (row.entity >> 32) as u32,
            row.cluster_id,
            row.cluster_ordinal,
            row.page_id,
            u32::from(row.lod_level),
            match row.state {
                RenderVirtualGeometryExecutionState::Resident => 0,
                RenderVirtualGeometryExecutionState::PendingUpload => 1,
                RenderVirtualGeometryExecutionState::Missing => 2,
            },
            raw,
            total,
        ]);
    }
    words[24] = u32::try_from(words.len()).map_err(|_| "VG node offset overflow")?;
    for node in &extract.hierarchy_nodes {
        words.extend_from_slice(&[
            node.instance_index,
            node.node_id,
            node.child_base,
            node.child_count,
            node.cluster_start,
            node.cluster_count,
        ]);
    }
    words[25] = u32::try_from(words.len()).map_err(|_| "VG child offset overflow")?;
    words.extend_from_slice(&extract.hierarchy_child_ids);
    words[26] = u32::try_from(words.len()).map_err(|_| "VG page offset overflow")?;
    for page in &extract.pages {
        words.extend_from_slice(&[page.page_id, u32::from(page.resident)]);
    }
    Ok(words)
}

const DECISION_PROJECTION_SHADER: &str = r###"
struct Words { values:array<u32> }
@group(0) @binding(0) var<storage,read> decisions:Words;
@group(0) @binding(1) var<storage,read> candidates:Words;
@group(0) @binding(2) var<storage,read> clusters:Words;
@group(0) @binding(3) var<storage,read> vis:Words;
@group(0) @binding(4) var<storage,read> authority:Words;
@group(0) @binding(5) var<storage,read> submission:Words;
@group(0) @binding(6) var<storage,read_write> selected:Words;
@group(0) @binding(7) var<storage,read_write> hardware:Words;
@compute @workgroup_size(1)
fn cs_main(){
    let validated_physical_count=hardware.values[0];
    selected.values[0]=0xffffffffu;hardware.values[0]=0xffffffffu;
    let count=decisions.values[2];
    if decisions.values[0]!=0x56475331u || decisions.values[1]!=0u || decisions.values[3]!=2u || decisions.values[6]!=1u ||
       validated_physical_count!=submission.values[0] ||
       count>4096u || candidates.values[0]!=count || clusters.values[0]!=count || vis.values[0]!=count ||
       count>(arrayLength(&selected.values)-1u)/8u || count>(arrayLength(&hardware.values)-1u)/18u{return;}
    for(var row=0u;row<count;row++){
        let r=8u+row*28u;let instance=decisions.values[r];let cluster=decisions.values[r+3u];
        let page=decisions.values[r+5u];let lod=decisions.values[r+6u];let state=decisions.values[r+7u];
        if state>2u || candidates.values[row+1u]!=decisions.values[r+27u] || clusters.values[row+1u]!=cluster ||
           vis.values[1u+row*2u]!=((cluster&0xfffffu)|((page&0xfffu)<<20u)) ||
           vis.values[2u+row*2u]!=(((page>>12u)&0xffu)|((instance&0xffffu)<<8u)|(lod<<24u)|(state<<30u)){return;}
    }
    // Actual admitted raster rows must be a subset of this GPU selection, never CPU debug storage.
    let draws=submission.values[0];
    if draws>(arrayLength(&submission.values)-1u)/17u || draws>arrayLength(&authority.values)/14u{return;}
    for(var draw=0u;draw<draws;draw++){
        let a=draw*14u;let s=1u+draw*17u;
        if submission.values[s+16u]!=1u{return;}
        var found=false;
        for(var row=0u;row<count;row++){
            let r=8u+row*28u;
            if authority.values[a]==decisions.values[r] &&
               authority.values[a+1u]==decisions.values[r+1u] && authority.values[a+2u]==decisions.values[r+2u] &&
               authority.values[a+3u]==decisions.values[r+5u] && authority.values[a+4u]==decisions.values[r+3u] &&
               authority.values[a+5u]==decisions.values[r+4u] && authority.values[a+11u]==decisions.values[r+27u] {
                found=true;break;
            }
        }
        if !found{return;}
    }
    for(var row=0u;row<count;row++){
        let r=8u+row*28u;
        for(var k=0u;k<8u;k++){selected.values[1u+row*8u+k]=decisions.values[r+k];}
        for(var k=0u;k<18u;k++){hardware.values[1u+row*18u+k]=decisions.values[r+8u+k];}
    }
    selected.values[0]=count;hardware.values[0]=count;
}
"###;

pub(super) type TerminalGpuBufferView = (wgpu::Buffer, u64, Option<std::num::NonZeroU64>, u64);

pub(super) fn encode_seed_decision_projection(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    buffers: &[TerminalGpuBufferView],
) -> Result<(TerminalGpuBufferView, TerminalGpuBufferView), String> {
    let mut views = Vec::new();
    for name in [
        "virtual-geometry-cull-decisions",
        "virtual-geometry-visible-candidate-indices",
        "virtual-geometry-visible-clusters",
        "virtual-geometry-visbuffer64",
    ] {
        let b = gpu.require_buffer_binding(
            name,
            if name == "virtual-geometry-cull-decisions" {
                RenderGraphResourceAccessKind::Write
            } else {
                RenderGraphResourceAccessKind::Read
            },
        )?;
        views.push((
            b.buffer.clone(),
            b.offset,
            b.size,
            b.size
                .map(std::num::NonZeroU64::get)
                .unwrap_or_else(|| b.buffer.size().saturating_sub(b.offset)),
        ));
    }
    views.push(buffers[6].clone());
    views.push(buffers[7].clone());
    for name in [
        "virtual-geometry-execution-selected-clusters",
        "virtual-geometry-execution-hardware-metadata",
    ] {
        let b = gpu.require_buffer_binding(name, RenderGraphResourceAccessKind::Write)?;
        views.push((
            b.buffer.clone(),
            b.offset,
            b.size,
            b.size
                .map(std::num::NonZeroU64::get)
                .unwrap_or_else(|| b.buffer.size().saturating_sub(b.offset)),
        ));
    }
    let expected = [
        (8 + 4096 * 33) * 4,
        (1 + 4096) * 4,
        (1 + 4096) * 4,
        (1 + 4096 * 2) * 4,
        65536 * 14 * 4,
        (1 + 65536 * 17) * 4,
        (1 + 4096 * 8) * 4,
        (1 + 65536 * 18) * 4,
    ];
    if views
        .iter()
        .zip(expected)
        .any(|(view, size)| view.3 != size)
    {
        return Err("seed decision projection has a different physical buffer ABI".into());
    }
    let selected = views[6].clone();
    let hardware = views[7].clone();
    dispatch(
        gpu,
        "virtual-geometry-seed-decision-projection",
        DECISION_PROJECTION_SHADER,
        &views,
        &[true, true, true, true, true, true, false, false],
    )?;
    Ok((hardware, selected))
}

fn graph_view(
    gpu: &RenderPassGpuExecutionContext<'_>,
    name: &str,
    access: RenderGraphResourceAccessKind,
) -> Result<TerminalGpuBufferView, String> {
    let b = gpu.require_buffer_binding(name, access)?;
    let bytes = b
        .size
        .map(std::num::NonZeroU64::get)
        .unwrap_or_else(|| b.buffer.size().saturating_sub(b.offset));
    Ok((b.buffer.clone(), b.offset, b.size, bytes))
}

fn dispatch(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    label: &str,
    shader: &str,
    views: &[TerminalGpuBufferView],
    read_only: &[bool],
) -> Result<(), String> {
    if views.len() != read_only.len() || views.len() > 8 {
        return Err("VG gate storage layout is invalid".into());
    }
    if views
        .iter()
        .enumerate()
        .any(|(i, view)| views[..i].iter().any(|prior| prior.0 == view.0))
    {
        return Err("VG gate requires distinct physical storage bindings".into());
    }
    let native = gpu.native_context();
    let entries = read_only
        .iter()
        .enumerate()
        .map(|(binding, &read_only)| wgpu::BindGroupLayoutEntry {
            binding: binding as u32,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        })
        .collect::<Vec<_>>();
    let layout = native.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &entries,
    });
    let pipeline_layout = native.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let module = native.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader.into()),
    });
    let pipeline = native.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("cs_main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let entries = views
        .iter()
        .enumerate()
        .map(
            |(binding, (buffer, offset, size, _))| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: *offset,
                    size: *size,
                }),
            },
        )
        .collect::<Vec<_>>();
    let group = native.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: &layout,
        entries: &entries,
    });
    let mut pass = native
        .encoder
        .begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(label),
            timestamp_writes: None,
        });
    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
    Ok(())
}

pub(super) fn pre_raster_gate(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
    author_words: &[u32],
) -> Result<(), String> {
    use RenderGraphResourceAccessKind::Write;
    let mut views = Vec::new();
    for name in [
        "virtual-geometry-cull-decisions",
        "virtual-geometry-indexed-args",
        "virtual-geometry-indexed-sidecar",
        "virtual-geometry-indexed-status",
        "virtual-geometry-indirect-args",
        "virtual-geometry-indirect-authority",
    ] {
        views.push(graph_view(gpu, name, Write)?);
    }
    let expected = [
        (8 + 4096 * 33) * 4,
        65536 * 5 * 4,
        65536 * 32 * 4,
        65536 * 4,
        (1 + 4096 * 4) * 4,
        4096 * 14 * 4,
    ];
    if views.iter().zip(expected).any(|(v, size)| v.3 != size) {
        return Err("VG pre-raster gate buffer ABI differs".into());
    }
    let author = gpu
        .native_context()
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("virtual-geometry-pre-raster-original-author-facts"),
            contents: bytemuck::cast_slice(author_words),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let size = author.size();
    views.push((author, 0, std::num::NonZeroU64::new(size), size));
    dispatch(
        gpu,
        "virtual-geometry-pre-raster-admission",
        include_str!("selection_gate.wgsl"),
        &views,
        &[false, false, true, true, true, true, true],
    )
}

pub(super) const FINAL_PUBLICATION_SHADER: &str = r###"
struct Words {values:array<u32>}
@group(0) @binding(0) var<storage,read_write> decision:Words;
@group(0) @binding(1) var<storage,read_write> args:Words;
@group(0) @binding(2) var<storage,read_write> submission:Words;
@group(0) @binding(3) var<storage,read_write> selected:Words;
@group(0) @binding(4) var<storage,read_write> hardware:Words;
@compute @workgroup_size(1)
fn cs_main(){
    let valid=decision.values[0]==0x56475331u && decision.values[1]==0u &&
        decision.values[6]==1u && args.values[0]==submission.values[0] &&
        args.values[0]<=(arrayLength(&args.values)-1u)/4u &&
        submission.values[0]<=(arrayLength(&submission.values)-1u)/17u &&
        selected.values[0]==decision.values[2] &&
        hardware.values[0]==select(args.values[0],decision.values[2],decision.values[3]==2u);
    if !valid{
        decision.values[7]=2u;
        args.values[0]=0u;submission.values[0]=0u;
        selected.values[0]=0xffffffffu;hardware.values[0]=0xffffffffu;
        return;
    }
    decision.values[7]=1u;
}
"###;

pub(super) fn final_publication_gate(
    gpu: &mut RenderPassGpuExecutionContext<'_>,
) -> Result<(), String> {
    use RenderGraphResourceAccessKind::Write;
    let mut views = Vec::new();
    for name in [
        "virtual-geometry-cull-decisions",
        "virtual-geometry-execution-args",
        "virtual-geometry-execution-submission",
        "virtual-geometry-execution-selected-clusters",
        "virtual-geometry-execution-hardware-metadata",
    ] {
        views.push(graph_view(gpu, name, Write)?);
    }
    dispatch(
        gpu,
        "virtual-geometry-final-publication",
        FINAL_PUBLICATION_SHADER,
        &views,
        &[false; 5],
    )
}
