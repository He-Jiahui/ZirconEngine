use super::*;

#[test]
fn radiance_cache_bootstrap_requires_successful_gpu_observation() {
    let mut bootstrap = RadianceCacheBootstrapState::default();

    let initial = bootstrap.begin_submission(false);
    assert!(initial.uses_bootstrap_snapshot);
    assert_eq!(initial.revision, 1);
    assert!(bootstrap.begin_submission(false).uses_bootstrap_snapshot);

    bootstrap.confirm_submission(initial.revision);
    let stable = bootstrap.begin_submission(false);
    assert!(!stable.uses_bootstrap_snapshot);
    assert_eq!(stable.revision, initial.revision);
}

#[test]
fn radiance_cache_update_retries_as_bootstrap_until_latest_revision_is_observed() {
    let mut bootstrap = RadianceCacheBootstrapState::default();
    let initial = bootstrap.begin_submission(false);
    bootstrap.confirm_submission(initial.revision);

    let incremental = bootstrap.begin_submission(true);
    assert!(!incremental.uses_bootstrap_snapshot);
    assert_eq!(incremental.revision, 2);
    let retry = bootstrap.begin_submission(false);
    assert!(retry.uses_bootstrap_snapshot);
    assert_eq!(retry.revision, incremental.revision);

    bootstrap.confirm_submission(initial.revision);
    assert!(bootstrap.begin_submission(false).uses_bootstrap_snapshot);
    bootstrap.confirm_submission(incremental.revision);
    assert!(!bootstrap.begin_submission(false).uses_bootstrap_snapshot);
}

#[test]
fn radiance_cache_readback_observation_is_bounded_to_the_shared_frame_ring() {
    let capacity = RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES;

    assert!(can_enqueue_readback_observation(capacity.saturating_sub(1)));
    assert!(!can_enqueue_readback_observation(capacity));
    assert!(!can_enqueue_readback_observation(
        capacity.saturating_add(1)
    ));
}

#[test]
fn global_sdf_deferred_requests_rotate_across_bounded_batches() {
    let mut requests = [0_u32, 1, 2, 3, 4];
    let mut cursor = 0;

    rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
    assert_eq!(requests, [0, 1, 2, 3, 4]);
    assert_eq!(cursor, 2);

    requests.sort_unstable();
    rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
    assert_eq!(requests, [2, 3, 4, 0, 1]);
    assert_eq!(cursor, 4);

    requests.sort_unstable();
    rotate_global_sdf_build_requests(&mut requests, &mut cursor, 2);
    assert_eq!(requests, [4, 0, 1, 2, 3]);
    assert_eq!(cursor, 1);
}

#[test]
fn global_sdf_readback_backpressure_reports_deferred_work() {
    let stats = GlobalSdfGpuBuildStats::deferred_by_readback_backpressure(7);

    assert_eq!(stats.deferred_page_count, 7);
    assert_eq!(stats.dispatched_page_count, 0);
    assert_eq!(stats.transient_upload_byte_count, 0);
}

#[test]
fn new_hgi_gpu_work_requires_shared_readback_admission_before_dispatch() {
    let source = include_str!("../runtime_prepare_collector.rs");
    let admission_gate = ["if !context.", "gpu_work_admitted() {"].concat();
    let admission_gate = source
        .find(&admission_gate)
        .expect("runtime prepare must reject new GPU work without a shared readback slot");
    let global_sdf_dispatch = source
        .find("gpu_resources.dispatch_global_sdf_pages(")
        .expect("global SDF dispatch must remain explicit");
    let prepare_dispatch = source
        .find("gpu_resources.execute_prepare(")
        .expect("HGI prepare dispatch must remain explicit");

    assert!(admission_gate < global_sdf_dispatch);
    assert!(admission_gate < prepare_dispatch);
}

#[test]
fn hgi_prepare_uses_queue_free_upload_and_state_transaction_recorders() {
    let collector = include_str!("../runtime_prepare_collector.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let params = include_str!("../../gpu_resources/execute_prepare/execute/queue_params.rs");
    let radiance = include_str!(
        "../../gpu_resources/execute_prepare/execute/dispatch_radiance_cache/dispatch.rs"
    );

    assert!(collector.contains("context.gpu_recording_context()"));
    assert!(collector.contains("mut buffer_uploads"));
    assert!(collector.contains("mut frame_transactions"));
    assert!(!collector.contains("context.queue"));
    assert!(!collector.contains("context.device"));
    assert!(!collector.contains("context.encoder"));
    assert!(!collector.contains("expect(\"hybrid GI GPU resources"));
    assert!(!params.contains("queue.write_buffer"));
    assert!(!radiance.contains("queue.write_buffer"));
}

#[test]
fn hgi_readback_futures_publish_only_after_frame_acceptance() {
    let source = include_str!("../runtime_prepare_collector.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let global_enqueue = source
        .find("let future = pending.enqueue(context)?;")
        .expect("Global SDF readback enqueue");
    let global_publish = source[global_enqueue..]
        .find("register_global_sdf_readback_frame_commit(")
        .map(|offset| global_enqueue + offset)
        .expect("Global SDF readback frame commit");
    let radiance_enqueue = source[global_publish..]
        .find("let future = pending_readback.enqueue(context)?;")
        .map(|offset| global_publish + offset)
        .expect("radiance-cache readback enqueue");
    let radiance_publish = source[radiance_enqueue..]
        .find("register_radiance_cache_readback_frame_commit(")
        .map(|offset| radiance_enqueue + offset)
        .expect("radiance-cache readback frame commit");

    assert!(global_enqueue < global_publish);
    assert!(radiance_enqueue < radiance_publish);
    assert!(source.contains("hybrid-gi.global-sdf-readback"));
    assert!(source.contains("hybrid-gi.radiance-cache-readback"));
    assert!(source.contains(".saturating_add(staged_readback_count)"));
    assert_eq!(
        source
            .matches("context.register_frame_transaction(RuntimePrepareFrameTransaction::new(")
            .count(),
        2
    );
}

#[test]
fn hgi_gpu_dispatches_publish_named_shared_timer_scopes() {
    let source = include_str!("../runtime_prepare_collector.rs");
    let global_scope = ["context.begin_gpu_pass(", "GLOBAL_SDF_BUILD_PROFILE_NAME"].concat();
    let prepare_scope = ["context.begin_gpu_pass(", "HGI_PREPARE_PROFILE_NAME"].concat();
    let end_scope = ["context.end_gpu_", "pass("].concat();
    let global_dispatch = source
        .find("gpu_resources.dispatch_global_sdf_pages(")
        .expect("Global SDF dispatch must remain explicit");
    let prepare_dispatch = source
        .find("gpu_resources.execute_prepare(")
        .expect("radiance-cache prepare dispatch must remain explicit");
    let global_scope = source
        .find(&global_scope)
        .expect("Global SDF dispatch must open a shared timer scope");
    let prepare_scope = source
        .find(&prepare_scope)
        .expect("radiance-cache prepare dispatch must open a shared timer scope");
    let global_end = source[global_dispatch..]
        .find(&end_scope)
        .map(|offset| global_dispatch + offset)
        .expect("Global SDF dispatch must close its shared timer scope");
    let prepare_end = source[prepare_dispatch..]
        .find(&end_scope)
        .map(|offset| prepare_dispatch + offset)
        .expect("radiance-cache prepare dispatch must close its shared timer scope");

    assert!(source.contains("runtime_prepare.hybrid_gi.global_sdf_build"));
    assert!(source.contains("runtime_prepare.hybrid_gi.prepare"));
    assert!(global_scope < global_dispatch);
    assert!(global_dispatch < global_end);
    assert!(prepare_scope < prepare_dispatch);
    assert!(prepare_dispatch < prepare_end);
}

#[test]
fn empty_global_sdf_dispatch_closes_its_scope_without_publishing_a_profile() {
    let source = include_str!("../runtime_prepare_collector.rs");
    let global_dispatch = source
        .find("gpu_resources.dispatch_global_sdf_pages(")
        .expect("Global SDF dispatch must remain explicit");
    let dispatch_profile = source[global_dispatch..]
        .find("if dispatch.encoded_gpu_work() {")
        .map(|offset| global_dispatch + offset)
        .expect("Global SDF profile must be conditional on encoded GPU work");
    let discard_scope = source[dispatch_profile..]
        .find("context.discard_gpu_pass(gpu_pass);")
        .map(|offset| dispatch_profile + offset)
        .expect("empty Global SDF dispatch must close its timer scope");
    let stats = source[global_dispatch..]
        .find("global_sdf_build_stats = dispatch.stats();")
        .map(|offset| global_dispatch + offset)
        .expect("Global SDF dispatch statistics must remain available");

    assert!(global_dispatch < dispatch_profile);
    assert!(dispatch_profile < discard_scope);
    assert!(discard_scope < stats);
}
