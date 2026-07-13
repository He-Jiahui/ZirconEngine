use super::*;
use crate::core::framework::render::{IblBakeArtifactRequest, ProceduralSkyParams};
use crate::graphics::scene::scene_renderer::environment::realtime_ibl_time_slice::{
    RealtimeIblCompletion, RealtimeIblTimeSliceConfig, RealtimeIblTimeSliceScheduler,
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

fn first_full_batch() -> RealtimeIblFrameBatch {
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid realtime config"),
    );
    scheduler.request_rebake(ProceduralSkyParams::default_gradient().ibl_bake_key());
    scheduler.begin_frame(1).expect("first full batch")
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
fn realtime_graph_materializes_only_live_work_slot_resources() {
    let batch = first_full_batch();
    let mut builder = RenderGraphBuilder::new("realtime-ibl-full");

    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");

    assert_eq!(plan.ready.slot, batch.ready_slot());
    assert_eq!(plan.work.slot, batch.work_slot());
    for resource in plan.ready.resources() {
        assert!(
            graph.resource_lifetime_by_name(&resource.name).is_none(),
            "unused ready-slot resource `{}` should be pruned from the compiled graph",
            resource.name
        );
    }
    for resource in plan.work.resources() {
        let is_live = graph.passes().iter().any(|pass| {
            pass.resources
                .iter()
                .any(|access| access.name == resource.name)
        });
        let lifetime = graph.resource_lifetime_by_name(&resource.name);
        assert_eq!(
            lifetime.is_some(),
            is_live,
            "work-slot resource `{}` lifetime should match compiled pass usage",
            resource.name
        );
        if let Some(lifetime) = lifetime {
            assert_eq!(lifetime.kind, RenderGraphResourceKind::External);
            assert!(lifetime.usage.persistent);
        }
    }
    assert_eq!(plan.work.source.sampled_mips.len(), 8);
    assert_eq!(plan.work.source.storage_mips.len(), 8);
    assert_eq!(plan.work.pmrem.storage_mips.len(), 8);
    assert!(graph.passes().iter().any(|pass| {
        pass.resources.iter().any(|access| {
            access.name == plan.work.source.storage_mips[0].name
                && access.access == RenderGraphResourceAccessKind::Write
        })
    }));
    assert!(graph.passes().iter().any(|pass| {
        pass.resources.iter().any(|access| {
            access.name == plan.work.pmrem.storage_mips[0].name
                && access.access == RenderGraphResourceAccessKind::Write
        })
    }));
    assert!(graph.passes().iter().any(|pass| {
        pass.resources.iter().any(|access| {
            access.name == plan.work.sh9.name
                && access.access == RenderGraphResourceAccessKind::Write
        })
    }));
}

#[test]
fn source_mip_generation_uses_non_overlapping_sampled_and_storage_views() {
    let batch = first_full_batch();
    let mut builder = RenderGraphBuilder::new("realtime-ibl-source-mips");

    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");
    let source_mip_passes = plan
        .passes
        .iter()
        .filter_map(|pass| match pass.kind {
            RealtimeIblGraphPassKind::GenerateSourceMip { mip_level } => {
                Some((mip_level, pass.pass_id))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(source_mip_passes.len(), 7);
    for (expected_mip, (mip_level, pass_id)) in (1_u32..8).zip(source_mip_passes) {
        assert_eq!(mip_level, expected_mip);
        let pass = graph
            .passes()
            .iter()
            .find(|pass| pass.id == pass_id)
            .expect("source mip pass");
        assert!(pass.resources.iter().any(|access| {
            access.name == plan.work.source.sampled_mips[(expected_mip - 1) as usize].name
                && access.access == RenderGraphResourceAccessKind::Read
        }));
        assert!(pass.resources.iter().any(|access| {
            access.name == plan.work.source.storage_mips[expected_mip as usize].name
                && access.access == RenderGraphResourceAccessKind::Write
        }));
        assert_ne!(
            plan.work.source.sampled_mips[(expected_mip - 1) as usize].name,
            plan.work.source.storage_mips[expected_mip as usize].name
        );
    }
}

#[test]
fn full_update_expands_prefilter_into_one_dispatch_per_mip() {
    let batch = first_full_batch();
    let mut builder = RenderGraphBuilder::new("realtime-ibl-prefilter-full");

    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("realtime IBL graph plan");
    let graph = builder.compile().expect("realtime graph compiles");

    let prefilter = plan
        .passes
        .iter()
        .filter_map(|pass| match pass.kind {
            RealtimeIblGraphPassKind::Prefilter(slice) => Some((slice, &pass.workload)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(prefilter.len(), 8);
    for (expected_mip, (slice, workload)) in prefilter.iter().enumerate() {
        assert_eq!(usize::from(slice.mip_level), expected_mip);
        assert_eq!(slice.first_face, 0);
        assert_eq!(slice.face_count, 6);
        assert_eq!(
            workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::Fixed([
                div_ceil((128_u32 >> expected_mip).max(1), 8),
                div_ceil((128_u32 >> expected_mip).max(1), 8),
                6,
            ])
        );
        let pass = graph_pass_for_kind(&plan, slice.mip_level);
        let compiled = graph
            .passes()
            .iter()
            .find(|candidate| candidate.id == pass.pass_id)
            .expect("prefilter pass");
        assert!(compiled.resources.iter().any(|access| {
            access.name == plan.work.source.sampled.name
                && access.access == RenderGraphResourceAccessKind::Read
        }));
        assert!(compiled.resources.iter().any(|access| {
            access.name == plan.work.pmrem.storage_mips[expected_mip].name
                && access.access == RenderGraphResourceAccessKind::Write
        }));
    }
}

#[test]
fn sliced_prefilter_preserves_face_range_and_orders_passes() {
    let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid realtime config"),
    );
    scheduler.request_rebake(key);
    let initial = scheduler.begin_frame(1).expect("initial batch");
    assert_eq!(
        scheduler.complete_frame(initial.token(), true),
        RealtimeIblCompletion::Published
    );
    let mut next = ProceduralSkyParams::default_gradient();
    next.horizon_color.x += 0.1;
    scheduler.request_rebake(next.ibl_bake_key());
    for frame in 2..=8 {
        let batch = scheduler.begin_frame(frame).expect("sliced batch");
        scheduler.complete_frame(batch.token(), true);
    }
    let batch = scheduler.begin_frame(9).expect("first PMREM face pair");
    assert_eq!(batch.logical_state(), 3);

    let mut builder = RenderGraphBuilder::new("realtime-ibl-prefilter-slice");
    let plan = append_realtime_ibl_graph_plan(&mut builder, &request(), &batch)
        .expect("sliced graph plan");
    let graph = builder.compile().expect("sliced graph compiles");

    assert_eq!(plan.passes.len(), 1);
    let RealtimeIblGraphPassKind::Prefilter(slice) = plan.passes[0].kind else {
        panic!("state 3 must emit a prefilter pass");
    };
    assert_eq!(slice.mip_level, 0);
    assert_eq!(slice.first_face, 0);
    assert_eq!(slice.face_count, 2);
    assert_eq!(
        plan.passes[0].workload.dispatch_extent,
        RenderGraphComputeDispatchExtent::Fixed([16, 16, 2])
    );
    assert_eq!(graph.passes().len(), 1);
}

fn graph_pass_for_kind(plan: &RealtimeIblGraphPlan, mip_level: u8) -> &RealtimeIblGraphPass {
    plan.passes
        .iter()
        .find(|pass| {
            matches!(
                pass.kind,
                RealtimeIblGraphPassKind::Prefilter(slice) if slice.mip_level == mip_level
            )
        })
        .expect("prefilter graph pass")
}
