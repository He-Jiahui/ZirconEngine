use super::{
    capture_epoch, capture_epoch_for_completion, record_counter_batch, reset_capture, snapshot,
    start_capture, stop_capture, test_capture_lock, with_recorder, ProfileCaptureConfig,
};

#[test]
fn profile_recorder_accessors_recover_poisoned_global_lock() {
    let _guard = test_capture_lock();
    let poison_result = std::panic::catch_unwind(|| {
        let _recorder = super::lock_recorder();
        panic!("poison profile recorder lock");
    });
    assert!(poison_result.is_err());

    let snapshot_after_poison = snapshot();
    assert!(!snapshot_after_poison.active);

    let status = with_recorder(|recorder| recorder.reset());
    assert_eq!(status.message, "profile capture reset");
    assert!(!snapshot().active);
}

#[test]
fn asynchronous_producers_can_reject_reports_from_an_older_capture_epoch() {
    let _guard = test_capture_lock();
    reset_capture();
    assert_eq!(capture_epoch(), None);

    let first_status = start_capture(ProfileCaptureConfig::default());
    if !first_status.active {
        assert_eq!(capture_epoch(), None);
        return;
    }
    let first_epoch = capture_epoch().expect("active capture must expose an epoch");
    stop_capture();
    assert_eq!(capture_epoch(), None);
    assert_eq!(capture_epoch_for_completion(), Some(first_epoch));

    reset_capture();
    assert_eq!(capture_epoch(), None);
    assert_ne!(capture_epoch_for_completion(), Some(first_epoch));

    let second_status = start_capture(ProfileCaptureConfig::default());
    assert!(second_status.active);
    let second_epoch = capture_epoch().expect("restarted capture must expose an epoch");
    reset_capture();

    assert!(second_epoch > first_epoch);
}

#[cfg(feature = "profiling")]
#[test]
fn scope_completion_from_a_retired_capture_epoch_does_not_enter_the_new_capture() {
    let _guard = test_capture_lock();
    reset_capture();
    start_capture(ProfileCaptureConfig {
        session_id: "retired-scope-source".to_string(),
        max_spans: 4,
        ..ProfileCaptureConfig::default()
    });
    let retired_scope = super::ProfileScope::enter("runtime", "test", "retired");

    reset_capture();
    start_capture(ProfileCaptureConfig {
        session_id: "retired-scope-target".to_string(),
        max_spans: 4,
        ..ProfileCaptureConfig::default()
    });
    let current_scope = super::ProfileScope::enter("runtime", "test", "current");
    drop(retired_scope);
    drop(current_scope);

    let captured = snapshot();
    reset_capture();
    assert_eq!(
        captured
            .spans
            .iter()
            .map(|span| span.name.as_str())
            .collect::<Vec<_>>(),
        vec!["current"]
    );
    assert_eq!(captured.spans[0].parent_id, None);
    assert_eq!(captured.spans[0].depth, 0);
}

#[cfg(feature = "profiling")]
#[test]
fn frame_context_from_a_retired_capture_epoch_does_not_attach_to_the_new_capture() {
    let _guard = test_capture_lock();
    reset_capture();
    start_capture(ProfileCaptureConfig {
        session_id: "retired-frame-source".to_string(),
        max_frames: 2,
        ..ProfileCaptureConfig::default()
    });
    let retired_frame = super::ProfileFrameScope::enter("runtime", "retired");
    let retired_context = super::ProfileFrameContext::capture();

    reset_capture();
    start_capture(ProfileCaptureConfig {
        session_id: "retired-frame-target".to_string(),
        max_frames: 2,
        max_counters: 2,
        ..ProfileCaptureConfig::default()
    });
    let _retired_context = retired_context.attach();
    super::record_counter("runtime", "current.counter", 1.0);
    drop(retired_frame);

    let captured = snapshot();
    reset_capture();
    assert!(captured.frames.is_empty());
    assert_eq!(captured.counters.len(), 1);
    assert_eq!(captured.counters[0].frame_index, None);
}

#[cfg(feature = "profiling")]
#[test]
fn profile_macros_capture_nested_spans_inside_frame() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "nested-span-test".to_string();
    config.max_frames = 4;
    config.max_spans = 8;
    start_capture(config);

    {
        crate::profile_frame!("runtime", "test_frame");
        {
            crate::profile_scope!("runtime", "test", "outer");
            {
                crate::profile_scope!("runtime", "test", "inner");
            }
        }
    }

    let snapshot = snapshot();
    reset_capture();
    assert_eq!(snapshot.frames.len(), 1);
    assert_eq!(snapshot.spans.len(), 2);
    let outer = snapshot
        .spans
        .iter()
        .find(|span| span.name == "outer")
        .expect("outer span");
    let inner = snapshot
        .spans
        .iter()
        .find(|span| span.name == "inner")
        .expect("inner span");
    assert_eq!(outer.parent_id, None);
    assert_eq!(inner.parent_id, Some(outer.id));
    assert_eq!(inner.depth, 1);
    assert_eq!(inner.frame_index, Some(0));
}

#[cfg(feature = "profiling")]
#[test]
fn counter_batch_records_each_counter_under_the_active_frame() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "counter-batch-test".to_string();
    config.max_frames = 2;
    config.max_counters = 4;
    start_capture(config);

    {
        crate::profile_frame!("runtime", "test_frame");
        record_counter_batch(
            "runtime",
            &[("test.batch_first", 1.0), ("test.batch_second", 2.0)],
        );
    }

    let snapshot = snapshot();
    reset_capture();
    assert_eq!(snapshot.counters.len(), 2);
    assert_eq!(snapshot.counters[0].name, "test.batch_first");
    assert_eq!(snapshot.counters[0].value, 1.0);
    assert_eq!(snapshot.counters[0].frame_index, Some(0));
    assert_eq!(snapshot.counters[1].name, "test.batch_second");
    assert_eq!(snapshot.counters[1].value, 2.0);
    assert_eq!(snapshot.counters[1].frame_index, Some(0));
    assert_eq!(
        snapshot.counters[0].timestamp_us,
        snapshot.counters[1].timestamp_us
    );
}

#[cfg(feature = "profiling")]
#[test]
fn captured_frame_context_attaches_scoped_worker_samples_to_the_frame() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "scoped-worker-frame-context".to_string();
    config.max_frames = 2;
    config.max_spans = 4;
    config.max_counters = 4;
    start_capture(config);

    {
        crate::profile_frame!("runtime", "test_frame");
        let frame_context = super::ProfileFrameContext::capture();
        std::thread::scope(|scope| {
            scope
                .spawn(move || {
                    let _frame_context = frame_context.attach();
                    crate::profile_scope!("runtime", "test", "scoped_worker");
                    crate::profile_counter!("runtime", "test.scoped_worker", 1);
                })
                .join()
                .expect("scoped profiling worker should finish");
        });
    }

    let snapshot = snapshot();
    reset_capture();
    assert_eq!(snapshot.frames.len(), 1);
    assert_eq!(
        snapshot
            .spans
            .iter()
            .find(|span| span.name == "scoped_worker")
            .and_then(|span| span.frame_index),
        Some(0)
    );
    assert_eq!(
        snapshot
            .counters
            .iter()
            .find(|counter| counter.name == "test.scoped_worker")
            .and_then(|counter| counter.frame_index),
        Some(0)
    );
}

#[cfg(feature = "profiling")]
#[test]
fn profile_scope_enter_named_captures_runtime_generated_names() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "dynamic-span-test".to_string();
    config.max_spans = 4;
    start_capture(config);

    {
        let pass_name = format!("{}-{}", "graph-pass", 7);
        let _scope = super::ProfileScope::enter_named("runtime", "render_graph.pass", pass_name);
    }

    let snapshot = snapshot();
    reset_capture();
    let span = snapshot
        .spans
        .iter()
        .find(|span| span.category == "render_graph.pass")
        .expect("dynamic render graph pass span");
    assert_eq!(span.name, "graph-pass-7");
    assert_eq!(span.path, "runtime/render_graph.pass:graph-pass-7");
}

#[cfg(feature = "profiling")]
#[test]
fn profile_dynamic_scope_macro_captures_runtime_generated_names() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "dynamic-macro-span-test".to_string();
    config.max_spans = 4;
    start_capture(config);

    {
        crate::profile_dynamic_scope!(
            "runtime",
            "render_graph.stage",
            format!("{:?}", crate::graphics::RenderPassStage::PostProcess),
        );
    }

    let snapshot = snapshot();
    reset_capture();
    let span = snapshot
        .spans
        .iter()
        .find(|span| span.category == "render_graph.stage")
        .expect("dynamic macro render graph stage span");
    assert_eq!(span.name, "PostProcess");
    assert_eq!(span.path, "runtime/render_graph.stage:PostProcess");
}

#[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
#[test]
fn inactive_profile_macros_skip_dynamic_payload_evaluation() {
    let _guard = test_capture_lock();
    reset_capture();

    fn inactive_dynamic_scope_name() -> String {
        panic!("inactive dynamic scope payload was evaluated")
    }

    crate::profile_dynamic_scope!("runtime", "test", inactive_dynamic_scope_name(),);
    crate::profile_counter!(
        "runtime",
        "inactive.counter",
        panic!("inactive counter payload was evaluated"),
    );

    assert!(!super::capture_active());
}

#[cfg(not(feature = "profiling"))]
#[test]
fn disabled_profile_macros_do_not_evaluate_arguments() {
    crate::profile_scope!(panic!("stream"), panic!("category"), panic!("name"));
    crate::profile_dynamic_scope!(panic!("stream"), panic!("category"), panic!("name"));
    crate::profile_frame!(panic!("stream"), panic!("name"));
    crate::profile_counter!(panic!("stream"), panic!("name"), panic!("value"));

    assert!(!super::feature_enabled());
}
