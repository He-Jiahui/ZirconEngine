use crate::core::framework::render::{
    build_source_cubemap_from_equirect, EnvironmentExtract, IblBakeArtifactContents,
    IblBakeArtifactRequest, ProceduralSkyParams, SourceCubemapEnvironment,
};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeWorkloadDispatchContext, RenderGraphExecutionRecord, RenderPassExecutorId,
};
use crate::render_graph::{QueueLane, RenderGraphBuilder};

use super::super::ibl_bake_graph_plan::{
    append_ibl_bake_artifact_graph_plan, ibl_bake_pmrem_pass_name,
    IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_CUBE_PASS,
    IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL, IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
    IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_SH9_PASS,
    IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL, IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
    IBL_BAKE_PMREM_EXECUTOR_ID, IBL_BAKE_PMREM_PIPELINE_LABEL, IBL_BAKE_PMREM_RESOURCE,
};
use super::*;

#[test]
fn ibl_bake_compute_dispatch_records_match_graph_plan_workloads() {
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        128,
        8,
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let mut builder = RenderGraphBuilder::new("ibl-bake-executor-plan-test");
    append_ibl_bake_artifact_graph_plan(&mut builder, &request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("graph should compile");
    let mut record = RenderGraphExecutionRecord::default();
    let dispatch_context = RenderGraphComputeWorkloadDispatchContext::new([1, 1], [1, 1], 0);

    for pass in graph.passes() {
        let context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                pass.resources.clone(),
            )
            .with_resource_resolver(&graph, pass.id);
        let (output, pipeline_label) = match context.executor_id.as_str() {
            IBL_BAKE_PMREM_EXECUTOR_ID => (IBL_BAKE_PMREM_RESOURCE, IBL_BAKE_PMREM_PIPELINE_LABEL),
            IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID => (
                IBL_BAKE_IRRADIANCE_SH9_RESOURCE,
                IBL_BAKE_IRRADIANCE_SH9_PIPELINE_LABEL,
            ),
            IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID => (
                IBL_BAKE_IRRADIANCE_CUBE_RESOURCE,
                IBL_BAKE_IRRADIANCE_CUBE_PIPELINE_LABEL,
            ),
            other => panic!("unexpected IBL bake executor `{other}`"),
        };
        let dispatch = ibl_bake_compute_dispatch_record(&context, output, pipeline_label)
            .expect("IBL bake dispatch should be derivable from graph metadata");

        record.audit_compute_workload(
            &pass.name,
            context.executor_id.as_str(),
            pass.compute_workload.as_ref(),
            dispatch_context,
            std::slice::from_ref(&dispatch),
        );
        record.push_compute_dispatch(dispatch);
    }

    assert_eq!(record.compute_dispatch_count(), 10);
    assert_eq!(record.compute_workload_planned_count(), 10);
    assert_eq!(record.compute_workload_matched_count(), 10);
    assert_eq!(record.compute_workload_mismatch_count(), 0);
    assert_eq!(record.compute_workload_missing_dispatch_count(), 0);
    let pmrem_mip0 = compute_dispatch_by_pass(&record, &ibl_bake_pmrem_pass_name(0));
    let pmrem_mip7 = compute_dispatch_by_pass(&record, &ibl_bake_pmrem_pass_name(7));
    let sh9 = compute_dispatch_by_pass(&record, IBL_BAKE_IRRADIANCE_SH9_PASS);
    let iem = compute_dispatch_by_pass(&record, IBL_BAKE_IRRADIANCE_CUBE_PASS);
    assert_eq!(
        pmrem_mip0.storage_write_resources,
        [IBL_BAKE_PMREM_RESOURCE.to_string()]
    );
    assert_eq!(pmrem_mip0.dispatch_groups, [16, 16, 6]);
    assert_eq!(pmrem_mip7.dispatch_groups, [1, 1, 1]);
    assert_eq!(sh9.dispatch_groups, [1, 1, 1]);
    assert_eq!(iem.dispatch_groups, [4, 4, 6]);
}

#[test]
fn ibl_bake_wgpu_request_uses_frame_source_shape_and_graph_contents() {
    let environment = source_cubemap_environment_extract(4, 17, [9, 8, 7, 6]);
    let bake_key = environment.ibl_bake_key().expect("source cubemap bake key");
    let request = IblBakeArtifactRequest::new(bake_key, 4, 3)
        .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let mut builder = RenderGraphBuilder::new("ibl-bake-request-rebuild-test");
    append_ibl_bake_artifact_graph_plan(&mut builder, &request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("graph should compile");
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == IBL_BAKE_IRRADIANCE_SH9_PASS)
        .expect("SH9 pass should exist");
    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_resource_resolver(&graph, pass.id);

    let rebuilt =
        ibl_bake_request_from_frame_environment_and_graph_metadata(&context, &environment)
            .expect("request rebuild");

    assert_eq!(rebuilt.bake_key(), bake_key);
    assert_eq!(rebuilt.source_face_size(), 4);
    assert_eq!(rebuilt.source_mip_count(), 3);
    assert_eq!(rebuilt.pmrem_face_size(), 128);
    assert_eq!(rebuilt.pmrem_mip_count(), 8);
    assert_eq!(
        rebuilt.required_contents(),
        IblBakeArtifactContents::PMREM_SH9_IEM
    );
}

#[test]
fn ibl_bake_wgpu_request_allows_sh9_only_without_pmrem_metadata() {
    let environment = source_cubemap_environment_extract(4, 23, [6, 7, 8, 9]);
    let bake_key = environment.ibl_bake_key().expect("source cubemap bake key");
    let request = IblBakeArtifactRequest::new(bake_key, 4, 3)
        .with_required_contents(IblBakeArtifactContents::SH9);
    let mut builder = RenderGraphBuilder::new("ibl-bake-request-sh9-only-test");
    append_ibl_bake_artifact_graph_plan(&mut builder, &request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("graph should compile");
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == IBL_BAKE_IRRADIANCE_SH9_PASS)
        .expect("SH9 pass should exist");
    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_resource_resolver(&graph, pass.id);

    assert!(context
        .resource_resolver()
        .and_then(|resolver| resolver.resource_lifetime_by_name(IBL_BAKE_PMREM_RESOURCE))
        .is_none());

    let rebuilt =
        ibl_bake_request_from_frame_environment_and_graph_metadata(&context, &environment)
            .expect("SH9-only graph should rebuild from the frame source cubemap request");

    assert_eq!(rebuilt.bake_key(), bake_key);
    assert_eq!(rebuilt.source_face_size(), 4);
    assert_eq!(rebuilt.source_mip_count(), 3);
    assert_eq!(rebuilt.pmrem_face_size(), 128);
    assert_eq!(rebuilt.pmrem_mip_count(), 8);
    assert_eq!(rebuilt.required_contents(), IblBakeArtifactContents::SH9);
}

#[test]
fn ibl_bake_compute_executor_registrations_are_explicit_opt_in() {
    let registrations = ibl_bake_compute_executor_registrations();

    assert_eq!(registrations.len(), 3);
    for executor_id in [
        IBL_BAKE_PMREM_EXECUTOR_ID,
        IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
        IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID,
    ] {
        assert!(
            registrations
                .iter()
                .any(|registration| registration.executor_id().as_str() == executor_id),
            "IBL bake executor `{executor_id}` should be explicitly registrable"
        );
    }
}

#[test]
fn ibl_bake_compute_dispatch_requires_source_read_and_output_write() {
    let context = RenderPassExecutionContext::with_declared_graph_metadata_and_resources(
        "env.ibl_prefilter",
        RenderPassExecutorId::new(IBL_BAKE_PMREM_EXECUTOR_ID),
        QueueLane::AsyncCompute,
        QueueLane::AsyncCompute,
        Default::default(),
        Vec::new(),
    );

    let error = ibl_bake_compute_dispatch_record(
        &context,
        IBL_BAKE_PMREM_RESOURCE,
        IBL_BAKE_PMREM_PIPELINE_LABEL,
    )
    .unwrap_err();

    assert!(error.contains("requires Read access"));
    assert!(error.contains(IBL_BAKE_SOURCE_CUBEMAP_RESOURCE));
}

#[test]
fn ibl_bake_pmrem_compute_dispatch_requires_mip_scoped_pass_name() {
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        128,
        8,
    )
    .with_required_contents(IblBakeArtifactContents::PMREM);
    let mut builder = RenderGraphBuilder::new("ibl-bake-pmrem-name-test");
    append_ibl_bake_artifact_graph_plan(&mut builder, &request)
        .expect("IBL bake graph plan should append");
    let graph = builder.compile().expect("graph should compile");
    let pass = graph
        .passes()
        .iter()
        .find(|pass| pass.name == ibl_bake_pmrem_pass_name(0))
        .expect("compiled graph should contain PMREM mip0");
    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            IBL_BAKE_PMREM_EXECUTOR_ID,
            RenderPassExecutorId::new(pass.executor_id.clone().unwrap()),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_resource_resolver(&graph, pass.id);

    let error = ibl_bake_compute_dispatch_record(
        &context,
        IBL_BAKE_PMREM_RESOURCE,
        IBL_BAKE_PMREM_PIPELINE_LABEL,
    )
    .unwrap_err();

    assert!(error.contains("must include a `.mipN` suffix"));
}

fn compute_dispatch_by_pass<'a>(
    record: &'a RenderGraphExecutionRecord,
    pass_name: &str,
) -> &'a RenderGraphComputeDispatchRecord {
    record
        .compute_dispatches()
        .iter()
        .find(|dispatch| dispatch.pass_name == pass_name)
        .unwrap_or_else(|| panic!("compute dispatch should exist for pass `{pass_name}`"))
}

fn source_cubemap_environment_extract(
    face_size: u32,
    source_revision: u64,
    source_hash: [u32; 4],
) -> EnvironmentExtract {
    EnvironmentExtract::source_cubemap(SourceCubemapEnvironment::new(
        build_source_cubemap_from_equirect(face_size, |_, _| [0.25, 0.5, 0.75, 1.0]),
        source_revision,
        source_hash,
    ))
}
