use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceRecord,
    UntypedResourceHandle,
};
use crate::graphics::scene::render_scene::RenderSceneResourceReferenceDelta;
use zr_rhi::{DeviceGeneration, DeviceId, RenderQueueClass, SubmissionStatus, SubmissionTicket};

use super::{
    RenderAssetDemandGeneration, RenderAssetDeviceEpoch, RenderAssetResidencyManager,
    RenderAssetResidencyReleaseKind, RenderAssetResidencyRoute, RenderAssetResidencyScope,
    RenderAssetResidencyState, RenderAssetResidencyTransitionError,
};

#[derive(Debug)]
struct TestPayload {
    _label: &'static str,
}

#[test]
fn render_residency_tickets_route_semantic_model_set_and_prepared_work_explicitly() {
    let resources = ResourceManager::new();
    let inputs = [
        (
            register_resource(&resources, "routes/model", ResourceKind::Model, Vec::new()),
            RenderAssetResidencyRoute::CanonicalMeshSet,
        ),
        (
            register_resource(&resources, "routes/mesh", ResourceKind::Mesh, Vec::new()),
            RenderAssetResidencyRoute::SemanticBlocks,
        ),
        (
            register_resource(
                &resources,
                "routes/texture",
                ResourceKind::Texture,
                Vec::new(),
            ),
            RenderAssetResidencyRoute::SemanticBlocks,
        ),
        (
            register_resource(
                &resources,
                "routes/material",
                ResourceKind::Material,
                Vec::new(),
            ),
            RenderAssetResidencyRoute::PreparedDependencies,
        ),
    ];
    let management = resources.management_generation();
    let readiness = resources.readiness_generation();
    let deltas = inputs
        .iter()
        .map(|(resource, _)| RenderSceneResourceReferenceDelta::acquire(*resource, 1))
        .collect::<Vec<_>>();
    let mut residency = RenderAssetResidencyManager::new();

    let mutation = residency
        .apply_scene_reference_deltas(
            &deltas,
            &management,
            &readiness,
            device_epoch(7, 2),
            demand_generation(5),
        )
        .unwrap_or_else(|error| panic!("route admission failed: {error:?}"));

    for (resource, expected) in inputs {
        assert_eq!(request_for(mutation.requests(), resource).route(), expected);
    }
}

fn demand_generation(raw: u64) -> RenderAssetDemandGeneration {
    match RenderAssetDemandGeneration::new(raw) {
        Some(generation) => generation,
        None => panic!("test demand generation must be non-zero"),
    }
}

fn device_epoch(device_id: u64, generation: u64) -> RenderAssetDeviceEpoch {
    RenderAssetDeviceEpoch::new(DeviceId::new(device_id), DeviceGeneration::new(generation))
}

fn resource_record(
    label: &str,
    kind: ResourceKind,
    dependencies: Vec<ResourceId>,
) -> ResourceRecord {
    let locator = match ResourceLocator::parse(&format!("res://{label}")) {
        Ok(locator) => locator,
        Err(error) => panic!("invalid test locator: {error}"),
    };
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator)
        .with_dependency_ids(dependencies)
}

fn register_resource(
    resources: &ResourceManager,
    label: &'static str,
    kind: ResourceKind,
    dependencies: Vec<ResourceId>,
) -> UntypedResourceHandle {
    match resources.register_ready(
        resource_record(label, kind, dependencies),
        TestPayload { _label: label },
    ) {
        Ok(handle) => handle,
        Err(error) => panic!("failed to register test resource: {error}"),
    }
}

fn request_for(
    requests: &[super::RenderAssetResidencyTicket],
    resource: UntypedResourceHandle,
) -> super::RenderAssetResidencyTicket {
    match requests
        .iter()
        .copied()
        .find(|ticket| ticket.resource() == resource)
    {
        Some(ticket) => ticket,
        None => panic!("missing request for {resource:?}"),
    }
}

fn advance_to_upload(
    residency: &mut RenderAssetResidencyManager,
    ticket: super::RenderAssetResidencyTicket,
) {
    for state in [
        RenderAssetResidencyState::Reading,
        RenderAssetResidencyState::Decoding,
        RenderAssetResidencyState::ReadyCpu,
        RenderAssetResidencyState::QueuedUpload,
    ] {
        assert_eq!(residency.advance(ticket, state), Ok(()));
    }
}

#[test]
fn render_scene_reference_delta_creates_generation_bound_all_lod_and_bootstrap_tickets() {
    let resources = ResourceManager::new();
    let model = register_resource(
        &resources,
        "models/ship.glb",
        ResourceKind::Model,
        Vec::new(),
    );
    let material = register_resource(
        &resources,
        "materials/ship.mat",
        ResourceKind::Material,
        Vec::new(),
    );
    let management = resources.management_generation();
    let readiness = resources.readiness_generation();
    let device = device_epoch(4, 9);
    let demand = demand_generation(17);
    let mut residency = RenderAssetResidencyManager::new();

    let mutation = match residency.apply_scene_reference_deltas(
        &[
            RenderSceneResourceReferenceDelta::acquire(model, 1),
            RenderSceneResourceReferenceDelta::acquire(material, 1),
        ],
        &management,
        &readiness,
        device,
        demand,
    ) {
        Ok(mutation) => mutation,
        Err(error) => panic!("reference admission failed: {error:?}"),
    };

    assert_eq!(mutation.requests().len(), 2);
    assert!(mutation.releases().is_empty());
    assert_eq!(mutation.stats().input_delta_count(), 2);
    assert_eq!(mutation.stats().preflight_entry_lookup_count(), 2);
    assert_eq!(mutation.stats().catalog_lookup_count(), 2);
    let model_ticket = request_for(mutation.requests(), model);
    let material_ticket = request_for(mutation.requests(), material);
    assert_eq!(model_ticket.scope(), RenderAssetResidencyScope::AllLods);
    assert_eq!(
        material_ticket.scope(),
        RenderAssetResidencyScope::Bootstrap
    );
    for ticket in [model_ticket, material_ticket] {
        let row = match management.row_by_id(ticket.resource().id()) {
            Some(row) => row,
            None => panic!("ticket resource must remain in catalog"),
        };
        assert_eq!(ticket.asset_revision(), row.revision);
        assert_eq!(ticket.readiness_generation(), readiness.sequence());
        assert_eq!(
            ticket.dependency_revision(),
            readiness
                .dependency_revision(ticket.resource().id())
                .unwrap_or(0)
        );
        assert_eq!(ticket.demand_generation(), demand);
        assert_eq!(ticket.device(), device);
    }
}

#[test]
fn render_repeated_scene_references_share_one_pending_ticket_until_the_last_release() {
    let resources = ResourceManager::new();
    let mesh = register_resource(
        &resources,
        "meshes/hull.mesh",
        ResourceKind::Mesh,
        Vec::new(),
    );
    let management = resources.management_generation();
    let readiness = resources.readiness_generation();
    let device = device_epoch(2, 1);
    let demand = demand_generation(3);
    let mut residency = RenderAssetResidencyManager::new();

    let first = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(mesh, 2)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("initial admission failed: {error:?}"));
    let ticket = request_for(first.requests(), mesh);
    let repeated = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(mesh, 3)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("repeated admission failed: {error:?}"));

    assert!(repeated.requests().is_empty());
    assert_eq!(residency.reference_count(mesh), 5);
    let retained = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::release(mesh, 4)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("partial release failed: {error:?}"));
    assert!(retained.releases().is_empty());
    assert_eq!(residency.reference_count(mesh), 1);

    let final_release = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::release(mesh, 1)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("final release failed: {error:?}"));
    assert_eq!(final_release.releases().len(), 1);
    assert_eq!(
        final_release.releases()[0].kind(),
        RenderAssetResidencyReleaseKind::CancelPending
    );
    assert_eq!(final_release.releases()[0].ticket(), ticket);
    assert_eq!(residency.reference_count(mesh), 0);
    assert!(matches!(
        residency.advance(ticket, RenderAssetResidencyState::Reading),
        Err(RenderAssetResidencyTransitionError::UnknownTicket { .. })
    ));
}

#[test]
fn render_reference_delta_preflight_is_atomic_on_underflow() {
    let resources = ResourceManager::new();
    let model = register_resource(
        &resources,
        "models/atomic.glb",
        ResourceKind::Model,
        Vec::new(),
    );
    let material = register_resource(
        &resources,
        "materials/atomic.mat",
        ResourceKind::Material,
        Vec::new(),
    );
    let management = resources.management_generation();
    let readiness = resources.readiness_generation();
    let device = device_epoch(6, 1);
    let demand = demand_generation(1);
    let mut residency = RenderAssetResidencyManager::new();
    let initial = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(model, 1)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("initial admission failed: {error:?}"));
    let initial_ticket = request_for(initial.requests(), model);

    let failed = residency.apply_scene_reference_deltas(
        &[
            RenderSceneResourceReferenceDelta::release(model, 2),
            RenderSceneResourceReferenceDelta::acquire(material, 1),
        ],
        &management,
        &readiness,
        device,
        demand,
    );
    assert!(failed.is_err());
    assert_eq!(residency.reference_count(model), 1);
    assert_eq!(residency.reference_count(material), 0);
    assert_eq!(residency.pending_ticket(model), Some(initial_ticket));

    let admitted = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(material, 1)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("post-rollback admission failed: {error:?}"));
    let material_ticket = request_for(admitted.requests(), material);
    assert_eq!(
        material_ticket.id().raw(),
        initial_ticket.id().raw().saturating_add(1)
    );
}

#[test]
fn render_upload_requires_legal_state_and_matching_device_submission() {
    let resources = ResourceManager::new();
    let mesh = register_resource(
        &resources,
        "meshes/upload.mesh",
        ResourceKind::Mesh,
        Vec::new(),
    );
    let management = resources.management_generation();
    let readiness = resources.readiness_generation();
    let device = device_epoch(8, 11);
    let mut residency = RenderAssetResidencyManager::new();
    let admitted = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(mesh, 1)],
            &management,
            &readiness,
            device,
            demand_generation(5),
        )
        .unwrap_or_else(|error| panic!("admission failed: {error:?}"));
    let ticket = request_for(admitted.requests(), mesh);

    assert!(matches!(
        residency.advance(ticket, RenderAssetResidencyState::ReadyCpu),
        Err(RenderAssetResidencyTransitionError::InvalidTransition { .. })
    ));
    advance_to_upload(&mut residency, ticket);
    let wrong_submission = SubmissionTicket::new(
        DeviceId::new(99),
        device.generation(),
        RenderQueueClass::Copy,
        1,
    );
    assert!(matches!(
        residency.bind_upload_submission(ticket, wrong_submission),
        Err(RenderAssetResidencyTransitionError::SubmissionDeviceMismatch { .. })
    ));

    let submission = SubmissionTicket::new(
        device.device_id(),
        device.generation(),
        RenderQueueClass::Copy,
        7,
    );
    assert_eq!(residency.bind_upload_submission(ticket, submission), Ok(()));
    assert!(matches!(
        residency.complete_upload(ticket, submission, SubmissionStatus::Submitted),
        Err(RenderAssetResidencyTransitionError::SubmissionNotTerminal { .. })
    ));
    let published = residency
        .complete_upload(ticket, submission, SubmissionStatus::Completed)
        .unwrap_or_else(|error| panic!("upload completion failed: {error:?}"));
    assert!(published.releases().is_empty());
    assert_eq!(residency.resident_ticket(mesh), Some(ticket));
    assert_eq!(
        residency.state(ticket),
        Some(RenderAssetResidencyState::Resident)
    );
}

#[test]
fn render_hot_reload_keeps_last_good_until_the_new_generation_completes() {
    let resources = ResourceManager::new();
    let model = register_resource(
        &resources,
        "models/reload.glb",
        ResourceKind::Model,
        Vec::new(),
    );
    let device = device_epoch(10, 2);
    let mut residency = RenderAssetResidencyManager::new();
    let initial = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(model, 1)],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device,
            demand_generation(1),
        )
        .unwrap_or_else(|error| panic!("initial admission failed: {error:?}"));
    let first_ticket = request_for(initial.requests(), model);
    advance_to_upload(&mut residency, first_ticket);
    let first_submission = SubmissionTicket::new(
        device.device_id(),
        device.generation(),
        RenderQueueClass::Copy,
        1,
    );
    assert_eq!(
        residency.bind_upload_submission(first_ticket, first_submission),
        Ok(())
    );
    let first_publish = residency
        .complete_upload(first_ticket, first_submission, SubmissionStatus::Completed)
        .unwrap_or_else(|error| panic!("initial upload failed: {error:?}"));
    assert!(first_publish.releases().is_empty());

    let dependency = ResourceId::from_stable_label("reload-dependency-v2");
    let updated = register_resource(
        &resources,
        "models/reload.glb",
        ResourceKind::Model,
        vec![dependency],
    );
    assert_eq!(updated, model);
    let replacement = residency
        .reconcile_changed_resources(
            &[model],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device,
            demand_generation(2),
        )
        .unwrap_or_else(|error| panic!("reload reconciliation failed: {error:?}"));
    let second_ticket = request_for(replacement.requests(), model);
    assert_ne!(
        second_ticket.asset_revision(),
        first_ticket.asset_revision()
    );
    assert_eq!(residency.resident_ticket(model), Some(first_ticket));
    assert_eq!(residency.pending_ticket(model), Some(second_ticket));
    assert!(matches!(
        residency.advance(first_ticket, RenderAssetResidencyState::Reading),
        Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
    ));

    assert_eq!(residency.fail_pending(second_ticket), Ok(()));
    assert_eq!(residency.resident_ticket(model), Some(first_ticket));
    assert_eq!(
        residency.state(second_ticket),
        Some(RenderAssetResidencyState::Failed)
    );

    let retry = residency
        .reconcile_changed_resources(
            &[model],
            &resources.management_generation(),
            &resources.readiness_generation(),
            device,
            demand_generation(3),
        )
        .unwrap_or_else(|error| panic!("reload retry reconciliation failed: {error:?}"));
    let third_ticket = request_for(retry.requests(), model);
    assert_eq!(retry.releases().len(), 1);
    assert_eq!(
        retry.releases()[0].kind(),
        RenderAssetResidencyReleaseKind::DropTerminal
    );
    advance_to_upload(&mut residency, third_ticket);
    let third_submission = SubmissionTicket::new(
        device.device_id(),
        device.generation(),
        RenderQueueClass::Copy,
        3,
    );
    assert_eq!(
        residency.bind_upload_submission(third_ticket, third_submission),
        Ok(())
    );
    let published = residency
        .complete_upload(third_ticket, third_submission, SubmissionStatus::Completed)
        .unwrap_or_else(|error| panic!("replacement upload failed: {error:?}"));

    assert_eq!(residency.resident_ticket(model), Some(third_ticket));
    assert_eq!(residency.pending_ticket(model), None);
    assert_eq!(published.releases().len(), 1);
    assert_eq!(published.releases()[0].ticket(), first_ticket);
    assert_eq!(
        published.releases()[0].kind(),
        RenderAssetResidencyReleaseKind::RetireResident
    );
    assert_eq!(published.releases()[0].submission(), Some(first_submission));
}

#[test]
fn render_releasing_an_uploading_asset_preserves_submission_for_fence_retirement() {
    let resources = ResourceManager::new();
    let texture = register_resource(
        &resources,
        "textures/streaming.ktx2",
        ResourceKind::Texture,
        Vec::new(),
    );
    let management = resources.management_generation();
    let readiness = resources.readiness_generation();
    let device = device_epoch(12, 1);
    let demand = demand_generation(4);
    let mut residency = RenderAssetResidencyManager::new();
    let admitted = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("texture admission failed: {error:?}"));
    let ticket = request_for(admitted.requests(), texture);
    advance_to_upload(&mut residency, ticket);
    let submission = SubmissionTicket::new(
        device.device_id(),
        device.generation(),
        RenderQueueClass::Copy,
        5,
    );
    assert_eq!(residency.bind_upload_submission(ticket, submission), Ok(()));

    let released = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::release(texture, 1)],
            &management,
            &readiness,
            device,
            demand,
        )
        .unwrap_or_else(|error| panic!("texture release failed: {error:?}"));
    assert_eq!(released.releases().len(), 1);
    assert_eq!(
        released.releases()[0].kind(),
        RenderAssetResidencyReleaseKind::RetireInFlight
    );
    assert_eq!(released.releases()[0].submission(), Some(submission));
    assert_eq!(residency.reference_count(texture), 0);
}

#[path = "tests/semantic_blocks.rs"]
mod semantic_blocks;

#[path = "tests/device_recovery.rs"]
mod device_recovery;
