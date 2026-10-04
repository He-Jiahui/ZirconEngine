use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactRequest, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
use crate::render_graph::{
    ExternalResource, QueueLane, RenderGraphBuilder, RenderGraphComputeWorkload, RenderGraphError,
    RenderGraphExternalResourceBinding, RenderGraphResource, RenderGraphResourceUsageFlags,
    RenderPassId, RgBufferHandle, RgTextureHandle,
};
use crate::rhi::{
    BufferDesc, BufferUsage, TextureDesc, TextureDimension, TextureFormat, TextureUsage,
};

pub(in crate::graphics) const IBL_BAKE_SOURCE_CUBEMAP_RESOURCE: &str =
    "environment.ibl.source_cubemap";
pub(in crate::graphics) const IBL_BAKE_PMREM_RESOURCE: &str = "environment.ibl.pmrem";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_SH9_RESOURCE: &str =
    "environment.ibl.irradiance_sh9";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_CUBE_RESOURCE: &str =
    "environment.ibl.irradiance_cube";

pub(in crate::graphics) const IBL_BAKE_PMREM_PASS: &str = "env.ibl_prefilter";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_SH9_PASS: &str = "env.ibl_irradiance_sh";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_CUBE_PASS: &str = "env.ibl_irradiance_cube";

pub(in crate::graphics) const IBL_BAKE_PMREM_EXECUTOR_ID: &str = "environment.ibl_prefilter";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID: &str =
    "environment.ibl_irradiance_sh";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID: &str =
    "environment.ibl_irradiance_cube";

pub(in crate::graphics) const IBL_BAKE_PMREM_PIPELINE_LABEL: &str = "zircon-env-ibl-prefilter";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL: &str =
    "zircon-env-ibl-irradiance-sh";
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL: &str =
    "zircon-env-ibl-irradiance-cube";

const IBL_BAKE_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
const IBL_BAKE_CUBE_FACE_COUNT: u32 = 6;
// The SH9 shader reduces a fixed 8x8 sample lattice; it does not scale with cubemap resolution.
pub(in crate::graphics) const IBL_BAKE_IRRADIANCE_SH9_DISPATCH_GROUPS: [u32; 3] = [1, 1, 1];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum IblBakeGraphPassKind {
    Pmrem { mip_level: u32 },
    IrradianceSh9,
    IrradianceCube,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct IblBakeGraphPass {
    pub kind: IblBakeGraphPassKind,
    pub pass_id: RenderPassId,
    pub output: RenderGraphResource,
    pub workload: RenderGraphComputeWorkload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct IblBakeGraphResources {
    pub source_cubemap: ExternalResource,
    pub pmrem: Option<RgTextureHandle>,
    pub irradiance_sh9: Option<RgBufferHandle>,
    pub irradiance_cube: Option<RgTextureHandle>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct IblBakeGraphPlan {
    pub resources: IblBakeGraphResources,
    pub passes: Vec<IblBakeGraphPass>,
}

pub(in crate::graphics) fn append_ibl_bake_artifact_graph_plan(
    builder: &mut RenderGraphBuilder,
    request: &IblBakeArtifactRequest,
) -> Result<IblBakeGraphPlan, RenderGraphError> {
    let source_cubemap = builder.import_external_resource_with_usage_and_binding(
        IBL_BAKE_SOURCE_CUBEMAP_RESOURCE,
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let mut resources = IblBakeGraphResources {
        source_cubemap,
        pmrem: None,
        irradiance_sh9: None,
        irradiance_cube: None,
    };
    let mut passes = Vec::new();
    let contents = request.required_contents();

    if contents.contains(IblBakeArtifactContents::PMREM) {
        let texture = builder.create_texture(pmrem_texture_desc(request));
        builder.mark_readback(RenderGraphResource::TransientTexture(texture))?;
        let mut previous_pmrem_pass = None;
        for mip_level in 0..request.pmrem_mip_count() {
            let workload = pmrem_workload(request, mip_level);
            let pass = builder.add_pass_with_executor(
                ibl_bake_pmrem_pass_name(mip_level),
                QueueLane::AsyncCompute,
                Some(IBL_BAKE_PMREM_EXECUTOR_ID),
            );
            builder.set_compute_workload(pass, workload.clone())?;
            builder.read_external(pass, source_cubemap)?;
            builder.write_storage_texture(pass, texture)?;
            if let Some(previous_pmrem_pass) = previous_pmrem_pass {
                builder.add_dependency(previous_pmrem_pass, pass)?;
            }
            previous_pmrem_pass = Some(pass);
            passes.push(IblBakeGraphPass {
                kind: IblBakeGraphPassKind::Pmrem { mip_level },
                pass_id: pass,
                output: RenderGraphResource::TransientTexture(texture),
                workload,
            });
        }
        resources.pmrem = Some(texture);
    }

    if contents.contains(IblBakeArtifactContents::SH9) {
        let buffer = builder.create_buffer(sh9_buffer_desc());
        builder.mark_readback(RenderGraphResource::TransientBuffer(buffer))?;
        let workload = irradiance_sh9_workload();
        let pass = builder.add_pass_with_executor(
            IBL_BAKE_IRRADIANCE_SH9_PASS,
            QueueLane::AsyncCompute,
            Some(IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID),
        );
        builder.set_compute_workload(pass, workload.clone())?;
        builder.read_external(pass, source_cubemap)?;
        builder.write_buffer(pass, buffer)?;
        resources.irradiance_sh9 = Some(buffer);
        passes.push(IblBakeGraphPass {
            kind: IblBakeGraphPassKind::IrradianceSh9,
            pass_id: pass,
            output: RenderGraphResource::TransientBuffer(buffer),
            workload,
        });
    }

    if contents.contains(IblBakeArtifactContents::IEM) {
        let texture = builder.create_texture(irradiance_cube_texture_desc());
        builder.mark_readback(RenderGraphResource::TransientTexture(texture))?;
        let workload = irradiance_cube_workload();
        let pass = builder.add_pass_with_executor(
            IBL_BAKE_IRRADIANCE_CUBE_PASS,
            QueueLane::AsyncCompute,
            Some(IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID),
        );
        builder.set_compute_workload(pass, workload.clone())?;
        builder.read_external(pass, source_cubemap)?;
        builder.write_storage_texture(pass, texture)?;
        resources.irradiance_cube = Some(texture);
        passes.push(IblBakeGraphPass {
            kind: IblBakeGraphPassKind::IrradianceCube,
            pass_id: pass,
            output: RenderGraphResource::TransientTexture(texture),
            workload,
        });
    }

    Ok(IblBakeGraphPlan { resources, passes })
}

pub(in crate::graphics) fn ibl_bake_pmrem_pass_name(mip_level: u32) -> String {
    format!("{IBL_BAKE_PMREM_PASS}.mip{mip_level}")
}

pub(in crate::graphics) fn ibl_bake_pmrem_mip_from_pass_name(pass_name: &str) -> Option<u32> {
    pass_name
        .strip_prefix(IBL_BAKE_PMREM_PASS)
        .and_then(|suffix| suffix.strip_prefix(".mip"))
        .and_then(|mip| mip.parse::<u32>().ok())
}

fn pmrem_texture_desc(request: &IblBakeArtifactRequest) -> TextureDesc {
    TextureDesc::new(
        IBL_BAKE_PMREM_RESOURCE,
        request.pmrem_face_size(),
        request.pmrem_face_size(),
        TextureFormat::Rgba16Float,
        TextureUsage::STORAGE | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
    )
    .with_dimension(TextureDimension::Cube)
    .with_array_layers(IBL_BAKE_CUBE_FACE_COUNT)
    .with_mip_levels(request.pmrem_mip_count())
}

fn sh9_buffer_desc() -> BufferDesc {
    BufferDesc::new(
        IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
        IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    )
}

fn irradiance_cube_texture_desc() -> TextureDesc {
    TextureDesc::new(
        IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        TextureFormat::Rgba16Float,
        TextureUsage::STORAGE | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
    )
    .with_dimension(TextureDimension::Cube)
    .with_array_layers(IBL_BAKE_CUBE_FACE_COUNT)
}

fn pmrem_workload(request: &IblBakeArtifactRequest, mip_level: u32) -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::fixed(
        IBL_BAKE_PMREM_PIPELINE_LABEL,
        IBL_BAKE_WORKGROUP_SIZE,
        ibl_bake_pmrem_dispatch_groups(
            request.pmrem_face_size(),
            request.pmrem_mip_count(),
            mip_level,
        ),
    )
}

fn irradiance_sh9_workload() -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::fixed(
        IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL,
        IBL_BAKE_WORKGROUP_SIZE,
        IBL_BAKE_IRRADIANCE_SH9_DISPATCH_GROUPS,
    )
}

fn irradiance_cube_workload() -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::fixed(
        IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL,
        IBL_BAKE_WORKGROUP_SIZE,
        [
            div_ceil(
                SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
                IBL_BAKE_WORKGROUP_SIZE[0],
            ),
            div_ceil(
                SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
                IBL_BAKE_WORKGROUP_SIZE[1],
            ),
            IBL_BAKE_CUBE_FACE_COUNT,
        ],
    )
}

pub(in crate::graphics) const fn ibl_bake_pmrem_dispatch_groups(
    face_size: u32,
    mip_count: u32,
    mip_level: u32,
) -> [u32; 3] {
    let mip_size = pmrem_mip_size(face_size, mip_level);
    [
        div_ceil(mip_size, IBL_BAKE_WORKGROUP_SIZE[0]),
        div_ceil(mip_size, IBL_BAKE_WORKGROUP_SIZE[1]),
        if ibl_bake_terminal_pmrem_average_mip(face_size, mip_count, mip_level) {
            1
        } else {
            IBL_BAKE_CUBE_FACE_COUNT
        },
    ]
}

pub(in crate::graphics) const fn ibl_bake_pmrem_dispatch_groups_for_face_range(
    face_size: u32,
    mip_count: u32,
    mip_level: u32,
    first_face: u32,
    face_count: u32,
) -> Option<[u32; 3]> {
    if face_count == 0
        || first_face >= IBL_BAKE_CUBE_FACE_COUNT
        || face_count > IBL_BAKE_CUBE_FACE_COUNT - first_face
        || mip_level >= mip_count
    {
        return None;
    }

    let mut groups = ibl_bake_pmrem_dispatch_groups(face_size, mip_count, mip_level);
    if first_face != 0
        || face_count != IBL_BAKE_CUBE_FACE_COUNT
        || !ibl_bake_terminal_pmrem_average_mip(face_size, mip_count, mip_level)
    {
        groups[2] = face_count;
    }
    Some(groups)
}

pub(in crate::graphics) const fn ibl_bake_terminal_pmrem_average_mip(
    face_size: u32,
    mip_count: u32,
    mip_level: u32,
) -> bool {
    mip_level.saturating_add(1) >= mip_count && pmrem_mip_size(face_size, mip_level) == 1
}

const fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}

const fn pmrem_mip_size(face_size: u32, mip_level: u32) -> u32 {
    let shifted = face_size >> mip_level;
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

#[cfg(test)]
#[path = "tests/ibl_bake_graph_plan.rs"]
mod tests;
