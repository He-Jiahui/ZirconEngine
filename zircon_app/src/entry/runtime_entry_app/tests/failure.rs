use std::time::{Duration, Instant};

use super::{RuntimeEntryAppFailure, RuntimeEntryAppFailureState};

#[test]
fn runtime_entry_failure_uses_actionable_startup_diagnostic_fields() {
    let failure = RuntimeEntryAppFailure::new(
        "runtime_surface_present",
        "viewport=1 size=1280x720",
        "frame capture failed: device lost",
        "verify the graphics adapter and restart zircon_runtime",
    );

    assert_eq!(
        failure.to_string(),
        "runtime startup diagnostic: component=runtime_surface_present requested=viewport=1 size=1280x720 cause=frame capture failed: device lost recovery=verify the graphics adapter and restart zircon_runtime"
    );
}

#[test]
fn runtime_entry_failure_state_retains_all_fatal_callback_failures() {
    let state = RuntimeEntryAppFailureState::default();
    assert!(
        !state.is_recorded(),
        "a fresh runtime entry failure state must allow host initialization"
    );
    state.record(RuntimeEntryAppFailure::new(
        "runtime_window",
        "primary_window",
        "window creation failed",
        "verify the desktop session and retry",
    ));
    state.record(RuntimeEntryAppFailure::new(
        "runtime_frame_loop",
        "runtime_session",
        "frame tick failed",
        "restart zircon_runtime",
    ));
    assert!(
        state.is_recorded(),
        "a terminal callback failure must remain visible until EntryRunner collects it"
    );

    let report = state.failure_ledger().snapshot();
    assert_eq!(report.records().len(), 2);
    assert_eq!(report.primary().unwrap().owner(), "runtime_window");
    assert_eq!(report.secondary()[0].owner(), "runtime_frame_loop");
    assert!(state.is_recorded());
}

#[test]
fn recorded_flag_publishes_the_failure_record_before_readers_stop() {
    let state = RuntimeEntryAppFailureState::default();
    let producer = state.clone();
    let producer = std::thread::spawn(move || {
        producer.record(RuntimeEntryAppFailure::new(
            "runtime_frame_loop",
            "runtime_session",
            "frame tick failed",
            "restart zircon_runtime",
        ));
    });
    let deadline = Instant::now() + Duration::from_secs(5);

    while !state.is_recorded() {
        assert!(
            Instant::now() < deadline,
            "the producer must publish its terminal callback failure"
        );
        std::thread::yield_now();
    }

    let report = state.failure_ledger().snapshot();
    producer.join().expect("failure producer must finish");
    assert_eq!(report.records().len(), 1);
    assert_eq!(report.primary().unwrap().owner(), "runtime_frame_loop");
}
