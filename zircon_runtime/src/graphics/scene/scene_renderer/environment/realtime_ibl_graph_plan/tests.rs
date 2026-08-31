use super::*;
use crate::core::framework::render::{IblBakeArtifactRequest, ProceduralSkyParams};
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::{
    CubeMipRange, RealtimeIblCompletion, RealtimeIblTimeSliceConfig, RealtimeIblTimeSliceScheduler,
};
use crate::render_graph::{
    RenderGraphBuilder, RenderGraphComputeDispatchExtent, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};

fn request() -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        128,
        8,
    )
}

fn scheduler() -> RealtimeIblTimeSliceScheduler {
    RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid realtime config"),
    )
}

fn ticket_batch_matching(
    scheduler: &mut RealtimeIblTimeSliceScheduler,
    predicate: impl Fn(&RealtimeIblFrameBatch) -> bool,
) -> RealtimeIblFrameBatch {
    for frame in 1..64 {
        let batch = scheduler.begin_frame(frame).expect("ticket batch");
        if predicate(&batch) {
            return batch;
        }
        assert_eq!(
            scheduler.complete_frame(batch.token(), true),
            RealtimeIblCompletion::Advanced
        );
    }
    panic!("ticket stage was not scheduled");
}

#[test]
fn slot_selection_borrows_the_requested_persistent_slot() {
    let request = request();
    let mut builder = RenderGraphBuilder::new("realtime-ibl-slot-selection");
    let slot_a = import_slot(&mut builder, &request, IblRealtimeBufferSlot::A);
    let slot_b = import_slot(&mut builder, &request, IblRealtimeBufferSlot::B);

    assert!(std::ptr::eq(
        select_slot(&slot_a, &slot_b, IblRealtimeBufferSlot::A),
        &slot_a
    ));
    assert!(std::ptr::eq(
        select_slot(&slot_a, &slot_b, IblRealtimeBufferSlot::B),
        &slot_b
    ));
}

#[test]
fn capture_ticket_materializes_only_the_live_work_slot_source_mip() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(ProceduralSkyParams::default_gradient().ibl_bake_key());
    let batch = scheduler.begin_frame(1).expect("capture ticket");
    let mut builder = RenderGraphBuilder::new("realtime-ibl-capture-ticket");

    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");

    assert_eq!(plan.passes.len(), 1);
    assert!(matches!(
        plan.passes[0].kind,
        RealtimeIblGraphPassKind::CaptureSky(CubeFaceRange { first: 0, count: 2 })
    ));
    for resource in plan.ready.resources() {
        assert!(graph.resource_lifetime_by_name(&resource.name).is_none());
    }
    assert!(graph
        .resource_lifetime_by_name(&plan.work.source.storage_mips[0].name)
        .is_some_and(|lifetime| lifetime.kind == RenderGraphResourceKind::External));
    assert!(graph
        .resource_lifetime_by_name(&plan.work.pmrem.storage_mips[0].name)
        .is_none());
    assert!(graph
        .resource_lifetime_by_name(&plan.work.sh9.name)
        .is_none());
}

#[test]
fn source_mip_ticket_reads_only_its_immediate_predecessor() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(ProceduralSkyParams::default_gradient().ibl_bake_key());
    let batch = ticket_batch_matching(&mut scheduler, |batch| {
        matches!(
            batch.operations(),
            [RealtimeIblOperation::GenerateSourceMip { mip_level: 1 }]
        )
    });
    let mut builder = RenderGraphBuilder::new("realtime-ibl-source-mip-ticket");

    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");
    let pass = graph.passes().first().expect("one source mip pass");

    assert_eq!(plan.passes.len(), 1);
    assert!(matches!(
        plan.passes[0].kind,
        RealtimeIblGraphPassKind::GenerateSourceMip { mip_level: 1 }
    ));
    assert!(pass.resources.iter().any(|access| {
        access.name == plan.work.source.sampled_mips[0].name
            && access.access == RenderGraphResourceAccessKind::Read
    }));
    assert!(pass.resources.iter().any(|access| {
        access.name == plan.work.source.storage_mips[1].name
            && access.access == RenderGraphResourceAccessKind::Write
    }));
}

#[test]
fn pmrem_ticket_preserves_face_budget_in_the_compiled_dispatch() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(ProceduralSkyParams::default_gradient().ibl_bake_key());
    let batch = ticket_batch_matching(&mut scheduler, |batch| {
        matches!(
            batch.operations(),
            [RealtimeIblOperation::Prefilter {
                mips: CubeMipRange { first: 0, count: 1 },
                faces: CubeFaceRange { first: 0, count: 2 },
            }]
        )
    });
    let mut builder = RenderGraphBuilder::new("realtime-ibl-prefilter-ticket");

    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");

    assert_eq!(plan.passes.len(), 1);
    assert_eq!(
        plan.passes[0].workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([16, 16, 2])
    );
    assert_eq!(graph.passes().len(), 1);
}

#[test]
fn diffuse_sh9_ticket_uses_one_canonical_workgroup_in_the_compiled_graph() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(ProceduralSkyParams::default_gradient().ibl_bake_key());
    let batch = ticket_batch_matching(&mut scheduler, |batch| {
        matches!(
            batch.operations(),
            [RealtimeIblOperation::ProjectDiffuseSh9]
        )
    });
    let mut builder = RenderGraphBuilder::new("realtime-ibl-diffuse-sh9-ticket");

    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");

    assert_eq!(plan.passes.len(), 1);
    assert!(matches!(
        plan.passes[0].kind,
        RealtimeIblGraphPassKind::ProjectDiffuseSh9
    ));
    assert_eq!(
        plan.passes[0].workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1])
    );
    assert_eq!(graph.passes().len(), 1);
}

#[test]
fn canonical_realtime_ticket_graph_budget_counts_4124_workgroups() {
    let sky = ProceduralSkyParams::default_gradient();
    let request = request();
    let mut scheduler = scheduler();
    scheduler.request_rebake(sky.ibl_bake_key());
    let mut workgroups = 0_u64;
    let mut completed_batches = 0;

    for frame in 1..64 {
        let batch = scheduler.begin_frame(frame).expect("ticket batch");
        let mut builder = RenderGraphBuilder::new("realtime-ibl-ticket-budget");
        let plan = append_realtime_ibl_graph_plan(&mut builder, &request, &batch)
            .expect("realtime IBL graph plan");
        builder.compile().expect("realtime graph compiles");
        completed_batches += 1;
        workgroups += plan
            .passes
            .iter()
            .map(|pass| match pass.workload.dispatch_extent {
                RenderGraphComputeDispatchExtent::Fixed(groups) => {
                    u64::from(groups[0]) * u64::from(groups[1]) * u64::from(groups[2])
                }
                ref other => panic!("realtime IBL requires fixed dispatches, got {other:?}"),
            })
            .sum::<u64>();

        if scheduler.complete_frame(batch.token(), true) == RealtimeIblCompletion::Published {
            break;
        }
    }

    assert_eq!(completed_batches, 21);
    assert_eq!(workgroups, 4_124);
}

#[test]
fn topology_key_distinguishes_swapped_a_b_slots_with_the_same_operation() {
    let mut scheduler = scheduler();
    let sky = ProceduralSkyParams::default_gradient();
    scheduler.request_rebake(sky.ibl_bake_key());
    let initial = scheduler.begin_frame(1).expect("initial capture batch");
    let initial_key = RealtimeIblGraphTopologyKey::from_batch(&initial);
    let repeated = scheduler
        .begin_frame(1)
        .expect("same frame retains its batch");
    assert_eq!(
        initial_key,
        RealtimeIblGraphTopologyKey::from_batch(&repeated),
        "the same ticket batch must reuse one topology identity"
    );

    assert_eq!(
        scheduler.complete_frame(initial.token(), true),
        RealtimeIblCompletion::Advanced
    );
    let mut published = false;
    for frame in 2..64 {
        let batch = scheduler.begin_frame(frame).expect("ticket batch");
        if scheduler.complete_frame(batch.token(), true) == RealtimeIblCompletion::Published {
            published = true;
            break;
        }
    }
    assert!(
        published,
        "the complete ticket must publish before the slot swap"
    );
    let mut changed_sky = sky;
    changed_sky.source_revision = 1;
    assert!(scheduler.request_rebake(changed_sky.ibl_bake_key()));
    let swapped = scheduler
        .begin_frame(64)
        .expect("swapped-slot capture batch");

    assert_eq!(initial.operation(), swapped.operation());
    assert_ne!(
        initial_key,
        RealtimeIblGraphTopologyKey::from_batch(&swapped),
        "A/B slot ownership is part of graph and execution-resource identity"
    );
}

#[test]
fn graph_plan_has_no_cloud_capture_without_a_distinct_cloud_producer() {
    let source = include_str!("../realtime_ibl_graph_plan.rs");

    assert!(!source.contains("CAPTURE_CLOUD"));
    assert!(!source.contains("CaptureCloud"));
}
