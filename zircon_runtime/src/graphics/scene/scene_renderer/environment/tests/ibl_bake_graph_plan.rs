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
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
    );
    assert_eq!(
        plan.passes[8].workload.pipeline_label,
        IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL
    );
    assert_eq!(
        plan.passes[8].workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
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
fn single_mip_pmrem_graph_plan_dispatches_one_cube_average_workgroup() {
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        16,
        5,
    )
    .with_pmrem_layout(1, 1)
    .with_required_contents(IblBakeArtifactContents::PMREM);
    let mut builder = RenderGraphBuilder::new("ibl-bake-single-mip-workload-test");

    let plan = append_ibl_bake_artifact_graph_plan(&mut builder, &request)
        .expect("IBL bake graph plan should append");

    assert_eq!(plan.passes.len(), 1);
    assert_eq!(
        plan.passes[0].workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
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
            assert_eq!(desc.depth, 1);
            assert_eq!(desc.array_layers, 6);
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
