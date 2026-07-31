use super::{
    AcquireDenial, AcquireOutcome, AcquireSetOutcome, ExclusiveResource, ReleaseOutcome,
    ReleaseSetOutcome, ToolId, ToolIdError, ToolLifecycleEvent, ToolResourceSet,
    ToolResourceSetError, ToolScheduler, WithdrawOutcome, WithdrawSetOutcome, MAX_TOOL_ID_BYTES,
};

#[test]
fn resource_sets_are_nonempty_deduplicated_and_canonically_sorted() {
    let resources = ToolResourceSet::new([
        ExclusiveResource::SceneModeSlot,
        ExclusiveResource::ViewportInput,
        ExclusiveResource::SceneModeSlot,
        ExclusiveResource::ModalSurface,
    ])
    .expect("a nonempty resource set should be valid");

    assert_eq!(
        resources.as_slice(),
        &[
            ExclusiveResource::ViewportInput,
            ExclusiveResource::ModalSurface,
            ExclusiveResource::SceneModeSlot,
        ]
    );
    assert_eq!(ToolResourceSet::new([]), Err(ToolResourceSetError::Empty));
}

#[test]
fn set_requests_activate_atomically_and_preserve_global_fifo() {
    let scene = tool("scene.viewport.mode");
    let export = tool("workbench.build_export.windows");
    let tail = tool("workbench.build_export.mac");
    let scene_resources = resources([
        ExclusiveResource::ViewportInput,
        ExclusiveResource::SceneModeSlot,
    ]);
    let export_resources = resources([
        ExclusiveResource::ModalSurface,
        ExclusiveResource::ViewportInput,
    ]);
    let tail_resources = resources([ExclusiveResource::ModalSurface]);
    let mut scheduler = ToolScheduler::new(4);

    assert_eq!(
        scheduler
            .acquire_set(scene.clone(), scene_resources.clone())
            .outcome(),
        &AcquireSetOutcome::Acquired
    );
    assert_eq!(
        scheduler
            .acquire_set(export.clone(), export_resources.clone())
            .outcome(),
        &AcquireSetOutcome::Queued { position: 1 }
    );
    assert_eq!(
        scheduler
            .acquire_set(tail.clone(), tail_resources.clone())
            .outcome(),
        &AcquireSetOutcome::Queued { position: 2 }
    );

    let release_scene = scheduler.release_set(&scene, &scene_resources);
    assert_eq!(
        release_scene.outcome(),
        &ReleaseSetOutcome::Released {
            activated: Some(export.clone())
        }
    );
    assert_eq!(
        release_scene.events(),
        [
            ToolLifecycleEvent::SetDeactivated {
                tool: scene,
                resources: scene_resources,
            },
            ToolLifecycleEvent::SetActivated {
                tool: export.clone(),
                resources: export_resources.clone(),
            },
        ]
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ViewportInput),
        Some(&export)
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ModalSurface),
        Some(&export)
    );
    assert_eq!(scheduler.holder(ExclusiveResource::SceneModeSlot), None);

    assert_eq!(
        scheduler.release_set(&export, &export_resources).outcome(),
        &ReleaseSetOutcome::Released {
            activated: Some(tail.clone())
        }
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ModalSurface),
        Some(&tail)
    );
}

#[test]
fn set_queue_head_cannot_be_bypassed_by_a_later_free_subset() {
    let scene = tool("scene.viewport.mode");
    let export = tool("workbench.build_export.windows");
    let tail = tool("workbench.build_export.mac");
    let scene_resources = resources([
        ExclusiveResource::ViewportInput,
        ExclusiveResource::SceneModeSlot,
    ]);
    let export_resources = resources([
        ExclusiveResource::ViewportInput,
        ExclusiveResource::ModalSurface,
    ]);
    let tail_resources = resources([ExclusiveResource::ModalSurface]);
    let mut scheduler = ToolScheduler::new(4);

    scheduler.acquire_set(scene.clone(), scene_resources.clone());
    scheduler.acquire_set(export.clone(), export_resources.clone());
    let queued_tail = scheduler.acquire_set(tail.clone(), tail_resources.clone());

    assert_eq!(
        queued_tail.outcome(),
        &AcquireSetOutcome::Queued { position: 2 }
    );
    assert_eq!(scheduler.holder(ExclusiveResource::ModalSurface), None);

    scheduler.release_set(&scene, &scene_resources);

    assert_eq!(
        scheduler.holder(ExclusiveResource::ViewportInput),
        Some(&export)
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ModalSurface),
        Some(&export)
    );
    assert_ne!(
        scheduler.holder(ExclusiveResource::ModalSurface),
        Some(&tail)
    );
}

#[test]
fn single_resource_release_cannot_partially_release_an_active_set() {
    let scene = tool("scene.viewport.mode");
    let scene_resources = resources([
        ExclusiveResource::ViewportInput,
        ExclusiveResource::SceneModeSlot,
    ]);
    let mut scheduler = ToolScheduler::new(4);

    scheduler.acquire_set(scene.clone(), scene_resources.clone());
    let release = scheduler.release(&scene, ExclusiveResource::ViewportInput);

    assert_eq!(
        release.outcome(),
        &ReleaseOutcome::SetHeld {
            resources: scene_resources,
        }
    );
    assert!(release.events().is_empty());
    assert_eq!(
        scheduler.holder(ExclusiveResource::ViewportInput),
        Some(&scene)
    );
}

#[test]
fn withdrawing_a_set_request_promotes_the_next_eligible_set() {
    let scene = tool("scene.viewport.mode");
    let export = tool("workbench.build_export.windows");
    let tail = tool("workbench.build_export.mac");
    let scene_resources = resources([ExclusiveResource::ViewportInput]);
    let export_resources = resources([
        ExclusiveResource::ViewportInput,
        ExclusiveResource::ModalSurface,
    ]);
    let tail_resources = resources([ExclusiveResource::ModalSurface]);
    let mut scheduler = ToolScheduler::new(4);

    scheduler.acquire_set(scene, scene_resources);
    scheduler.acquire_set(export.clone(), export_resources.clone());
    scheduler.acquire_set(tail.clone(), tail_resources.clone());

    let withdrawn = scheduler.withdraw_set(&export, &export_resources);

    assert_eq!(
        withdrawn.outcome(),
        &WithdrawSetOutcome::Withdrawn {
            previous_position: 1,
            activated: Some(tail.clone()),
        }
    );
    assert_eq!(
        withdrawn.events(),
        [
            ToolLifecycleEvent::SetWithdrawn {
                tool: export,
                resources: export_resources,
                previous_position: 1,
            },
            ToolLifecycleEvent::SetActivated {
                tool: tail.clone(),
                resources: tail_resources,
            },
        ]
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ModalSurface),
        Some(&tail)
    );
}

#[test]
fn acquire_is_idempotent_for_the_current_holder() {
    let tool = tool("scene.select");
    let mut scheduler = ToolScheduler::new(2);

    assert_eq!(
        scheduler
            .acquire(tool.clone(), ExclusiveResource::ViewportInput)
            .outcome(),
        &AcquireOutcome::Acquired
    );
    let repeated = scheduler.acquire(tool.clone(), ExclusiveResource::ViewportInput);

    assert_eq!(repeated.outcome(), &AcquireOutcome::AlreadyHeld);
    assert!(repeated.events().is_empty());
    assert_eq!(
        scheduler.holder(ExclusiveResource::ViewportInput),
        Some(&tool)
    );
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::ViewportInput)
            .count(),
        0
    );
}

#[test]
fn contended_tools_activate_in_fifo_order() {
    let first = tool("tool.first");
    let second = tool("tool.second");
    let third = tool("tool.third");
    let mut scheduler = ToolScheduler::new(2);

    scheduler.acquire(first.clone(), ExclusiveResource::ModalSurface);
    assert_eq!(
        scheduler
            .acquire(second.clone(), ExclusiveResource::ModalSurface)
            .outcome(),
        &AcquireOutcome::Queued { position: 1 }
    );
    assert_eq!(
        scheduler
            .acquire(third.clone(), ExclusiveResource::ModalSurface)
            .outcome(),
        &AcquireOutcome::Queued { position: 2 }
    );

    assert_eq!(
        scheduler
            .release(&first, ExclusiveResource::ModalSurface)
            .outcome(),
        &ReleaseOutcome::Released {
            activated: Some(second.clone())
        }
    );
    assert_eq!(
        scheduler
            .release(&second, ExclusiveResource::ModalSurface)
            .outcome(),
        &ReleaseOutcome::Released {
            activated: Some(third.clone())
        }
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ModalSurface),
        Some(&third)
    );
}

#[test]
fn duplicate_queued_acquire_does_not_grow_the_queue() {
    let holder = tool("tool.holder");
    let queued = tool("tool.queued");
    let mut scheduler = ToolScheduler::new(2);
    scheduler.acquire(holder, ExclusiveResource::SceneModeSlot);
    scheduler.acquire(queued.clone(), ExclusiveResource::SceneModeSlot);

    let repeated = scheduler.acquire(queued.clone(), ExclusiveResource::SceneModeSlot);

    assert_eq!(
        repeated.outcome(),
        &AcquireOutcome::AlreadyQueued { position: 1 }
    );
    assert!(repeated.events().is_empty());
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::SceneModeSlot)
            .collect::<Vec<_>>(),
        [&queued]
    );
}

#[test]
fn full_queue_returns_typed_denial_without_mutation() {
    let holder = tool("tool.holder");
    let queued = tool("tool.queued");
    let denied = tool("tool.denied");
    let mut scheduler = ToolScheduler::new(1);
    scheduler.acquire(holder.clone(), ExclusiveResource::ViewportInput);
    scheduler.acquire(queued.clone(), ExclusiveResource::ViewportInput);

    let report = scheduler.acquire(denied.clone(), ExclusiveResource::ViewportInput);

    assert_eq!(
        report.outcome(),
        &AcquireOutcome::Denied {
            holder: holder.clone(),
            reason: AcquireDenial::QueueFull { max_queued: 1 }
        }
    );
    assert_eq!(
        report.events(),
        [ToolLifecycleEvent::Denied {
            tool: denied,
            resource: ExclusiveResource::ViewportInput,
            holder,
            reason: AcquireDenial::QueueFull { max_queued: 1 },
        }]
    );
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::ViewportInput)
            .collect::<Vec<_>>(),
        [&queued]
    );
}

#[test]
fn withdraw_removes_only_the_callers_pending_request() {
    let holder = tool("tool.holder");
    let withdrawn = tool("tool.withdrawn");
    let retained = tool("tool.retained");
    let mut scheduler = ToolScheduler::new(2);
    scheduler.acquire(holder.clone(), ExclusiveResource::ModalSurface);
    scheduler.acquire(withdrawn.clone(), ExclusiveResource::ModalSurface);
    scheduler.acquire(retained.clone(), ExclusiveResource::ModalSurface);

    assert_eq!(
        scheduler
            .withdraw(&withdrawn, ExclusiveResource::ModalSurface)
            .outcome(),
        &WithdrawOutcome::Withdrawn {
            previous_position: 1
        }
    );
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::ModalSurface)
            .collect::<Vec<_>>(),
        [&retained]
    );
    assert_eq!(
        scheduler
            .release(&holder, ExclusiveResource::ModalSurface)
            .outcome(),
        &ReleaseOutcome::Released {
            activated: Some(retained)
        }
    );
}

#[test]
fn release_all_clears_owned_and_queued_resources() {
    let target = tool("tool.target");
    let next = tool("tool.next");
    let viewport_tail = tool("tool.viewport-tail");
    let modal_holder = tool("tool.modal-holder");
    let modal_before = tool("tool.modal-before");
    let modal_after = tool("tool.modal-after");
    let scene_holder = tool("tool.scene-holder");
    let scene_after = tool("tool.scene-after");
    let mut scheduler = ToolScheduler::new(4);
    scheduler.acquire(target.clone(), ExclusiveResource::ViewportInput);
    scheduler.acquire(next.clone(), ExclusiveResource::ViewportInput);
    scheduler.acquire(viewport_tail.clone(), ExclusiveResource::ViewportInput);
    scheduler.acquire(modal_holder.clone(), ExclusiveResource::ModalSurface);
    scheduler.acquire(modal_before.clone(), ExclusiveResource::ModalSurface);
    scheduler.acquire(target.clone(), ExclusiveResource::ModalSurface);
    scheduler.acquire(modal_after.clone(), ExclusiveResource::ModalSurface);
    scheduler.acquire(scene_holder.clone(), ExclusiveResource::SceneModeSlot);
    scheduler.acquire(target.clone(), ExclusiveResource::SceneModeSlot);
    scheduler.acquire(scene_after.clone(), ExclusiveResource::SceneModeSlot);

    let report = scheduler.release_all(&target);

    assert_eq!(
        report.outcome().released_resources,
        [ExclusiveResource::ViewportInput]
    );
    assert_eq!(
        report.outcome().withdrawn_resources,
        [
            ExclusiveResource::ModalSurface,
            ExclusiveResource::SceneModeSlot
        ]
    );
    assert_eq!(
        report.outcome().activated_tools,
        [(ExclusiveResource::ViewportInput, next.clone())]
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ViewportInput),
        Some(&next)
    );
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::ViewportInput)
            .collect::<Vec<_>>(),
        [&viewport_tail]
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::ModalSurface),
        Some(&modal_holder)
    );
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::ModalSurface)
            .collect::<Vec<_>>(),
        [&modal_before, &modal_after]
    );
    assert_eq!(
        scheduler.holder(ExclusiveResource::SceneModeSlot),
        Some(&scene_holder)
    );
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::SceneModeSlot)
            .collect::<Vec<_>>(),
        [&scene_after]
    );
}

#[test]
fn lifecycle_events_preserve_release_then_activation_order() {
    let holder = tool("tool.holder");
    let next = tool("tool.next");
    let mut scheduler = ToolScheduler::new(1);
    scheduler.acquire(holder.clone(), ExclusiveResource::ViewportInput);
    scheduler.acquire(next.clone(), ExclusiveResource::ViewportInput);

    let report = scheduler.release(&holder, ExclusiveResource::ViewportInput);

    assert_eq!(
        report.events(),
        [
            ToolLifecycleEvent::Deactivated {
                tool: holder,
                resource: ExclusiveResource::ViewportInput,
            },
            ToolLifecycleEvent::Activated {
                tool: next,
                resource: ExclusiveResource::ViewportInput,
            },
        ]
    );
}

#[test]
fn non_owner_release_and_missing_withdraw_are_side_effect_free() {
    let holder = tool("tool.holder");
    let queued = tool("tool.queued");
    let unrelated = tool("tool.unrelated");
    let mut scheduler = ToolScheduler::new(2);
    scheduler.acquire(holder.clone(), ExclusiveResource::ViewportInput);
    scheduler.acquire(queued.clone(), ExclusiveResource::ViewportInput);

    let release = scheduler.release(&unrelated, ExclusiveResource::ViewportInput);
    let withdraw = scheduler.withdraw(&unrelated, ExclusiveResource::ViewportInput);

    assert_eq!(
        release.outcome(),
        &ReleaseOutcome::NotHolder {
            holder: holder.clone()
        }
    );
    assert!(release.events().is_empty());
    assert_eq!(withdraw.outcome(), &WithdrawOutcome::NotQueued);
    assert!(withdraw.events().is_empty());
    assert_eq!(
        scheduler.holder(ExclusiveResource::ViewportInput),
        Some(&holder)
    );
    assert_eq!(
        scheduler
            .queued_tools(ExclusiveResource::ViewportInput)
            .collect::<Vec<_>>(),
        [&queued]
    );
}

#[test]
fn tool_id_rejects_empty_and_non_contract_characters() {
    assert!(ToolId::parse("").is_err());
    assert!(ToolId::parse("scene transform").is_err());
    assert_eq!(
        ToolId::parse("a".repeat(MAX_TOOL_ID_BYTES + 1)),
        Err(ToolIdError::TooLong {
            actual_bytes: MAX_TOOL_ID_BYTES + 1,
            max_bytes: MAX_TOOL_ID_BYTES,
        })
    );
    assert_eq!(tool("scene.transform").as_str(), "scene.transform");
}

fn tool(value: &str) -> ToolId {
    ToolId::parse(value).unwrap()
}

fn resources<const N: usize>(values: [ExclusiveResource; N]) -> ToolResourceSet {
    ToolResourceSet::new(values).expect("test resource sets should be nonempty")
}
