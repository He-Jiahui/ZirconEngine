use super::*;
use crate::graphics::scene::resources::render_asset_residency::{
    RenderAssetResidencyAdmissionError, RenderAssetResidencyMutation, RenderAssetResidencyTicket,
};

fn acquire(
    residency: &mut RenderAssetResidencyManager,
    resources: &ResourceManager,
    resource: UntypedResourceHandle,
) -> RenderAssetResidencyTicket {
    let projection = resources.projection_snapshot();
    let mutation = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(resource, 1)],
            &projection,
            device_epoch(41, 3),
            demand_generation(1),
        )
        .expect("resource admission");
    request_for(mutation.requests(), resource)
}

fn reconcile(
    residency: &mut RenderAssetResidencyManager,
    resources: &ResourceManager,
    resource: UntypedResourceHandle,
) -> RenderAssetResidencyMutation {
    let projection = resources.projection_snapshot();
    residency
        .reconcile_changed_resources(
            &[resource],
            &projection,
            device_epoch(41, 3),
            demand_generation(1),
        )
        .expect("resource reconciliation")
}

fn begin_upload(
    residency: &mut RenderAssetResidencyManager,
    ticket: &RenderAssetResidencyTicket,
) -> SubmissionTicket {
    advance_to_upload(residency, ticket);
    let submission = SubmissionTicket::new(
        ticket.device().device_id(),
        ticket.device().generation(),
        RenderQueueClass::Copy,
        ticket.id().raw(),
    );
    residency
        .bind_upload_submission(ticket, submission)
        .expect("bind upload");
    submission
}

#[test]
fn unrelated_publication_keeps_pending_and_resident_tickets() {
    let resources = ResourceManager::new();
    let mesh = register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new());
    let mut residency = RenderAssetResidencyManager::new();
    let ticket = acquire(&mut residency, &resources, mesh);
    for (resident, unrelated) in [
        (false, "identity/unrelated-a"),
        (true, "identity/unrelated-b"),
    ] {
        let previous_generation = resources.readiness_generation().identity();
        register_resource(&resources, unrelated, ResourceKind::Data, Vec::new());
        assert_ne!(
            previous_generation,
            resources.readiness_generation().identity()
        );
        let mutation = reconcile(&mut residency, &resources, mesh);
        assert!(mutation.requests().is_empty());
        assert!(mutation.releases().is_empty());
        let current = if resident {
            residency.resident_ticket(mesh)
        } else {
            residency.pending_ticket(mesh)
        };
        assert_eq!(current.as_ref(), Some(&ticket));
        if !resident {
            let submission = begin_upload(&mut residency, &ticket);
            residency
                .complete_upload(&ticket, submission, SubmissionStatus::Completed)
                .expect("publish unchanged resource");
        }
    }
}

#[test]
fn dependency_publication_reissues_unchanged_root_and_rejects_old_completion() {
    let resources = ResourceManager::new();
    let dependency = register_resource(
        &resources,
        "identity/dependency",
        ResourceKind::Data,
        Vec::new(),
    );
    let mesh = register_resource(
        &resources,
        "identity/mesh",
        ResourceKind::Mesh,
        vec![dependency.id()],
    );
    let mut residency = RenderAssetResidencyManager::new();
    let old = acquire(&mut residency, &resources, mesh);
    let old_submission = begin_upload(&mut residency, &old);
    resources
        .register_ready(
            resource_record("identity/dependency", ResourceKind::Data, Vec::new())
                .with_source_hash("v2"),
            TestPayload {
                _label: "identity/dependency",
            },
        )
        .expect("publish changed dependency");

    let mutation = reconcile(&mut residency, &resources, mesh);
    let current = request_for(mutation.requests(), mesh);
    assert_eq!(current.asset_revision(), old.asset_revision());
    assert_ne!(current.source_identity(), old.source_identity());
    assert_eq!(mutation.releases().len(), 1);
    assert_eq!(
        mutation.releases()[0].kind(),
        RenderAssetResidencyReleaseKind::RetireInFlight
    );
    assert_eq!(mutation.releases()[0].ticket(), old);
    assert_eq!(
        residency.complete_upload(&old, old_submission, SubmissionStatus::Completed),
        Err(RenderAssetResidencyTransitionError::StaleTicket {
            presented: old.id(),
            current: current.id(),
        })
    );
    assert_eq!(residency.resident_ticket(mesh), None);
    assert_eq!(
        residency.state(&current),
        Some(RenderAssetResidencyState::QueuedIo)
    );
    let submission = begin_upload(&mut residency, &current);
    residency
        .complete_upload(&current, submission, SubmissionStatus::Completed)
        .expect("publish current dependency closure");
    assert_eq!(residency.resident_ticket(mesh).as_ref(), Some(&current));
}

#[test]
fn manager_replacement_and_remove_readd_cannot_alias_retained_publication() {
    let mut residency = RenderAssetResidencyManager::new();
    let (mesh, old, old_projection) = {
        let resources = ResourceManager::new();
        let mesh = register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new());
        let old = acquire(&mut residency, &resources, mesh);
        (mesh, old, resources.projection_snapshot())
    };
    let resources = ResourceManager::new();
    assert_eq!(
        register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new()),
        mesh
    );
    let mutation = reconcile(&mut residency, &resources, mesh);
    let current = request_for(mutation.requests(), mesh);
    assert_eq!(old.asset_revision(), current.asset_revision());
    assert_ne!(old.source_identity(), current.source_identity());
    let wrong_publication = RenderAssetResidencyTicket::from_parts(
        current.id(),
        mesh,
        current.asset_revision(),
        old.source_identity().clone(),
        current.demand_generation(),
        current.device(),
        current.scope(),
        current.route(),
    );
    assert!(matches!(
        residency.advance(&wrong_publication, RenderAssetResidencyState::Reading),
        Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
    ));
    let locator = resources.projection_snapshot().active().row(mesh.id())
        .expect("active fixture row").record.primary_locator.clone();
    assert!(resources
        .remove_by_locator(&locator)
        .expect("remove resource")
        .is_some());
    register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new());
    let readded = request_for(reconcile(&mut residency, &resources, mesh).requests(), mesh);
    assert_ne!(current.source_identity(), readded.source_identity());
    assert!(matches!(
        residency.advance(&current, RenderAssetResidencyState::Reading),
        Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
    ));
    assert_eq!(
        old_projection.active().row(mesh.id()).expect("retained active metadata").record.primary_locator,
        locator
    );
}

#[test]
fn mixed_catalog_readiness_revisions_fail_before_residency_mutation() {
    // Independent catalog/readiness inputs no longer exist. Keep this fixture and FQN,
    // and exercise the actual negative boundary: a source identity from an old capture
    // cannot advance or publish the pending ticket from a newer, paired capture.
    let resources = ResourceManager::new();
    let mesh = register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new());
    let mut residency = RenderAssetResidencyManager::new();
    let current = acquire(&mut residency, &resources, mesh);
    let old_projection = resources.projection_snapshot();
    let old_submission = begin_upload(&mut residency, &current);
    resources.register_ready(
        resource_record("identity/mesh", ResourceKind::Mesh, Vec::new()).with_source_hash("v2"),
        TestPayload { _label: "identity/mesh" },
    ).expect("publish changed root");
    let fresh = resources.projection_snapshot();
    assert_ne!(old_projection.active().key(mesh.id()), fresh.active().key(mesh.id()));
    let mutation = residency.reconcile_changed_resources(
        &[mesh], &fresh, device_epoch(41, 3), demand_generation(1),
    ).expect("fresh paired capture");
    let next = request_for(mutation.requests(), mesh);
    assert_eq!(next.id().raw(), current.id().raw() + 1);
    assert!(matches!(
        residency.advance(&current, RenderAssetResidencyState::Reading),
        Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
    ));
    assert!(matches!(
        residency.complete_upload(&current, old_submission, SubmissionStatus::Completed),
        Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
    ));
    assert_eq!(residency.pending_ticket(mesh).as_ref(), Some(&next));
    assert_eq!(residency.reference_count(mesh), 1);
    assert_eq!(residency.resident_ticket(mesh), None);
    let submission = begin_upload(&mut residency, &next);
    residency.complete_upload(&next, submission, SubmissionStatus::Completed)
        .expect("fresh identity can publish after stale rejection");
    assert_eq!(residency.resident_ticket(mesh).as_ref(), Some(&next));
}
