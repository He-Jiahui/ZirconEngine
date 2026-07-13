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
const CUBE_FACE_COUNT: u32 = 6;

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
    .with_depth(CUBE_FACE_COUNT)
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
    .with_depth(CUBE_FACE_COUNT)
}

fn pmrem_workload(request: &IblBakeArtifactRequest, mip_level: u32) -> RenderGraphComputeWorkload {
    let mip_size = pmrem_mip_size(request.pmrem_face_size(), mip_level);
    RenderGraphComputeWorkload::fixed(
        IBL_BAKE_PMREM_PIPELINE_LABEL,
        IBL_BAKE_WORKGROUP_SIZE,
        [
            div_ceil(mip_size, IBL_BAKE_WORKGROUP_SIZE[0]),
            div_ceil(mip_size, IBL_BAKE_WORKGROUP_SIZE[1]),
            CUBE_FACE_COUNT,
        ],
    )
}

fn irradiance_sh9_workload() -> RenderGraphComputeWorkload {
    RenderGraphComputeWorkload::fixed(
        IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL,
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
            CUBE_FACE_COUNT,
        ],
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
            CUBE_FACE_COUNT,
        ],
    )
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
mod tests {
    use super::*;
    use crate::core::framework::render::ProceduralSkyParams;
    use crate::render_graph::{
        CompiledRenderGraph, CompiledRenderPass, RenderGraphComputeDispatchExtent,
        RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
    };

    #[test]
    fn ibl_bake_graph_plan_declares_pmrem_sh9_iem_passes_in_artifact_order() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            128,
            8,
        )
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
        let mut builder = RenderGraphBuilder::new("ibl-bake-test");

        let plan = append_ibl_bake_artifact_graph_plan(&mut builder, &request)
            .expect("IBL bake graph plan should append");
        let graph = builder.compile().expect("graph should compile");

        let expected_pmrem = (0..8)
            .map(|mip_level| IblBakeGraphPassKind::Pmrem { mip_level })
            .collect::<Vec<_>>();
        assert_eq!(
            plan.passes
                .iter()
                .take(8)
                .map(|pass| pass.kind)
                .collect::<Vec<_>>(),
            expected_pmrem
        );
        assert_eq!(plan.passes[8].kind, IblBakeGraphPassKind::IrradianceSh9);
        assert_eq!(plan.passes[9].kind, IblBakeGraphPassKind::IrradianceCube);
        assert_eq!(graph.passes().len(), 10);
        for mip_level in 0..8 {
            let pass_name = ibl_bake_pmrem_pass_name(mip_level);
            assert!(
                graph.passes().iter().any(|pass| pass.name == pass_name),
                "compiled graph should contain PMREM mip pass `{pass_name}`"
            );
        }
        assert!(graph
            .passes()
            .iter()
            .any(|pass| pass.name == IBL_BAKE_IRRADIANCE_SH9_PASS));
        assert!(graph
            .passes()
            .iter()
            .any(|pass| pass.name == IBL_BAKE_IRRADIANCE_CUBE_PASS));
        for mip_level in 1..8 {
            let previous = compiled_pass_by_name(&graph, &ibl_bake_pmrem_pass_name(mip_level - 1));
            let current = compiled_pass_by_name(&graph, &ibl_bake_pmrem_pass_name(mip_level));
            assert!(
                current.dependencies.contains(&previous.id),
                "PMREM mip {mip_level} should depend on the previous mip pass"
            );
        }
        assert!(
            plan.resources.pmrem.is_some()
                && plan.resources.irradiance_sh9.is_some()
                && plan.resources.irradiance_cube.is_some()
        );
    }

    #[test]
    fn ibl_bake_graph_plan_records_expected_fixed_workloads() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            128,
            8,
        )
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
        let mut builder = RenderGraphBuilder::new("ibl-bake-workload-test");

        let plan = append_ibl_bake_artifact_graph_plan(&mut builder, &request)
            .expect("IBL bake graph plan should append");

        for mip_level in 0..8 {
            assert_eq!(
                plan.passes[mip_level as usize].workload.pipeline_label,
                IBL_BAKE_PMREM_PIPELINE_LABEL
            );
            assert_eq!(
                plan.passes[mip_level as usize].workload.workgroup_size,
                [8, 8, 1]
            );
        }
        assert_eq!(
            plan.passes[0].workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::Fixed([16, 16, 6])
        );
        assert_eq!(
            plan.passes[7].workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::Fixed([1, 1, 6])
        );
        assert_eq!(
            plan.passes[8].workload.pipeline_label,
            IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL
        );
        assert_eq!(
            plan.passes[8].workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::Fixed([4, 4, 6])
        );
        assert_eq!(
            plan.passes[9].workload.pipeline_label,
            IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL
        );
        assert_eq!(
            plan.passes[9].workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::Fixed([4, 4, 6])
        );
    }

    #[test]
    fn ibl_bake_graph_plan_declares_storage_outputs_and_readback_roots() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            64,
            7,
        )
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
        let mut builder = RenderGraphBuilder::new("ibl-bake-resource-test");

        append_ibl_bake_artifact_graph_plan(&mut builder, &request)
            .expect("IBL bake graph plan should append");
        let graph = builder.compile().expect("graph should compile");

        let pmrem = graph
            .resource_lifetime_by_name(IBL_BAKE_PMREM_RESOURCE)
            .expect("PMREM resource lifetime");
        let sh9 = graph
            .resource_lifetime_by_name(IBL_BAKE_IRRADIANCE_SH9_RESOURCE)
            .expect("SH9 resource lifetime");
        let iem = graph
            .resource_lifetime_by_name(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE)
            .expect("IEM resource lifetime");
        let source = graph
            .resource_lifetime_by_name(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE)
            .expect("source cubemap resource lifetime");

        assert_eq!(source.kind, RenderGraphResourceKind::External);
        assert!(source.external_binding.is_required());
        assert!(pmrem.usage.readback);
        assert!(sh9.usage.readback);
        assert!(iem.usage.readback);
        match &pmrem.desc {
            RenderGraphResourceDesc::Texture(desc) => {
                assert_eq!(desc.dimension, TextureDimension::Cube);
                assert_eq!(desc.depth, 6);
                assert_eq!(desc.width, 128);
                assert_eq!(desc.height, 128);
                assert_eq!(desc.mip_levels, 8);
                assert!(desc.usage.contains(TextureUsage::STORAGE));
                assert!(desc.usage.contains(TextureUsage::COPY_SRC));
            }
            other => panic!("expected PMREM texture desc, got {other:?}"),
        }
        match &sh9.desc {
            RenderGraphResourceDesc::Buffer(desc) => {
                assert_eq!(desc.size_bytes, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64);
                assert!(desc.usage.contains(BufferUsage::STORAGE));
                assert!(desc.usage.contains(BufferUsage::COPY_SRC));
            }
            other => panic!("expected SH9 buffer desc, got {other:?}"),
        }

        for pass in graph.passes() {
            assert_eq!(pass.queue, QueueLane::AsyncCompute);
            assert!(
                pass.resources.iter().any(|resource| {
                    resource.name == IBL_BAKE_SOURCE_CUBEMAP_RESOURCE
                        && resource.access == RenderGraphResourceAccessKind::Read
                }),
                "IBL bake pass `{}` must read source cubemap",
                pass.name
            );
        }
    }

    #[test]
    fn ibl_bake_graph_plan_only_declares_requested_content_outputs() {
        let request = IblBakeArtifactRequest::new(
            ProceduralSkyParams::default_gradient().ibl_bake_key(),
            32,
            6,
        )
        .with_required_contents(IblBakeArtifactContents::SH9);
        let mut builder = RenderGraphBuilder::new("ibl-bake-sh9-only-test");

        let plan = append_ibl_bake_artifact_graph_plan(&mut builder, &request)
            .expect("IBL bake graph plan should append");
        let graph = builder.compile().expect("graph should compile");

        assert_eq!(plan.passes.len(), 1);
        assert_eq!(plan.passes[0].kind, IblBakeGraphPassKind::IrradianceSh9);
        assert!(plan.resources.pmrem.is_none());
        assert!(plan.resources.irradiance_sh9.is_some());
        assert!(plan.resources.irradiance_cube.is_none());
        assert!(graph
            .resource_lifetime_by_name(IBL_BAKE_PMREM_RESOURCE)
            .is_none());
        assert!(graph
            .resource_lifetime_by_name(IBL_BAKE_IRRADIANCE_CUBE_RESOURCE)
            .is_none());
    }

    fn compiled_pass_by_name<'a>(
        graph: &'a CompiledRenderGraph,
        pass_name: &str,
    ) -> &'a CompiledRenderPass {
        graph
            .passes()
            .iter()
            .find(|pass| pass.name == pass_name)
            .unwrap_or_else(|| panic!("compiled graph should contain pass `{pass_name}`"))
    }
}
