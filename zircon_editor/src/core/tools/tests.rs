use super::{
    AcquireDenial, AcquireOutcome, MAX_TOOL_DEFINITION_ID_BYTES, ReleaseOutcome, ToolDefinitionId,
    ToolDefinitionIdError, ToolInputCaptureDenial, ToolInputCaptureDisposition,
    ToolInputCaptureEvent, ToolInputCaptureOutcome, ToolInputCaptureOwner,
    ToolInputCapturePriority, ToolInputCaptureRequest, ToolInputScope, ToolInputSource,
    ToolInstanceId, ToolLeaseHandle, ToolLifecycleEvent, ToolOwnerGeneration, ToolQueueLimits,
    ToolRequestHandle, ToolResourceChannelId, ToolResourceKey, ToolResourceKindDeclaration,
    ToolResourceKindId, ToolResourceKindRegistration, ToolResourceKindRegistrationError,
    ToolResourceSet, ToolResourceSetError, ToolScheduleReport, ToolScheduler, ToolScope,
    WithdrawOutcome,
};
use zircon_runtime_interface::ui::dispatch::{
    UiPointerId, UiPointerSource, UiSurfaceId, UiWindowId,
};

use crate::core::editor_event::ViewInstanceId;

mod fairness;
mod snapshot;

#[test]
fn resource_sets_are_nonempty_deduplicated_and_canonically_sorted() {
    let viewport = viewport_input("viewport.main");
    let modal = modal_surface("window.main");
    let scene_mode = scene_mode_slot("viewport.main");
    let resources = ToolResourceSet::new([
        scene_mode.clone(),
        viewport.clone(),
        scene_mode.clone(),
        modal.clone(),
    ])
    .expect("a nonempty resource set should be valid");

    assert_eq!(resources.as_slice(), &[modal, scene_mode, viewport]);
    assert_eq!(ToolResourceSet::new([]), Err(ToolResourceSetError::Empty));
}

#[test]
fn resource_keys_require_builtin_scope_and_preserve_extension_channels() {
    let window = UiWindowId::new("window.main");
    let viewport_id = ViewInstanceId::new("editor.scene#1");
    let viewport = ToolResourceKey::viewport_input(viewport_id.clone());
    assert_eq!(viewport.scope().viewport_id(), Some(&viewport_id));
    assert!(
        ToolResourceKey::new(
            ToolResourceKindId::viewport_input(),
            ToolScope::Window {
                window_id: window.clone(),
            },
            None,
        )
        .is_err()
    );

    let kind = ToolResourceKindId::parse("example.plugin.brush-lock")
        .expect("a qualified extension resource kind should be valid");
    let channel = ToolResourceChannelId::parse("stylus.primary")
        .expect("a qualified resource channel should be valid");
    let extension = ToolResourceKey::new(
        kind.clone(),
        ToolScope::Window { window_id: window },
        Some(channel.clone()),
    )
    .expect("extension resources may use an explicit window scope and channel");

    assert_eq!(extension.kind(), &kind);
    assert_eq!(extension.channel(), Some(&channel));

    let invalid_serialized_key = serde_json::json!({
        "kind": "editor.viewport-input",
        "scope": { "Window": { "window_id": "window.main" } },
        "channel": null,
    });
    assert!(serde_json::from_value::<ToolResourceKey>(invalid_serialized_key).is_err());
}

#[test]
fn resource_kind_declaration_requires_a_nonempty_canonical_scope_set() {
    let kind = ToolResourceKindId::parse("plugin.sample.capture").unwrap();
    assert_eq!(
        ToolResourceKindDeclaration::new(
            kind.clone(),
            std::iter::empty::<super::ToolScopeKind>(),
            super::ToolResourceChannelPolicy::Optional,
        ),
        Err(ToolResourceKindRegistrationError::EmptySupportedScopes)
    );
    let declaration = ToolResourceKindDeclaration::new(
        kind,
        [super::ToolScopeKind::Window, super::ToolScopeKind::Window],
        super::ToolResourceChannelPolicy::Required,
    )
    .unwrap();
    let registration =
        ToolResourceKindRegistration::from_declaration(declaration, ToolOwnerGeneration::BUILTIN);
    assert_eq!(
        registration.supported_scopes(),
        [super::ToolScopeKind::Window]
    );

    let invalid = serde_json::json!({
        "kind": "plugin.sample.capture",
        "owner_generation": 1,
        "supported_scopes": [],
        "channel_policy": "optional",
    });
    assert!(serde_json::from_value::<ToolResourceKindRegistration>(invalid).is_err());
}

#[test]
fn same_resource_kind_in_distinct_viewports_can_be_held_concurrently() {
    let viewport_a = viewport_input("viewport.a");
    let viewport_b = viewport_input("viewport.b");
    let mut scheduler = ToolScheduler::default();

    let lease_a = acquired(scheduler.acquire(
        tool("scene.viewport.a"),
        ToolResourceSet::single(viewport_a.clone()),
    ));
    let lease_b = acquired(scheduler.acquire(
        tool("scene.viewport.b"),
        ToolResourceSet::single(viewport_b.clone()),
    ));

    assert_eq!(scheduler.holder(&viewport_a), Some(&lease_a));
    assert_eq!(scheduler.holder(&viewport_b), Some(&lease_b));
}

#[test]
fn modal_resources_conflict_per_window_instead_of_globally() {
    let main_window = modal_surface("window.main");
    let secondary_window = modal_surface("window.secondary");
    let mut scheduler = ToolScheduler::default();

    let main_lease = acquired(scheduler.acquire(
        tool("modal.main.owner"),
        ToolResourceSet::single(main_window.clone()),
    ));
    let queued_main = queued(scheduler.acquire(
        tool("modal.main.waiter"),
        ToolResourceSet::single(main_window.clone()),
    ));
    let secondary_lease = acquired(scheduler.acquire(
        tool("modal.secondary.owner"),
        ToolResourceSet::single(secondary_window.clone()),
    ));

    assert_eq!(scheduler.holder(&main_window), Some(&main_lease));
    assert_eq!(
        scheduler.queued_requests(&main_window).collect::<Vec<_>>(),
        [&queued_main]
    );
    assert_eq!(scheduler.holder(&secondary_window), Some(&secondary_lease));
}

#[test]
fn resource_set_requests_activate_atomically_and_preserve_fifo() {
    let scene = tool("scene.viewport.mode");
    let export = tool("workbench.build-export.windows");
    let scene_resources = resources([
        viewport_input("viewport.main"),
        scene_mode_slot("viewport.main"),
    ]);
    let export_resources = resources([
        viewport_input("viewport.main"),
        modal_surface("window.main"),
    ]);
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(4, 4));

    let scene_lease = acquired(scheduler.acquire(scene, scene_resources));
    let export_request = queued(scheduler.acquire(export, export_resources));
    let released = scheduler.release(scene_lease.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = released.outcome()
    else {
        panic!("scene lease should be released");
    };

    assert_eq!(activated_leases.len(), 1);
    assert_eq!(activated_leases[0].request_id(), export_request.id());
    for resource in export_request.resources().as_slice() {
        assert_eq!(
            scheduler.holder(resource).map(ToolLeaseHandle::id),
            Some(activated_leases[0].id())
        );
    }
}

#[test]
fn one_instance_cannot_replace_its_active_claim() {
    let instance = tool("scene.transform");
    let mut scheduler = ToolScheduler::default();
    let lease = acquired(scheduler.acquire(
        instance.clone(),
        ToolResourceSet::single(viewport_input("viewport.main")),
    ));

    let denied = scheduler.acquire(
        instance.clone(),
        ToolResourceSet::single(modal_surface("window.main")),
    );

    assert_eq!(
        denied.outcome(),
        &AcquireOutcome::Denied {
            holder: Some(lease.clone()),
            reason: AcquireDenial::AlreadyHeld {
                resources: lease.resources().clone(),
            },
        }
    );
    assert_eq!(scheduler.active_lease(&instance), Some(&lease));
}

#[test]
fn one_instance_cannot_replace_its_queued_claim() {
    let holder = tool("tool.holder");
    let queued_instance = tool("tool.queued");
    let mut scheduler = ToolScheduler::default();
    acquired(scheduler.acquire(
        holder,
        ToolResourceSet::single(viewport_input("viewport.main")),
    ));
    let request = queued(scheduler.acquire(
        queued_instance.clone(),
        ToolResourceSet::single(viewport_input("viewport.main")),
    ));

    let denied = scheduler.acquire(
        queued_instance.clone(),
        ToolResourceSet::single(modal_surface("window.main")),
    );

    assert_eq!(
        denied.outcome(),
        &AcquireOutcome::Denied {
            holder: None,
            reason: AcquireDenial::AlreadyQueued {
                resources: request.resources().clone(),
                position: 1,
            },
        }
    );
    assert_eq!(scheduler.pending_request(&queued_instance), Some(&request));
}

#[test]
fn repeated_same_claim_returns_the_canonical_handle() {
    let instance = tool("tool.repeat");
    let resources = ToolResourceSet::single(viewport_input("viewport.main"));
    let mut scheduler = ToolScheduler::default();
    let lease = acquired(scheduler.acquire(instance.clone(), resources.clone()));

    assert_eq!(
        scheduler.acquire(instance, resources).outcome(),
        &AcquireOutcome::AlreadyHeld { lease }
    );
}

#[test]
fn queue_full_denial_identifies_the_canonical_holder_lease() {
    let holder = tool("tool.holder");
    let queued_instance = tool("tool.queued");
    let denied_instance = tool("tool.denied");
    let resources = ToolResourceSet::single(viewport_input("viewport.main"));
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(1, 1));
    let holder_lease = acquired(scheduler.acquire(holder, resources.clone()));
    queued(scheduler.acquire(queued_instance, resources.clone()));

    let denied = scheduler.acquire(denied_instance.clone(), resources.clone());

    assert_eq!(
        denied.outcome(),
        &AcquireOutcome::Denied {
            holder: Some(holder_lease.clone()),
            reason: AcquireDenial::QueueFull { max_queued: 1 },
        }
    );
    assert_eq!(
        denied.events(),
        [ToolLifecycleEvent::Denied {
            instance: denied_instance,
            resources,
            holder: Some(holder_lease),
            reason: AcquireDenial::QueueFull { max_queued: 1 },
        }]
    );
}

#[test]
fn withdraw_removes_only_the_exact_request_id() {
    let resources = ToolResourceSet::single(modal_surface("window.main"));
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(2, 2));
    let holder = acquired(scheduler.acquire(tool("tool.holder"), resources.clone()));
    let removed = queued(scheduler.acquire(tool("tool.removed"), resources.clone()));
    let retained = queued(scheduler.acquire(tool("tool.retained"), resources));

    let withdrawn = scheduler.withdraw(removed.id());
    assert_eq!(
        withdrawn.outcome(),
        &WithdrawOutcome::Withdrawn {
            request: removed,
            previous_position: 1,
            activated_leases: Box::default(),
        }
    );

    let released = scheduler.release(holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = released.outcome()
    else {
        panic!("holder should release");
    };
    assert_eq!(activated_leases[0].request_id(), retained.id());
}

#[test]
fn stale_lease_cannot_release_a_newer_claim() {
    let instance = tool("tool.reacquired");
    let viewport = viewport_input("viewport.main");
    let resources = ToolResourceSet::single(viewport.clone());
    let mut scheduler = ToolScheduler::default();
    let stale = acquired(scheduler.acquire(instance.clone(), resources.clone()));
    assert!(matches!(
        scheduler.release(stale.id()).outcome(),
        ReleaseOutcome::Released { .. }
    ));
    let current = acquired(scheduler.acquire(instance, resources));

    assert_eq!(
        scheduler.release(stale.id()).outcome(),
        &ReleaseOutcome::NotHeld
    );
    assert_eq!(
        scheduler.holder(&viewport).map(ToolLeaseHandle::id),
        Some(current.id())
    );
}

#[test]
fn lifecycle_events_preserve_deactivation_then_activation_order() {
    let resources = ToolResourceSet::single(viewport_input("viewport.main"));
    let mut scheduler = ToolScheduler::default();
    let holder = acquired(scheduler.acquire(tool("tool.holder"), resources.clone()));
    let next = queued(scheduler.acquire(tool("tool.next"), resources));

    let report = scheduler.release(holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = report.outcome()
    else {
        panic!("holder should release");
    };
    assert_eq!(
        report.events(),
        [
            ToolLifecycleEvent::Deactivated { lease: holder },
            ToolLifecycleEvent::Activated {
                lease: activated_leases[0].clone(),
            },
        ]
    );
    assert_eq!(activated_leases[0].request_id(), next.id());
}

#[test]
fn tool_definition_and_instance_ids_enforce_distinct_contracts() {
    assert!(ToolDefinitionId::parse("").is_err());
    assert!(ToolDefinitionId::parse("scene transform").is_err());
    assert_eq!(
        ToolDefinitionId::parse("a".repeat(MAX_TOOL_DEFINITION_ID_BYTES + 1)),
        Err(ToolDefinitionIdError::TooLong {
            actual_bytes: MAX_TOOL_DEFINITION_ID_BYTES + 1,
            max_bytes: MAX_TOOL_DEFINITION_ID_BYTES,
        })
    );
    assert!(ToolInstanceId::from_parts("scene.transform", 0, 1).is_err());
    assert!(ToolInstanceId::from_parts("scene.transform", 1, 0).is_err());
    assert_eq!(tool("scene.transform").as_str(), "scene.transform@1.1");
    assert_eq!(
        tool("scene.transform").owner_generation(),
        ToolOwnerGeneration::BUILTIN
    );
    let maximum_instance =
        ToolInstanceId::from_parts("a".repeat(MAX_TOOL_DEFINITION_ID_BYTES), u64::MAX, u64::MAX)
            .unwrap();
    assert_eq!(
        maximum_instance.as_str().len(),
        super::MAX_TOOL_INSTANCE_ID_BYTES
    );
}

#[test]
fn request_and_lease_identity_preserve_owner_generation() {
    let generation = ToolOwnerGeneration::new(7).expect("non-zero owner generation");
    let resource = viewport_input("viewport.owner-generation");
    let mut scheduler = ToolScheduler::default();
    let holder = acquired(scheduler.acquire(
        tool("tool.owner-generation.holder"),
        ToolResourceSet::single(resource.clone()),
    ));
    let instance = ToolInstanceId::from_parts("plugin.sample.tool", generation.value(), 2)
        .expect("valid generation-owned instance");
    let request = queued(scheduler.acquire(instance.clone(), ToolResourceSet::single(resource)));

    assert_eq!(instance.owner_generation(), generation);
    assert_eq!(request.owner_generation(), generation);
    assert_eq!(
        serde_json::from_str::<ToolRequestHandle>(
            &serde_json::to_string(&request).expect("request should serialize")
        )
        .expect("request should deserialize")
        .owner_generation(),
        generation
    );

    let released = scheduler.release(holder.id());
    let ReleaseOutcome::Released {
        activated_leases, ..
    } = released.outcome()
    else {
        panic!("holder release should activate the queued request");
    };
    assert_eq!(activated_leases[0].owner_generation(), generation);
    assert_eq!(activated_leases[0].instance(), &instance);
}

#[test]
fn shutdown_releases_and_withdraws_every_claim_without_promotion() {
    let single_resources = ToolResourceSet::single(viewport_input("viewport.main"));
    let set_resources = resources([
        modal_surface("window.main"),
        scene_mode_slot("viewport.main"),
    ]);
    let mut scheduler = ToolScheduler::new(ToolQueueLimits::new(2, 2));
    acquired(scheduler.acquire(tool("tool.single-holder"), single_resources.clone()));
    queued(scheduler.acquire(tool("tool.single-waiter"), single_resources));
    acquired(scheduler.acquire(tool("tool.set-holder"), set_resources.clone()));
    queued(scheduler.acquire(tool("tool.set-waiter"), set_resources));

    let report = scheduler.shutdown();

    assert_eq!(report.outcome().released_single_leases(), 1);
    assert_eq!(report.outcome().released_set_leases(), 1);
    assert_eq!(report.outcome().withdrawn_single_requests(), 1);
    assert_eq!(report.outcome().withdrawn_set_requests(), 1);
    assert!(scheduler.snapshot().active_leases().is_empty());
    assert!(scheduler.snapshot().queued_requests().is_empty());
    assert!(
        scheduler
            .snapshot()
            .resources()
            .iter()
            .all(|state| { state.holder().is_none() && state.queued().is_empty() })
    );
}

#[test]
fn input_capture_rejects_a_lease_that_is_no_longer_active() {
    let mut scheduler = ToolScheduler::default();
    let lease = acquired(scheduler.acquire(
        tool("tool.capture.stale"),
        ToolResourceSet::single(viewport_input("viewport.main")),
    ));
    assert!(matches!(
        scheduler.release(lease.id()).outcome(),
        ReleaseOutcome::Released { .. }
    ));
    let request = capture_request(&lease, 1);

    let denied = scheduler.begin_input_capture(request);

    assert!(matches!(
        denied.outcome(),
        ToolInputCaptureOutcome::Denied {
            reason: ToolInputCaptureDenial::LeaseNotActive { lease_id },
            ..
        } if *lease_id == lease.id()
    ));
}

#[test]
fn input_capture_rejects_a_resource_outside_the_active_lease() {
    let mut scheduler = ToolScheduler::default();
    let lease = acquired(scheduler.acquire(
        tool("tool.capture.resource"),
        ToolResourceSet::single(modal_surface("window.main")),
    ));
    let request = ToolInputCaptureRequest::new(
        ToolInputCaptureOwner::from_lease(&lease),
        capture_source(1),
        viewport_input("viewport.main"),
        ToolInputCapturePriority::new(1),
    );

    let denied = scheduler.begin_input_capture(request);

    assert!(matches!(
        denied.outcome(),
        ToolInputCaptureOutcome::Denied {
            reason: ToolInputCaptureDenial::ResourceNotLeased {
                resource,
            },
            ..
        } if resource == &viewport_input("viewport.main")
    ));
    assert!(scheduler.snapshot().active_input_captures().is_empty());
}

#[test]
fn lease_release_ends_input_capture_before_tool_deactivation() {
    let mut scheduler = ToolScheduler::default();
    let lease = acquired(scheduler.acquire(
        tool("tool.capture.release"),
        ToolResourceSet::single(viewport_input("viewport.main")),
    ));
    let captured = scheduler.begin_input_capture(capture_request(&lease, 1));
    assert!(matches!(
        captured.outcome(),
        ToolInputCaptureOutcome::Captured { .. }
    ));

    let released = scheduler.release(lease.id());

    assert!(matches!(
        released.events().first(),
        Some(ToolLifecycleEvent::InputCapture {
            event: ToolInputCaptureEvent::Ended {
                disposition: ToolInputCaptureDisposition::OwnerLost,
                ..
            }
        })
    ));
    assert!(matches!(
        released.events().get(1),
        Some(ToolLifecycleEvent::Deactivated { lease: deactivated }) if deactivated == &lease
    ));
    assert!(scheduler.snapshot().active_input_captures().is_empty());
}

fn acquired(report: ToolScheduleReport<AcquireOutcome>) -> ToolLeaseHandle {
    match report.into_parts().0 {
        AcquireOutcome::Acquired { lease } | AcquireOutcome::AlreadyHeld { lease } => lease,
        outcome => panic!("expected an acquired lease, got {outcome:?}"),
    }
}

fn queued(report: ToolScheduleReport<AcquireOutcome>) -> ToolRequestHandle {
    match report.into_parts().0 {
        AcquireOutcome::Queued { request, .. } | AcquireOutcome::AlreadyQueued { request, .. } => {
            request
        }
        outcome => panic!("expected a queued request, got {outcome:?}"),
    }
}

fn tool(definition: &str) -> ToolInstanceId {
    ToolInstanceId::for_test(definition, ToolOwnerGeneration::BUILTIN).unwrap()
}

fn resources<const N: usize>(values: [ToolResourceKey; N]) -> ToolResourceSet {
    ToolResourceSet::new(values).expect("test resource sets should be nonempty")
}

fn capture_request(lease: &ToolLeaseHandle, pointer_id: u64) -> ToolInputCaptureRequest {
    ToolInputCaptureRequest::new(
        ToolInputCaptureOwner::from_lease(lease),
        capture_source(pointer_id),
        viewport_input("viewport.main"),
        ToolInputCapturePriority::new(1),
    )
}

fn capture_source(pointer_id: u64) -> ToolInputSource {
    ToolInputSource::Pointer {
        scope: ToolInputScope::new(
            UiWindowId::new("test.window"),
            UiSurfaceId::new("test.viewport"),
        ),
        user_id: None,
        device_id: None,
        pointer_id: Some(UiPointerId::new(pointer_id)),
        pointer_source: UiPointerSource::Mouse,
    }
}

fn viewport_input(viewport_id: &str) -> ToolResourceKey {
    ToolResourceKey::viewport_input(ViewInstanceId::new(viewport_id))
}

fn modal_surface(window_id: &str) -> ToolResourceKey {
    ToolResourceKey::modal_surface(UiWindowId::new(window_id))
}

fn scene_mode_slot(viewport_id: &str) -> ToolResourceKey {
    ToolResourceKey::scene_mode_slot(ViewInstanceId::new(viewport_id))
}
