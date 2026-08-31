use super::*;

#[test]
fn owner_generation_registry_is_bounded_before_identity_or_revision_changes() {
    let limits =
        ToolSchedulerLimits::with_owner_generation_capacity(ToolQueueLimits::new(1, 1), 1, 1)
            .unwrap();
    let scheduler = ToolSchedulerService::with_limits(
        crate::core::editor_message::SharedEditorMessageBus::default(),
        limits,
    );
    let initial = scheduler.snapshot();

    assert_eq!(
        initial.active_owner_generations(),
        [ToolOwnerGeneration::BUILTIN]
    );
    assert!(matches!(
        scheduler.register_owner_generation([]),
        Err(ToolSchedulerServiceError::OwnerGenerationCapacityReached {
            max_active_owner_generations: 1,
        })
    ));
    assert_eq!(scheduler.snapshot(), initial);
}

#[test]
fn owner_generation_revoke_is_capture_first_atomic_and_stale_safe() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    let registration = scheduler.register_owner_generation([]).unwrap();
    let generation = *registration.outcome();
    assert_ne!(generation, ToolOwnerGeneration::BUILTIN);
    assert!(matches!(
        registration.events(),
        [ToolLifecycleEvent::OwnerGenerationRegistered {
            generation: registered
        }] if *registered == generation
    ));

    let definition = ToolDefinitionId::parse("plugin.sample.viewport-tool").unwrap();
    let active = scheduler
        .allocate_instance_id(&definition, generation)
        .unwrap();
    let queued = scheduler
        .allocate_instance_id(&definition, generation)
        .unwrap();
    let builtin = scheduler
        .allocate_instance_id(&definition, ToolOwnerGeneration::BUILTIN)
        .unwrap();
    let resource_set = ToolResourceSet::single(viewport_resource());
    let active_lease = match scheduler
        .acquire(active, resource_set.clone())
        .unwrap()
        .into_parts()
        .0
    {
        AcquireOutcome::Acquired { lease } => lease,
        outcome => panic!("expected owner lease, got {outcome:?}"),
    };
    assert!(matches!(
        scheduler
            .acquire(queued, resource_set.clone())
            .unwrap()
            .outcome(),
        AcquireOutcome::Queued { .. }
    ));
    assert!(matches!(
        scheduler.acquire(builtin, resource_set).unwrap().outcome(),
        AcquireOutcome::Queued { .. }
    ));
    let source = ToolInputSource::Pointer {
        scope: ToolInputScope::new(
            UiWindowId::new("editor.main"),
            UiSurfaceId::new("editor.scene#1"),
        ),
        user_id: None,
        device_id: None,
        pointer_id: Some(UiPointerId::new(17)),
        pointer_source: UiPointerSource::Mouse,
    };
    assert!(matches!(
        scheduler
            .begin_input_capture(ToolInputCaptureRequest::new(
                ToolInputCaptureOwner::from_lease(&active_lease),
                source.clone(),
                viewport_resource(),
                ToolInputCapturePriority::new(10),
            ))
            .unwrap()
            .outcome(),
        ToolInputCaptureOutcome::Captured { .. }
    ));

    let revoked = scheduler.revoke_owner_generation(generation).unwrap();
    let ToolOwnerRevokeOutcome::Revoked {
        released_leases,
        withdrawn_requests,
        activated_leases,
        ..
    } = revoked.outcome()
    else {
        panic!("registered owner generation should revoke");
    };
    assert_eq!(released_leases.len(), 1);
    assert_eq!(withdrawn_requests.len(), 1);
    assert_eq!(activated_leases.len(), 1);
    assert_eq!(
        activated_leases[0].owner_generation(),
        ToolOwnerGeneration::BUILTIN
    );
    assert!(matches!(
        revoked.events().first(),
        Some(ToolLifecycleEvent::InputCapture {
            event: ToolInputCaptureEvent::Ended {
                disposition: ToolInputCaptureDisposition::OwnerLost,
                ..
            }
        })
    ));
    assert!(matches!(
        revoked.events().get(1),
        Some(ToolLifecycleEvent::Deactivated { lease })
            if lease.owner_generation() == generation
    ));
    assert!(matches!(
        revoked.events().get(2),
        Some(ToolLifecycleEvent::Withdrawn { request, .. })
            if request.owner_generation() == generation
    ));
    assert!(matches!(
        revoked.events().get(3),
        Some(ToolLifecycleEvent::Activated { lease })
            if lease.owner_generation() == ToolOwnerGeneration::BUILTIN
    ));
    assert!(matches!(
        revoked.events().last(),
        Some(ToolLifecycleEvent::OwnerGenerationRevoked {
            generation: revoked_generation
        }) if *revoked_generation == generation
    ));
    assert!(scheduler.active_input_capture(&source).is_none());
    assert!(
        !scheduler
            .snapshot()
            .active_owner_generations()
            .contains(&generation)
    );
    assert!(matches!(
        scheduler.allocate_instance_id(&definition, generation),
        Err(ToolSchedulerServiceError::OwnerGenerationUnavailable {
            generation: rejected
        }) if rejected == generation
    ));
    let forged = ToolInstanceId::from_parts(&definition.to_string(), generation.value(), 99)
        .expect("a stale serialized instance remains structurally valid");
    assert!(matches!(
        scheduler.acquire(
            forged,
            ToolResourceSet::single(ToolResourceKey::viewport_input(ViewInstanceId::new(
                "editor.scene#stale"
            )))
        ),
        Err(ToolSchedulerServiceError::OwnerGenerationUnavailable {
            generation: rejected
        }) if rejected == generation
    ));

    let revision = scheduler.snapshot().cursor().revision();
    assert!(matches!(
        scheduler
            .revoke_owner_generation(generation)
            .unwrap()
            .outcome(),
        ToolOwnerRevokeOutcome::NotRegistered {
            generation: missing
        } if *missing == generation
    ));
    assert_eq!(scheduler.snapshot().cursor().revision(), revision);
    assert!(matches!(
        scheduler
            .revoke_owner_generation(ToolOwnerGeneration::BUILTIN)
            .unwrap()
            .outcome(),
        ToolOwnerRevokeOutcome::BuiltinProtected
    ));
    assert_eq!(scheduler.snapshot().cursor().revision(), revision);
}

#[test]
fn resource_kind_owner_revoke_cleans_foreign_claims_and_unregisters_the_kind() {
    let scheduler =
        ToolSchedulerService::new(crate::core::editor_message::SharedEditorMessageBus::default());
    let initial = scheduler.snapshot();
    assert!(matches!(
        scheduler.register_owner_generation([
            crate::core::tools::ToolResourceKindDeclaration::new(
                ToolResourceKindId::parse("editor.extension-lock").unwrap(),
                [ToolScopeKind::Viewport],
                ToolResourceChannelPolicy::Forbidden,
            )
            .unwrap()
        ]),
        Err(ToolSchedulerServiceError::ResourceCatalog(
            ToolResourceCatalogError::ReservedBuiltinNamespace { .. }
        ))
    ));
    assert_eq!(scheduler.snapshot(), initial);
    let kind = ToolResourceKindId::parse("plugin.sample.viewport-lock").unwrap();
    let declaration = crate::core::tools::ToolResourceKindDeclaration::new(
        kind.clone(),
        [ToolScopeKind::Viewport],
        ToolResourceChannelPolicy::Forbidden,
    )
    .unwrap();
    let registered = scheduler
        .register_owner_generation([declaration.clone()])
        .unwrap();
    let provider = *registered.outcome();
    let registration =
        ToolResourceKindRegistration::from_declaration(declaration.clone(), provider);
    assert!(matches!(
        registered.events(),
        [
            ToolLifecycleEvent::OwnerGenerationRegistered {
                generation: registered_owner
            },
            ToolLifecycleEvent::ResourceKindRegistered {
            registration: published
            }
        ] if *registered_owner == provider && published == &registration
    ));
    let consumer = *scheduler.register_owner_generation([]).unwrap().outcome();
    let registration_revision = scheduler.snapshot().cursor().revision();
    let snapshot_before_duplicate = scheduler.snapshot();
    assert!(matches!(
        scheduler.register_owner_generation([declaration.clone()]),
        Err(ToolSchedulerServiceError::ResourceCatalog(
            ToolResourceCatalogError::DuplicateKind { kind: duplicate }
        )) if duplicate == kind
    ));
    assert_eq!(scheduler.snapshot(), snapshot_before_duplicate);

    let definition = ToolDefinitionId::parse("plugin.consumer.viewport-tool").unwrap();
    let instance = scheduler
        .allocate_instance_id(&definition, consumer)
        .unwrap();
    let resource = ToolResourceKey::new(
        kind.clone(),
        ToolScope::Viewport {
            viewport_id: ViewInstanceId::new("editor.scene#catalog"),
        },
        None,
    )
    .unwrap();
    let wrong_scope = ToolResourceKey::new(
        kind.clone(),
        ToolScope::Window {
            window_id: UiWindowId::new("editor.main"),
        },
        None,
    )
    .unwrap();
    assert!(matches!(
        scheduler.acquire(instance.clone(), ToolResourceSet::single(wrong_scope)),
        Err(ToolSchedulerServiceError::ResourceCatalog(
            ToolResourceCatalogError::UnsupportedScope {
                kind: rejected,
                actual: ToolScopeKind::Window,
                ..
            }
        )) if rejected == kind
    ));
    let channelled = ToolResourceKey::new(
        kind.clone(),
        ToolScope::Viewport {
            viewport_id: ViewInstanceId::new("editor.scene#catalog"),
        },
        Some(crate::core::tools::ToolResourceChannelId::parse("pointer.primary").unwrap()),
    )
    .unwrap();
    assert!(matches!(
        scheduler.acquire(instance.clone(), ToolResourceSet::single(channelled)),
        Err(ToolSchedulerServiceError::ResourceCatalog(
            ToolResourceCatalogError::ChannelForbidden { kind: rejected }
        )) if rejected == kind
    ));
    assert_eq!(
        scheduler.snapshot().cursor().revision(),
        registration_revision
    );
    let lease = match scheduler
        .acquire(instance.clone(), ToolResourceSet::single(resource.clone()))
        .unwrap()
        .into_parts()
        .0
    {
        AcquireOutcome::Acquired { lease } => lease,
        outcome => panic!("registered resource should acquire, got {outcome:?}"),
    };

    let revoked = scheduler.revoke_owner_generation(provider).unwrap();
    let ToolOwnerRevokeOutcome::Revoked {
        released_leases,
        revoked_resource_kinds,
        ..
    } = revoked.outcome()
    else {
        panic!("provider generation should revoke");
    };
    assert_eq!(released_leases.as_ref(), &[lease]);
    assert_eq!(revoked_resource_kinds.as_ref(), &[kind.clone()]);
    assert!(
        scheduler
            .snapshot()
            .active_owner_generations()
            .contains(&consumer)
    );
    assert!(
        scheduler
            .snapshot()
            .resource_catalog()
            .iter()
            .all(|entry| entry.kind() != &kind)
    );
    assert!(matches!(
        revoked.events().get(revoked.events().len().saturating_sub(2)),
        Some(ToolLifecycleEvent::ResourceKindsRevoked {
            owner_generation,
            kinds,
        }) if *owner_generation == provider && kinds.as_ref() == [kind.clone()]
    ));

    let revision = scheduler.snapshot().cursor().revision();
    assert!(matches!(
        scheduler.acquire(instance, ToolResourceSet::single(resource)),
        Err(ToolSchedulerServiceError::ResourceCatalog(
            ToolResourceCatalogError::UnregisteredKind { kind: missing }
        )) if missing == kind
    ));
    assert_eq!(scheduler.snapshot().cursor().revision(), revision);

    let replacement_provider = *scheduler
        .register_owner_generation([declaration])
        .unwrap()
        .outcome();
    assert_ne!(replacement_provider, provider);
}
