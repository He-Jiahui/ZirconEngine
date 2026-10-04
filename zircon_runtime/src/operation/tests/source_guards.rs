#[test]
fn owner_apply_claim_rechecks_deadline_before_mutating_world() {
    let source = include_str!("../service.rs");
    let start = source
        .find("    fn take_prepared_task(")
        .expect("owner apply claim helper");
    let end = source[start..]
        .find("    fn finish_completed_task(")
        .map(|offset| start + offset)
        .expect("owner apply completion helper");
    let claim_source = &source[start..end];

    assert!(claim_source.contains("task.deadline.is_none_or(|deadline| deadline > now)"));
    assert!(claim_source.contains("!task.apply_claimed"));
}

#[test]
fn deadline_admission_uses_the_shared_timer_and_arms_before_dispatch() {
    let source = include_str!("../service.rs");
    let start = source
        .find("    fn arm_deadline(")
        .expect("deadline arming owner");
    let end = source[start..]
        .find("    fn rollback_unarmed_admission(")
        .map(|offset| start + offset)
        .expect("deadline rollback boundary");
    let arm_source = &source[start..end];

    assert!(arm_source.contains("self.refresh_maintenance()"));
    assert!(arm_source.contains("task.deadline_armed = true"));
    assert!(!source.contains("deadline_subscription"));
}

#[test]
fn queued_phase_index_retains_live_admissions_until_deadline_arming_completes() {
    let source = include_str!("../service.rs");
    let start = source
        .find("    fn take_queued_snapshot_task(")
        .expect("queued snapshot claim helper");
    let end = source[start..]
        .find("    fn finish_snapshot_failed_task(")
        .map(|offset| start + offset)
        .expect("queued snapshot claim boundary");
    let claim_source = &source[start..end];

    assert!(claim_source.contains("let candidate = *state.queued_snapshot_tasks.front()?"));
    assert!(claim_source.contains("live && !task.deadline_armed"));
    assert!(claim_source.contains("if retain"));
    assert!(claim_source.contains("return None"));
    assert!(!claim_source.contains("push_back(candidate)"));
}

#[test]
fn operation_maintenance_timer_is_service_scoped_and_rearms_deadlines_and_ttl() {
    let task_state_source = include_str!("../service/task_state.rs");
    let source = include_str!("../maintenance.rs");
    let start = source
        .find("fn refresh_operation_maintenance_alarm(")
        .expect("service maintenance timer owner");
    let end = source[start..]
        .find("fn next_maintenance_deadline(")
        .map(|offset| start + offset)
        .expect("maintenance deadline selection boundary");
    let maintenance_source = &source[start..end];

    assert!(task_state_source.contains("maintenance_subscription: Option<TaskTimerSubscription>"));
    assert!(task_state_source.contains("maintenance_deadline: Option<Instant>"));
    assert!(maintenance_source.contains("TaskTimer::process_default()"));
    assert!(maintenance_source.contains("Arc::downgrade(state)"));
    assert!(maintenance_source.contains("expire_due_deadlines_in_state"));
    assert!(maintenance_source.contains("expire_terminal_results_in_state"));
    assert!(maintenance_source.contains("refresh_operation_maintenance_alarm("));
}

#[test]
fn raw_admission_release_preserves_exact_count_and_byte_invariants() {
    let source = include_str!("../service/admission.rs");
    let start = source
        .find("fn consume_raw_admission(")
        .expect("raw admission release owner");
    let release_source = &source[start..];

    assert!(release_source.contains("checked_sub(1)"));
    assert!(release_source.contains("checked_sub(reservation.bytes)"));
    assert!(!release_source.contains("saturating_sub"));
}

#[test]
fn terminal_transitions_release_retained_bytes_with_checked_accounting() {
    let service_source = include_str!("../service.rs");
    let maintenance_source = include_str!("../maintenance.rs");

    for owner in [
        "    pub fn cancel(",
        "    pub fn harvest(",
        "    fn rollback_unarmed_admission(",
        "    fn finish_failed_task(",
    ] {
        let start = service_source
            .find(owner)
            .expect("service transition owner");
        let transition_source = &service_source[start..];
        assert!(transition_source.contains("checked_sub("));
    }
    assert!(maintenance_source.contains("checked_sub(released_bytes)"));
    assert!(!maintenance_source.contains("saturating_sub(released_bytes)"));
}

#[test]
fn operation_handler_hard_cut_requires_snapshot_prepared_result_and_unit_apply() {
    let source = include_str!("../handler.rs");
    assert!(source.contains("fn snapshot("));
    assert!(source.contains("RuntimeOperationPrepared"));
    assert!(source.contains("fn apply("));
    assert!(source.contains("Result<(), RuntimeOperationHandlerError>"));
    assert!(!source.contains("fn apply(\n        &self,\n        context: RuntimeOperationContext<'_>,\n        prepared: serde_json::Value,\n    ) -> Result<serde_json::Value"));
}

#[test]
fn completion_reserves_result_and_converts_channel_loss_to_a_terminal_failure() {
    let completion_source = include_str!("../service/completion.rs");
    let service_source = include_str!("../service.rs");
    let apply_start = service_source
        .find("    fn apply_prepared(")
        .expect("owner apply boundary");
    let apply_end = service_source[apply_start..]
        .find("    fn take_prepared_task(")
        .map(|offset| apply_start + offset)
        .expect("owner apply end");
    let apply_source = &service_source[apply_start..apply_end];
    let navigation_source = include_str!("../../navigation/operation/handler.rs");

    assert!(completion_source.contains("prepared_command = Some(command)"));
    assert!(completion_source.contains("prepared_result = Some(result)"));
    assert!(completion_source.contains("checked_add(result_bytes)"));
    assert!(completion_source.contains("Err(TryRecvError::Disconnected)"));
    assert!(completion_source.contains("fail_worker_completion_channel"));
    assert!(completion_source.contains("WorkerChannelLost"));
    assert!(completion_source.contains("checked_sub(released_bytes)"));
    assert!(completion_source.contains("task.prepare_in_flight"));
    assert!(apply_source.contains("handler.apply_owned("));
    assert!(!apply_source.contains("json_value_byte_len"));
    assert!(!navigation_source.contains("bake_surface("));
    assert!(navigation_source.contains("generated bake state changed before owner apply"));
    assert!(navigation_source.contains("capture_bake_operation"));
}

#[test]
fn operation_task_uses_registered_canonical_id_instead_of_request_owned_text() {
    let source = include_str!("../service/admission.rs");
    let submit_start = source
        .find("    pub fn submit_with_deadline(")
        .expect("operation admission owner");
    let submit_end = source[submit_start..]
        .find("    fn reserve_raw_admission(")
        .map(|offset| submit_start + offset)
        .expect("operation admission boundary");
    let submit_source = &source[submit_start..submit_end];
    let task_start = submit_source
        .find("RuntimeOperationTask {")
        .expect("operation task construction");
    let task_source = &submit_source[task_start..];

    assert!(submit_source.contains(".get_key_value(&request.operation_id)"));
    assert!(task_source.contains("operation_id,"));
    assert!(!task_source.contains("operation_id: request.operation_id"));
}

#[test]
fn dynamic_submit_admits_raw_bytes_before_bounded_json_decode() {
    let source = include_str!("../../dynamic_api/session/operation.rs");
    let start = source
        .find("pub(crate) unsafe fn submit_operation(")
        .expect("dynamic operation submit owner");
    let end = source[start..]
        .find("pub(crate) unsafe fn poll_operation(")
        .map(|offset| start + offset)
        .expect("dynamic operation poll boundary");
    let submit_source = &source[start..end];
    let admission = submit_source
        .find("submit_with_raw_admission(")
        .expect("dynamic submit raw admission");
    let decode = submit_source
        .find("bounded_json::decode")
        .expect("dynamic submit bounded decoder");

    assert!(admission < decode);
    assert!(submit_source.contains("request_json.len()"));
    assert!(submit_source.contains("request_json.len() > maximum"));
    assert!(submit_source.contains("Ok(Err(error)) => operation_error_status(error)"));
    assert!(!submit_source.contains("runtime.operations.submit(request)"));
}

#[test]
fn worker_snapshot_bytes_stay_reserved_until_completion_or_channel_loss() {
    let service_source = include_str!("../service.rs");
    let completion_source = include_str!("../service/completion.rs");
    let maintenance_source = include_str!("../maintenance.rs");

    assert!(service_source.contains("in_flight_owner_bytes = task_bytes"));
    assert!(service_source.contains("if worker_still_owns_snapshot"));
    assert!(service_source.contains("task.in_flight_owner_bytes = 0"));
    assert!(completion_source.contains("replace(&mut task.in_flight_owner_bytes, 0)"));
    assert!(completion_source.contains("checked_sub(released_bytes)"));
    assert!(maintenance_source.contains("if worker_still_owns_snapshot"));
    assert!(maintenance_source.contains("released_bytes = if worker_still_owns_snapshot"));
}

#[test]
fn generated_mutation_epoch_is_part_of_every_owner_fence() {
    let manager_source =
        include_str!("../../../../zircon_plugins/navigation/runtime/src/manager.rs");
    let state_source =
        include_str!("../../../../zircon_plugins/navigation/runtime/src/manager/state.rs");
    let navigation_source = include_str!("../../navigation/operation/handler.rs");

    assert!(manager_source.contains("generated_mutation_epoch"));
    assert!(manager_source.contains("ensure_generated_mutation_epoch_available"));
    assert!(state_source.contains("generated_mutation_epoch: u64"));
    assert!(state_source.contains("clear_generated_snapshots"));
    assert!(state_source.contains("checked_add(1)"));
    assert!(navigation_source.contains("generated_mutation_epoch"));
    assert!(navigation_source.contains("generated bake mutation epoch changed before owner apply"));
}

#[test]
fn changed_navigation_operation_tests_use_deadlines_and_real_owner_state() {
    let source =
        include_str!("../../../../zircon_plugins/navigation/runtime/src/tests/operation.rs");
    assert!(source.contains("let deadline = Instant::now() + Duration::from_secs(5)"));
    assert!(source
        .contains("runtime_operation_owner_source_rejects_edit_replacement_and_unload_reload_aba"));
    assert!(source.contains("manager.generated_bake_snapshot(Some(surface))"));
    assert!(!source.contains("for _ in 0..256"));
    assert!(!source.contains("for _ in 0..4096"));
    assert!(source.contains("clear then restore must publish a fresh loaded identity"));
    let generation_tests = include_str!(
        "../../../../zircon_plugins/navigation/runtime/src/manager/bake/tests/geometry_provider_generation_tests.rs"
    );
    assert!(generation_tests
        .contains("a fresh project generation should publish after A to empty to A"));
    assert!(generation_tests.contains("project_asset_generation_after_empty"));
    assert!(generation_tests.contains("let first_loaded_handle = loaded[0].0"));
    assert!(generation_tests.contains("assert_ne!(reloaded[0].0, first_loaded_handle)"));
    assert!(generation_tests.contains("A-to-empty transition must release the loaded identity"));
    assert!(!generation_tests.contains("manager.assets"));
    let test_support =
        include_str!("../../../../zircon_plugins/navigation/runtime/src/tests/test_support.rs");
    assert!(test_support.contains("ProjectAssetManagerAccess::new"));
    assert!(test_support.contains("project_asset_manager_handle"));
    assert!(test_support.contains("level_from_world"));
    let navigation_lib = include_str!("../../../../zircon_plugins/navigation/runtime/src/lib.rs");
    assert!(navigation_lib.contains("ASSET_MODULE_NAME"));
    assert!(navigation_lib.contains("ProjectAssetManagerAccess::new"));
}

#[test]
fn navigation_bake_owner_bytes_are_measured_before_source_move() {
    let source = include_str!("../../navigation/operation/handler.rs");
    let capture = source
        .find("let (backend_snapshot, owner_bytes, before)")
        .expect("navigation bake owner capture");
    let end = source[capture..]
        .find("NavigationOperationKind::ClearSurface")
        .map(|offset| capture + offset)
        .expect("navigation owner capture boundary");
    let capture_source = &source[capture..end];
    let source_bytes = capture_source
        .find("let source_bytes = source_retained_bytes(&source)")
        .expect("source bytes are captured before owner move");
    let source_move = capture_source
        .find("source,\n                        runtime,")
        .expect("source moves into the owner state");
    assert!(source_bytes < source_move);
    assert!(capture_source.contains("owner_bytes.saturating_add(source_bytes)"));
}

#[test]
fn dynamic_session_ticks_operations_without_holding_the_world_mutex() {
    let source = include_str!("../../dynamic_api/session/state.rs");
    assert!(source.contains("self.operations.tick(&core, &self.level)"));
    assert!(!source.contains("self.level.with_world_mut(|world| self.operations.tick"));
}

#[test]
fn navigation_restore_rejects_null_surface_before_owner_capture() {
    let source = include_str!("../../navigation/operation/handler.rs");
    assert!(source.contains("navigation snapshot restore requires a non-null surface_entity"));
    assert!(source.contains("reject_noncanonical_restore_target"));
    assert!(source.contains("requested_surface.is_none()"));
    assert!(source.contains("before.surface_entity != requested_surface"));
    let operation_tests =
        include_str!("../../../../zircon_plugins/navigation/runtime/src/tests/operation.rs");
    assert!(operation_tests.contains("surface_entity"));
    assert!(operation_tests.contains("null_restore_deadline"));
    assert!(operation_tests.contains("null_restore_result.failure().is_some()"));
    assert!(operation_tests.contains("assert_eq!(manager.loaded_assets(), loaded_before)"));
}

#[test]
fn bake_generation_admission_rejects_max_without_saturating_reuse() {
    let state = include_str!("../../../../zircon_plugins/navigation/runtime/src/manager/state.rs");
    let manager = include_str!("../../../../zircon_plugins/navigation/runtime/src/manager.rs");
    let tiled =
        include_str!("../../../../zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs");
    let dirty =
        include_str!("../../../../zircon_plugins/navigation/runtime/src/manager/bake/dirty.rs");
    let bake = include_str!("../../../../zircon_plugins/navigation/runtime/src/manager/bake.rs");
    let navigation_handler = include_str!("../../navigation/operation/handler.rs");
    assert!(state.contains("try_advance_bake_context"));
    assert!(state.contains("checked_add(1)"));
    assert!(!state.contains("context.next_generation = context.next_generation.saturating_add(1)"));
    assert!(manager.contains("try_advance_bake_context"));
    assert!(state.contains("navigation bake generation exhausted"));
    assert!(tiled.contains("try_advance_bake_context"));
    assert!(dirty.contains("try_advance_bake_context"));
    assert!(bake.contains("begin_bake_generation"));
    assert!(bake.contains("publish_bake_with_source_fence"));
    assert!(navigation_handler.contains("publish_operation_bake_with_source_fence"));
    assert!(state.contains(
        "bake_generation_max_minus_one_is_admitted_once_then_exhausted_without_mutation"
    ));
    assert!(state.contains(
        "bake_generation_boundary_is_failure_atomic_for_direct_tiled_dirty_and_operation_keys"
    ));
}

#[test]
fn lost_worker_channel_releases_nonzero_owner_bytes_exactly_once() {
    let completion = include_str!("../service/completion.rs");
    let service = include_str!("../service.rs");
    assert!(completion.contains("in_flight_owner_bytes: retained_bytes"));
    assert!(completion.contains("assert_eq!(service.lock_state().retained_bytes, 0)"));
    let second_drain = completion
        .find("service.drain_prepare_completions();")
        .expect("channel-loss completion drain");
    let second_assert = completion[second_drain..]
        .find("assert_eq!(service.lock_state().retained_bytes, 0)")
        .expect("second drain retains zero bytes");
    assert!(second_assert > 0);
    let retention = include_str!("inflight_retention.rs");
    assert!(retention.contains(
        "real_worker_channel_loss_releases_nonzero_owner_bytes_once_and_allows_admission"
    ));
    assert!(retention.contains("drop_prepare_completion_receivers_for_test"));
    assert!(service.contains("lost_completion_batches"));
    assert!(service.contains("if completion_sender.send(completion).is_err()"));
}

#[test]
fn operation_phase_barriers_use_monotonic_deadlines() {
    let phase_indexes = include_str!("phase_indexes.rs");
    assert!(phase_indexes.contains("fn tick_until<F>("));
    assert!(phase_indexes.contains("Instant::now() + Duration::from_secs(5)"));
    assert!(!phase_indexes.contains("0..16_384"));
    assert!(!phase_indexes.contains("0..1_024"));
}
