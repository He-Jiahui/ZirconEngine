use crate::core::framework::render::IblBakeArtifactRequest;
use crate::render_graph::{
    ExternalResource, QueueLane, RenderGraphBuilder, RenderGraphComputeWorkload, RenderGraphError,
    RenderGraphExternalResourceBinding, RenderGraphResourceUsageFlags, RenderPassId,
};

use super::realtime_ibl_time_slice::{
    CubeFaceRange, IblRealtimeBufferSlot, RealtimeIblFrameBatch, RealtimeIblOperation,
    RealtimeIblPrefilterDispatchSlice,
};

pub(in crate::graphics) const REALTIME_IBL_CAPTURE_SKY_EXECUTOR_ID: &str =
    "environment.realtime_ibl.capture_sky";
pub(in crate::graphics) const REALTIME_IBL_CAPTURE_CLOUD_EXECUTOR_ID: &str =
    "environment.realtime_ibl.capture_cloud";
pub(in crate::graphics) const REALTIME_IBL_GENERATE_SOURCE_MIPS_EXECUTOR_ID: &str =
    "environment.realtime_ibl.generate_source_mips";
pub(in crate::graphics) const REALTIME_IBL_PREFILTER_EXECUTOR_ID: &str =
    "environment.realtime_ibl.prefilter";
pub(in crate::graphics) const REALTIME_IBL_PROJECT_SH9_EXECUTOR_ID: &str =
    "environment.realtime_ibl.project_sh9";

const REALTIME_IBL_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
const CUBE_FACE_COUNT: u32 = 6;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblGraphExternalResource {
    pub name: String,
    pub handle: ExternalResource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblGraphTextureResources {
    pub sampled: RealtimeIblGraphExternalResource,
    pub sampled_mips: Vec<RealtimeIblGraphExternalResource>,
    pub storage_mips: Vec<RealtimeIblGraphExternalResource>,
}

impl RealtimeIblGraphTextureResources {
    fn resources(&self) -> impl Iterator<Item = &RealtimeIblGraphExternalResource> {
        std::iter::once(&self.sampled)
            .chain(self.sampled_mips.iter())
            .chain(self.storage_mips.iter())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblGraphSlotResources {
    pub slot: IblRealtimeBufferSlot,
    pub source: RealtimeIblGraphTextureResources,
    pub pmrem: RealtimeIblGraphTextureResources,
    pub sh9: RealtimeIblGraphExternalResource,
}

impl RealtimeIblGraphSlotResources {
    pub(in crate::graphics) fn resources(&self) -> Vec<&RealtimeIblGraphExternalResource> {
        self.source
            .resources()
            .chain(self.pmrem.resources())
            .chain(std::iter::once(&self.sh9))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum RealtimeIblGraphPassKind {
    CaptureSky(CubeFaceRange),
    CaptureCloud(CubeFaceRange),
    GenerateSourceMip { mip_level: u32 },
    Prefilter(RealtimeIblPrefilterDispatchSlice),
    ProjectDiffuseSh9,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblGraphPass {
    pub kind: RealtimeIblGraphPassKind,
    pub pass_id: RenderPassId,
    pub workload: RenderGraphComputeWorkload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct RealtimeIblGraphPlan {
    pub ready: RealtimeIblGraphSlotResources,
    pub work: RealtimeIblGraphSlotResources,
    pub passes: Vec<RealtimeIblGraphPass>,
}

pub(in crate::graphics) fn append_realtime_ibl_graph_plan(
    builder: &mut RenderGraphBuilder,
    request: &IblBakeArtifactRequest,
    batch: &RealtimeIblFrameBatch,
) -> Result<RealtimeIblGraphPlan, RenderGraphError> {
    let slot_a = import_slot(builder, request, IblRealtimeBufferSlot::A);
    let slot_b = import_slot(builder, request, IblRealtimeBufferSlot::B);
    let ready = select_slot(&slot_a, &slot_b, batch.ready_slot()).clone();
    let work = select_slot(&slot_a, &slot_b, batch.work_slot()).clone();
    let mut passes = Vec::new();
    let mut previous = None;

    for operation in batch.operations() {
        match *operation {
            RealtimeIblOperation::CaptureSky(faces) => {
                let workload = face_workload(request.source_face_size(), faces);
                let pass = add_pass(
                    builder,
                    capture_pass_name("sky", faces),
                    REALTIME_IBL_CAPTURE_SKY_EXECUTOR_ID,
                    workload.clone(),
                    previous,
                )?;
                builder.write_storage_external(pass, work.source.storage_mips[0].handle)?;
                previous = Some(pass);
                passes.push(RealtimeIblGraphPass {
                    kind: RealtimeIblGraphPassKind::CaptureSky(faces),
                    pass_id: pass,
                    workload,
                });
            }
            RealtimeIblOperation::CaptureCloud(faces) => {
                let workload = face_workload(request.source_face_size(), faces);
                let pass = add_pass(
                    builder,
                    capture_pass_name("cloud", faces),
                    REALTIME_IBL_CAPTURE_CLOUD_EXECUTOR_ID,
                    workload.clone(),
                    previous,
                )?;
                builder.write_storage_external(pass, work.source.storage_mips[0].handle)?;
                previous = Some(pass);
                passes.push(RealtimeIblGraphPass {
                    kind: RealtimeIblGraphPassKind::CaptureCloud(faces),
                    pass_id: pass,
                    workload,
                });
            }
            RealtimeIblOperation::GenerateSourceMips => {
                for mip_level in 1..request.source_mip_count() {
                    let mip_size = mip_dimension(request.source_face_size(), mip_level);
                    let workload = RenderGraphComputeWorkload::fixed(
                        "zircon-env-realtime-ibl-generate-source-mips",
                        REALTIME_IBL_WORKGROUP_SIZE,
                        [
                            div_ceil(mip_size, REALTIME_IBL_WORKGROUP_SIZE[0]),
                            div_ceil(mip_size, REALTIME_IBL_WORKGROUP_SIZE[1]),
                            CUBE_FACE_COUNT,
                        ],
                    );
                    let pass = add_pass(
                        builder,
                        format!("env.realtime_ibl.generate_source_mip.mip{mip_level}"),
                        REALTIME_IBL_GENERATE_SOURCE_MIPS_EXECUTOR_ID,
                        workload.clone(),
                        previous,
                    )?;
                    builder.read_external(
                        pass,
                        work.source.sampled_mips[(mip_level - 1) as usize].handle,
                    )?;
                    builder.write_storage_external(
                        pass,
                        work.source.storage_mips[mip_level as usize].handle,
                    )?;
                    previous = Some(pass);
                    passes.push(RealtimeIblGraphPass {
                        kind: RealtimeIblGraphPassKind::GenerateSourceMip { mip_level },
                        pass_id: pass,
                        workload,
                    });
                }
            }
            RealtimeIblOperation::Prefilter { mips, faces } => {
                for mip_level in mips.first..mips.first.saturating_add(mips.count) {
                    let slice = RealtimeIblPrefilterDispatchSlice {
                        mip_level,
                        first_face: faces.first,
                        face_count: faces.count,
                    };
                    let mip_size = mip_dimension(request.pmrem_face_size(), u32::from(mip_level));
                    let workload = RenderGraphComputeWorkload::fixed(
                        "zircon-env-realtime-ibl-prefilter",
                        REALTIME_IBL_WORKGROUP_SIZE,
                        [
                            div_ceil(mip_size, REALTIME_IBL_WORKGROUP_SIZE[0]),
                            div_ceil(mip_size, REALTIME_IBL_WORKGROUP_SIZE[1]),
                            u32::from(faces.count),
                        ],
                    );
                    let pass = add_pass(
                        builder,
                        prefilter_pass_name(slice),
                        REALTIME_IBL_PREFILTER_EXECUTOR_ID,
                        workload.clone(),
                        previous,
                    )?;
                    builder.read_external(pass, work.source.sampled.handle)?;
                    builder.write_storage_external(
                        pass,
                        work.pmrem.storage_mips[usize::from(mip_level)].handle,
                    )?;
                    previous = Some(pass);
                    passes.push(RealtimeIblGraphPass {
                        kind: RealtimeIblGraphPassKind::Prefilter(slice),
                        pass_id: pass,
                        workload,
                    });
                }
            }
            RealtimeIblOperation::ProjectDiffuseSh9 => {
                let workload = RenderGraphComputeWorkload::fixed(
                    "zircon-env-realtime-ibl-project-sh9",
                    REALTIME_IBL_WORKGROUP_SIZE,
                    [4, 4, CUBE_FACE_COUNT],
                );
                let pass = add_pass(
                    builder,
                    "env.realtime_ibl.project_sh9".to_string(),
                    REALTIME_IBL_PROJECT_SH9_EXECUTOR_ID,
                    workload.clone(),
                    previous,
                )?;
                builder.read_external(pass, work.source.sampled.handle)?;
                builder.write_storage_external(pass, work.sh9.handle)?;
                previous = Some(pass);
                passes.push(RealtimeIblGraphPass {
                    kind: RealtimeIblGraphPassKind::ProjectDiffuseSh9,
                    pass_id: pass,
                    workload,
                });
            }
        }
    }

    Ok(RealtimeIblGraphPlan {
        ready,
        work,
        passes,
    })
}

fn import_slot(
    builder: &mut RenderGraphBuilder,
    request: &IblBakeArtifactRequest,
    slot: IblRealtimeBufferSlot,
) -> RealtimeIblGraphSlotResources {
    let prefix = slot_resource_prefix(slot);
    let source = import_texture_resources(
        builder,
        format!("{prefix}.source"),
        request.source_mip_count(),
    );
    let pmrem = import_texture_resources(
        builder,
        format!("{prefix}.pmrem"),
        request.pmrem_mip_count(),
    );
    let sh9 = import_external_buffer(builder, format!("{prefix}.sh9"));
    RealtimeIblGraphSlotResources {
        slot,
        source,
        pmrem,
        sh9,
    }
}

fn import_texture_resources(
    builder: &mut RenderGraphBuilder,
    prefix: String,
    mip_count: u32,
) -> RealtimeIblGraphTextureResources {
    let sampled = import_external_texture(builder, format!("{prefix}.sampled"));
    let sampled_mips = (0..mip_count)
        .map(|mip_level| {
            import_external_texture(builder, format!("{prefix}.sampled.mip{mip_level}"))
        })
        .collect();
    let storage_mips = (0..mip_count)
        .map(|mip_level| {
            import_external_texture(builder, format!("{prefix}.storage.mip{mip_level}"))
        })
        .collect();
    RealtimeIblGraphTextureResources {
        sampled,
        sampled_mips,
        storage_mips,
    }
}

fn import_external_texture(
    builder: &mut RenderGraphBuilder,
    name: String,
) -> RealtimeIblGraphExternalResource {
    let handle = builder.import_external_resource_with_usage_and_binding(
        name.clone(),
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_texture(),
    );
    RealtimeIblGraphExternalResource { name, handle }
}

fn import_external_buffer(
    builder: &mut RenderGraphBuilder,
    name: String,
) -> RealtimeIblGraphExternalResource {
    let handle = builder.import_external_resource_with_usage_and_binding(
        name.clone(),
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    RealtimeIblGraphExternalResource { name, handle }
}

fn select_slot<'a>(
    slot_a: &'a RealtimeIblGraphSlotResources,
    slot_b: &'a RealtimeIblGraphSlotResources,
    selected: IblRealtimeBufferSlot,
) -> &'a RealtimeIblGraphSlotResources {
    match selected {
        IblRealtimeBufferSlot::A => slot_a,
        IblRealtimeBufferSlot::B => slot_b,
    }
}

fn add_pass(
    builder: &mut RenderGraphBuilder,
    name: String,
    executor: &'static str,
    workload: RenderGraphComputeWorkload,
    previous: Option<RenderPassId>,
) -> Result<RenderPassId, RenderGraphError> {
    let pass = builder.add_pass_with_executor(name, QueueLane::AsyncCompute, Some(executor));
    builder.set_compute_workload(pass, workload)?;
    if let Some(previous) = previous {
        builder.add_dependency(previous, pass)?;
    }
    Ok(pass)
}

fn face_workload(face_size: u32, faces: CubeFaceRange) -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::fixed(
        "zircon-env-realtime-ibl-capture",
        REALTIME_IBL_WORKGROUP_SIZE,
        [
            div_ceil(face_size, REALTIME_IBL_WORKGROUP_SIZE[0]),
            div_ceil(face_size, REALTIME_IBL_WORKGROUP_SIZE[1]),
            u32::from(faces.count),
        ],
    )
}

fn capture_pass_name(kind: &str, faces: CubeFaceRange) -> String {
    format!(
        "env.realtime_ibl.capture_{kind}.faces{}_{}",
        faces.first, faces.count
    )
}

fn prefilter_pass_name(slice: RealtimeIblPrefilterDispatchSlice) -> String {
    format!(
        "env.realtime_ibl.prefilter.mip{}.faces{}_{}",
        slice.mip_level, slice.first_face, slice.face_count
    )
}

const fn slot_resource_prefix(slot: IblRealtimeBufferSlot) -> &'static str {
    match slot {
        IblRealtimeBufferSlot::A => "environment.realtime_ibl.a",
        IblRealtimeBufferSlot::B => "environment.realtime_ibl.b",
    }
}

const fn mip_dimension(base_size: u32, mip_level: u32) -> u32 {
    let shifted = base_size >> mip_level;
    if shifted == 0 { 1 } else { shifted }
}

const fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}

#[cfg(test)]
mod tests;
