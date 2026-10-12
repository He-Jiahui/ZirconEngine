use crate::core::resource::{ResourceKind, ResourceManager};
use crate::graphics::scene::render_scene::RenderSceneResourceReferenceDelta;

use super::super::{
    RenderAssetResidencyManager, RenderAssetResidencyRoute, RenderAssetResidencyWorkQueue,
};
use super::{demand_generation, device_epoch, register_resource, request_for};

#[test]
fn work_queue_partitions_routes_and_keeps_failed_semantic_admission_queued() {
    let resources = ResourceManager::new();
    let model = register_resource(&resources, "routing/model", ResourceKind::Model, Vec::new());
    let mesh = register_resource(&resources, "routing/mesh", ResourceKind::Mesh, Vec::new());
    let material = register_resource(
        &resources,
        "routing/material",
        ResourceKind::Material,
        Vec::new(),
    );
    let mut residency = RenderAssetResidencyManager::new();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[
                RenderSceneResourceReferenceDelta::acquire(model, 1),
                RenderSceneResourceReferenceDelta::acquire(mesh, 1),
                RenderSceneResourceReferenceDelta::acquire(material, 1),
            ],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device_epoch(41, 2),
            demand_generation(7),
        )
        .unwrap_or_else(|error| panic!("residency setup failed: {error:?}"));
    let mesh_ticket = request_for(mutation.requests(), mesh);
    let mut queue = RenderAssetResidencyWorkQueue::default();
    queue.retain_mutation(&mutation);

    assert_eq!(queue.pending_request_count(), 3);
    assert_eq!(
        queue.pending_request_count_for_route(RenderAssetResidencyRoute::SemanticBlocks),
        1
    );
    assert_eq!(
        queue.pending_request_count_for_route(RenderAssetResidencyRoute::CanonicalMeshSet),
        1
    );
    assert_eq!(
        queue.pending_request_count_for_route(RenderAssetResidencyRoute::PreparedDependencies),
        1
    );

    let failure = queue
        .try_admit_next_semantic(|ticket| {
            assert_eq!(ticket, mesh_ticket);
            Err("bounded admission full")
        })
        .expect_err("failed owner admission must be reported");
    assert_eq!(failure.ticket(), mesh_ticket);
    assert_eq!(failure.error(), &"bounded admission full");
    assert_eq!(queue.pending_request_count(), 3);

    let admitted = queue
        .try_admit_next_semantic(|ticket| {
            assert_eq!(ticket, mesh_ticket);
            Ok::<_, &'static str>(())
        })
        .unwrap_or_else(|failure| panic!("semantic admission failed: {failure:?}"));
    assert_eq!(admitted, Some(mesh_ticket));
    assert_eq!(queue.pending_request_count(), 2);
    assert_eq!(
        queue.pending_request_count_for_route(RenderAssetResidencyRoute::SemanticBlocks),
        0
    );
}

#[test]
fn work_queue_cancels_dispatched_semantic_work_but_drops_unstarted_requests_locally() {
    let resources = ResourceManager::new();
    let texture = register_resource(
        &resources,
        "routing/texture",
        ResourceKind::Texture,
        Vec::new(),
    );
    let management = resources.management_generation();
    let readiness = resources.readiness_generation();
    let device = device_epoch(42, 3);
    let demand = demand_generation(8);
    let mut residency = RenderAssetResidencyManager::new();
    let acquired = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("texture admission failed: {error:?}"));
    let ticket = request_for(acquired.requests(), texture);
    let mut queue = RenderAssetResidencyWorkQueue::default();
    queue.retain_mutation(&acquired);

    assert_eq!(
        queue
            .try_admit_next_semantic(|_| Ok::<_, &'static str>(()))
            .unwrap_or_else(|failure| panic!("semantic dispatch failed: {failure:?}")),
        Some(ticket.clone())
    );
    let released = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::release(texture, 1)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("texture release failed: {error:?}"));
    queue.retain_mutation(&released);
    assert_eq!(queue.pending_semantic_cancellation_count(), 1);
    assert_eq!(queue.pop_next_semantic_cancellation(), Some(ticket));

    let reacquired = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &management,
            &readiness,
            device,
            demand_generation(9),
        )
        .unwrap_or_else(|error| panic!("texture readmission failed: {error:?}"));
    queue.retain_mutation(&reacquired);
    let rereleased = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::release(texture, 1)],
            &management,
            &readiness,
            device,
            demand_generation(9),
        )
        .unwrap_or_else(|error| panic!("queued texture release failed: {error:?}"));
    queue.retain_mutation(&rereleased);

    assert_eq!(queue.pending_request_count(), 0);
    assert_eq!(queue.pending_semantic_cancellation_count(), 0);
}
