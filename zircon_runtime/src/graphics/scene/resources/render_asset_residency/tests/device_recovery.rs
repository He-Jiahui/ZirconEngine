use super::*;
use crate::graphics::scene::resources::render_asset_residency::{
    RenderAssetDeviceRecoveryError, RenderAssetResidencyAdmissionError,
};

#[test]
fn render_device_recovery_reissues_live_resources_and_terminalizes_old_projection_once() {
    let resources = ResourceManager::new();
    let texture = register_resource(
        &resources,
        "recovery/texture.ktx2",
        ResourceKind::Texture,
        Vec::new(),
    );
    let mesh = register_resource(
        &resources,
        "recovery/mesh.zmesh",
        ResourceKind::Mesh,
        Vec::new(),
    );
    let failed = device_epoch(41, 3);
    let replacement = device_epoch(41, 4);
    let mut residency = RenderAssetResidencyManager::new();
    let admitted = residency
        .apply_scene_reference_deltas(
            &[
                RenderSceneResourceReferenceDelta::acquire(texture, 1),
                RenderSceneResourceReferenceDelta::acquire(mesh, 2),
            ],
            &resources.management_generation(),
            &resources.readiness_generation(),
            failed,
            demand_generation(7),
        )
        .expect("recovery fixture admission should succeed");
    assert_eq!(admitted.requests().len(), 2);

    let texture_ticket = request_for(admitted.requests(), texture);
    advance_to_upload(&mut residency, texture_ticket);
    let texture_submission = SubmissionTicket::new(
        failed.device_id(),
        failed.generation(),
        RenderQueueClass::Copy,
        1,
    );
    residency
        .bind_upload_submission(texture_ticket, texture_submission)
        .expect("texture upload should bind");
    residency
        .complete_upload(
            texture_ticket,
            texture_submission,
            SubmissionStatus::Completed,
        )
        .expect("texture upload should publish");

    let mesh_ticket = request_for(admitted.requests(), mesh);
    advance_to_upload(&mut residency, mesh_ticket);
    let mesh_submission = SubmissionTicket::new(
        failed.device_id(),
        failed.generation(),
        RenderQueueClass::Copy,
        2,
    );
    residency
        .bind_upload_submission(mesh_ticket, mesh_submission)
        .expect("mesh upload should bind");

    let report = residency
        .recover_device_epoch(
            failed,
            replacement,
            &resources.management_generation(),
            &resources.readiness_generation(),
            demand_generation(8),
        )
        .expect("device recovery should commit atomically");

    assert_eq!(report.failed_epoch(), failed);
    assert_eq!(report.replacement_epoch(), replacement);
    assert_eq!(report.mutation().requests().len(), 2);
    assert_eq!(report.mutation().releases().len(), 2);
    assert_eq!(residency.reference_count(texture), 1);
    assert_eq!(residency.reference_count(mesh), 2);
    assert_eq!(residency.resident_ticket(texture), None);
    assert_eq!(residency.resident_ticket(mesh), None);
    assert_eq!(residency.state(texture_ticket), None);
    assert_eq!(residency.state(mesh_ticket), None);
    for resource in [texture, mesh] {
        let ticket = request_for(report.mutation().requests(), resource);
        assert_eq!(ticket.device(), replacement);
        assert_eq!(
            residency.state(ticket),
            Some(RenderAssetResidencyState::QueuedIo)
        );
    }
    assert!(report.mutation().releases().iter().any(|release| {
        release.ticket() == texture_ticket
            && release.kind() == RenderAssetResidencyReleaseKind::RetireResident
    }));
    assert!(report.mutation().releases().iter().any(|release| {
        release.ticket() == mesh_ticket
            && release.kind() == RenderAssetResidencyReleaseKind::RetireInFlight
    }));
}

#[test]
fn render_device_recovery_preflight_is_atomic_and_does_not_consume_ticket_ids() {
    let resources = ResourceManager::new();
    let texture = register_resource(
        &resources,
        "recovery/atomic.ktx2",
        ResourceKind::Texture,
        Vec::new(),
    );
    let failed = device_epoch(52, 9);
    let replacement = device_epoch(52, 10);
    let mut residency = RenderAssetResidencyManager::new();
    let admitted = residency
        .apply_scene_reference_deltas(
            &[RenderSceneResourceReferenceDelta::acquire(texture, 1)],
            &resources.management_generation(),
            &resources.readiness_generation(),
            failed,
            demand_generation(1),
        )
        .expect("recovery fixture admission should succeed");
    let old_ticket = request_for(admitted.requests(), texture);

    let premature_release = residency.apply_scene_reference_deltas(
        &[RenderSceneResourceReferenceDelta::release(texture, 1)],
        &resources.management_generation(),
        &resources.readiness_generation(),
        replacement,
        demand_generation(2),
    );
    assert!(matches!(
        premature_release,
        Err(RenderAssetResidencyAdmissionError::DeviceEpochMismatch { .. })
    ));
    assert_eq!(residency.reference_count(texture), 1);
    assert_eq!(residency.pending_ticket(texture), Some(old_ticket));

    let unchanged = residency.recover_device_epoch(
        failed,
        failed,
        &resources.management_generation(),
        &resources.readiness_generation(),
        demand_generation(2),
    );
    assert!(matches!(
        unchanged,
        Err(RenderAssetDeviceRecoveryError::UnchangedEpoch { .. })
    ));
    assert_eq!(residency.pending_ticket(texture), Some(old_ticket));

    let stale_replacement = residency.recover_device_epoch(
        failed,
        device_epoch(52, 8),
        &resources.management_generation(),
        &resources.readiness_generation(),
        demand_generation(2),
    );
    assert!(matches!(
        stale_replacement,
        Err(RenderAssetDeviceRecoveryError::NonAdvancingGeneration { .. })
    ));
    assert_eq!(residency.pending_ticket(texture), Some(old_ticket));

    let wrong_failed = device_epoch(999, 1);
    let foreign = residency.recover_device_epoch(
        wrong_failed,
        replacement,
        &resources.management_generation(),
        &resources.readiness_generation(),
        demand_generation(2),
    );
    assert!(matches!(
        foreign,
        Err(RenderAssetDeviceRecoveryError::GpuEpochMismatch { .. })
    ));
    assert_eq!(residency.pending_ticket(texture), Some(old_ticket));

    let recovered = residency
        .recover_device_epoch(
            failed,
            replacement,
            &resources.management_generation(),
            &resources.readiness_generation(),
            demand_generation(2),
        )
        .expect("valid recovery should succeed after rejected preflights");
    let replacement_ticket = request_for(recovered.mutation().requests(), texture);
    assert_eq!(replacement_ticket.id().raw(), old_ticket.id().raw() + 1);
}
