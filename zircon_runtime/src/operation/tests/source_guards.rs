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
fn operation_maintenance_timer_is_service_scoped_and_rearms_deadlines_and_ttl() {
    let service_source = include_str!("../service.rs");
    let source = include_str!("../maintenance.rs");
    let start = source
        .find("fn refresh_operation_maintenance_alarm(")
        .expect("service maintenance timer owner");
    let end = source[start..]
        .find("fn next_maintenance_deadline(")
        .map(|offset| start + offset)
        .expect("maintenance deadline selection boundary");
    let maintenance_source = &source[start..end];

    assert!(service_source.contains("maintenance_subscription: Option<TaskTimerSubscription>"));
    assert!(service_source.contains("maintenance_deadline: Option<Instant>"));
    assert!(maintenance_source.contains("TaskTimer::process_default()"));
    assert!(maintenance_source.contains("Arc::downgrade(state)"));
    assert!(maintenance_source.contains("expire_due_deadlines_in_state"));
    assert!(maintenance_source.contains("expire_terminal_results_in_state"));
    assert!(maintenance_source.contains("refresh_operation_maintenance_alarm("));
}

#[test]
fn raw_admission_release_preserves_exact_count_and_byte_invariants() {
    let source = include_str!("../service.rs");
    let start = source
        .find("fn consume_raw_admission(")
        .expect("raw admission release owner");
    let end = source[start..]
        .find("struct JsonByteCounter")
        .map(|offset| start + offset)
        .expect("raw admission release boundary");
    let release_source = &source[start..end];

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
    assert!(completion_source.contains("command_bytes.checked_add(result_bytes)"));
    assert!(completion_source.contains("Err(TryRecvError::Disconnected)"));
    assert!(completion_source.contains("fail_worker_completion_channel"));
    assert!(completion_source.contains("WorkerChannelLost"));
    assert!(completion_source.contains("checked_sub(lost_prepare_count)"));
    assert!(completion_source.contains("task.prepare_in_flight"));
    assert!(
        apply_source.contains("handler.apply(RuntimeOperationContext::new(core, world), command)")
    );
    assert!(!apply_source.contains("json_value_byte_len"));
    assert!(!navigation_source.contains("bake_surface("));
    assert!(navigation_source.contains("generated bake state changed after operation snapshot"));
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
