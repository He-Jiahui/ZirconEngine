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
            projection.management(),
            projection.readiness(),
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
            projection.management(),
            projection.readiness(),
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
    assert_ne!(current.readiness_identity(), old.readiness_identity());
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
    let (mesh, old) = {
        let resources = ResourceManager::new();
        let mesh = register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new());
        (mesh, acquire(&mut residency, &resources, mesh))
    };
    let resources = ResourceManager::new();
    assert_eq!(
        register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new()),
        mesh
    );
    let mutation = reconcile(&mut residency, &resources, mesh);
    let current = request_for(mutation.requests(), mesh);
    assert_eq!(old.asset_revision(), current.asset_revision());
    assert_ne!(old.readiness_identity(), current.readiness_identity());
    let wrong_publication = RenderAssetResidencyTicket::from_parts(
        current.id(),
        mesh,
        current.asset_revision(),
        old.readiness_identity().clone(),
        current.demand_generation(),
        current.device(),
        current.scope(),
        current.route(),
    );
    assert!(matches!(
        residency.advance(&wrong_publication, RenderAssetResidencyState::Reading),
        Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
    ));
    let locator = current
        .readiness_identity()
        .row()
        .record
        .primary_locator
        .clone();
    assert!(resources
        .remove_by_locator(&locator)
        .expect("remove resource")
        .is_some());
    register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new());
    let readded = request_for(reconcile(&mut residency, &resources, mesh).requests(), mesh);
    assert_ne!(current.readiness_identity(), readded.readiness_identity());
    assert!(matches!(
        residency.advance(&current, RenderAssetResidencyState::Reading),
        Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
    ));
    assert_eq!(
        old.readiness_identity().row().record.primary_locator,
        locator
    );
}

#[test]
fn mixed_catalog_readiness_revisions_fail_before_residency_mutation() {
    let resources = ResourceManager::new();
    let mesh = register_resource(&resources, "identity/mesh", ResourceKind::Mesh, Vec::new());
    let mut residency = RenderAssetResidencyManager::new();
    let current = acquire(&mut residency, &resources, mesh);
    let old_management = resources.management_generation();
    resources
        .register_ready(
            resource_record("identity/mesh", ResourceKind::Mesh, Vec::new()).with_source_hash("v2"),
            TestPayload {
                _label: "identity/mesh",
            },
        )
        .expect("publish changed root");
    let readiness = resources.readiness_generation();
    let newer_revision = readiness
        .row_identity(mesh.id())
        .expect("updated row")
        .row()
        .record
        .revision;
    assert_ne!(newer_revision, current.asset_revision());
    let result = residency.reconcile_changed_resources(
        &[mesh],
        &old_management,
        &readiness,
        device_epoch(41, 3),
        demand_generation(1),
    );
    assert_eq!(
        result,
        Err(
            RenderAssetResidencyAdmissionError::CatalogReadinessRevisionMismatch {
                resource: mesh,
                catalog_revision: current.asset_revision(),
                readiness_revision: newer_revision,
            }
        )
    );
    assert_eq!(residency.pending_ticket(mesh).as_ref(), Some(&current));
    assert_eq!(residency.reference_count(mesh), 1);
    let next = request_for(reconcile(&mut residency, &resources, mesh).requests(), mesh);
    assert_eq!(next.id().raw(), current.id().raw() + 1);
}
