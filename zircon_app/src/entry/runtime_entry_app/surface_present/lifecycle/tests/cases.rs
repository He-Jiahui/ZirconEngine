//! Behavior tests call the actual App cleanup methods and actual RuntimeSession/ABI state machine.
//! Native Window/HWND lifetime and OS Destroyed callback timing still require a physical-window gate.

use zircon_runtime_interface::{
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeImeCompositionNegotiation,
    ZrRuntimeNativeSurfaceTargetV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_API_VERSION_V8,
    ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY,
};

use super::RuntimeEntryApp;
use crate::entry::runtime_entry_app::{
    ime_input::{RuntimeImeCompositionContextAllocator, RuntimeImeCompositionProducer},
    RuntimeEntryAppConfig, RuntimeEntryAppFailureState,
};
use crate::entry::runtime_library::{
    surface_teardown_fixture::{with_session, Call, Policy, ProviderControl},
    RuntimeSession,
};

fn bound_app(session: RuntimeSession) -> RuntimeEntryApp {
    let mut app = RuntimeEntryApp::new(
        session,
        RuntimeEntryAppConfig::default(),
        RuntimeEntryAppFailureState::default(),
    );
    let request = ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        app.viewport,
        app.viewport_size,
        ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
    );
    assert_eq!(app.session.bind_viewport_surface(request).unwrap(), true);
    app.surface_present_attempted = true;
    app.enable_surface_present();
    app
}

fn teardown_calls(control: &ProviderControl) -> Vec<Call> {
    control
        .snapshot()
        .calls
        .into_iter()
        .filter(|call| matches!(call, Call::Unbind | Call::Destroy | Call::Event))
        .collect()
}

#[test]
fn failed_surface_release_preserves_app_flags_and_binding_for_retry() {
    with_session(
        Policy {
            unbind_failures: 1,
            ..Policy::default()
        },
        |session, control| {
            let mut app = bound_app(session);
            assert!(app.release_surface_present().is_err());
            assert!(app.surface_present_enabled);
            assert!(app.surface_present_attempted);
            assert!(control.snapshot().bound);

            app.release_surface_present().unwrap();
            assert!(!app.surface_present_enabled);
            assert!(!app.surface_present_attempted);
            assert!(!control.snapshot().bound);
            app.release_surface_present().unwrap();
            assert_eq!(teardown_calls(control), [Call::Unbind, Call::Unbind]);
        },
    );
}

#[test]
fn stale_app_flags_do_not_hide_a_registered_runtime_surface() {
    with_session(Policy::default(), |session, control| {
        let mut app = bound_app(session);
        app.surface_present_enabled = false;
        app.surface_present_attempted = false;
        app.release_surface_present().unwrap();
        assert!(!control.snapshot().bound);
        assert_eq!(teardown_calls(control), [Call::Unbind]);
    });
}

#[test]
fn failed_primary_teardown_destroys_session_and_preserves_the_product_failure() {
    with_session(
        Policy {
            unbind_failures: 1,
            ..Policy::default()
        },
        |session, control| {
            let mut app = bound_app(session);
            assert!(!app.teardown_primary_window());
            assert!(app.failure_state.is_recorded());
            assert_eq!(
                app.failure_state
                    .failure_ledger()
                    .snapshot()
                    .records()
                    .len(),
                1
            );
            assert!(!control.snapshot().active);
            assert!(!app.surface_present_enabled);
            assert!(!app.surface_present_attempted);
            assert_eq!(
                teardown_calls(control),
                [Call::Unbind, Call::Unbind, Call::Destroy]
            );

            assert!(app.teardown_primary_window());
            assert!(app.failure_state.is_recorded());
            drop(app);
            assert_eq!(
                teardown_calls(control),
                [Call::Unbind, Call::Unbind, Call::Destroy]
            );
        },
    );
}

#[test]
fn persistent_unbind_failure_uses_session_destroy_without_restoring_the_product() {
    with_session(
        Policy {
            unbind_failures: usize::MAX,
            ..Policy::default()
        },
        |session, control| {
            let mut app = bound_app(session);
            assert!(!app.teardown_primary_window());
            assert!(app.failure_state.is_recorded());
            assert!(!control.snapshot().active);
            assert!(!control.snapshot().bound);
            assert_eq!(
                teardown_calls(control),
                [Call::Unbind, Call::Unbind, Call::Destroy]
            );
            drop(app);
            assert_eq!(
                teardown_calls(control),
                [Call::Unbind, Call::Unbind, Call::Destroy]
            );
        },
    );
}

#[test]
fn missing_unbind_export_cannot_report_a_bound_surface_as_released() {
    with_session(
        Policy {
            unbind_export: false,
            ..Policy::default()
        },
        |session, control| {
            let mut app = bound_app(session);
            assert!(app.release_surface_present().is_err());
            assert!(app.surface_present_enabled);
            assert!(control.snapshot().bound);
            assert!(!app.teardown_primary_window());
            assert!(app.failure_state.is_recorded());
            assert!(!control.snapshot().active);
            assert_eq!(teardown_calls(control), [Call::Destroy]);
        },
    );
}

#[test]
fn failed_cpu_presenter_transition_keeps_the_live_binding_and_terminal_failure() {
    with_session(
        Policy {
            unbind_failures: 1,
            ..Policy::default()
        },
        |session, control| {
            let mut app = bound_app(session);
            assert!(!app.enable_reference_cpu_presenter());
            assert!(app.failure_state.is_recorded());
            assert!(app.surface_present_enabled);
            assert!(app.surface_present_attempted);
            assert!(control.snapshot().active);
            assert!(control.snapshot().bound);
            assert!(app.presenter.is_none());
            assert!(!app.enable_reference_cpu_presenter());
            assert_eq!(teardown_calls(control), [Call::Unbind]);
        },
    );
}

#[test]
fn failed_destroy_retains_the_session_for_a_checked_retry() {
    with_session(
        Policy {
            destroy_failures: 1,
            ..Policy::default()
        },
        |mut session, control| {
            assert!(session.try_destroy().is_err());
            assert!(control.snapshot().active);
            control.allow_cleanup();
            session.try_destroy().unwrap();
            assert!(!control.snapshot().active);
            session.try_destroy().unwrap();
            assert_eq!(teardown_calls(control), [Call::Destroy, Call::Destroy]);
        },
    );
}

#[test]
fn destroyed_session_rejects_events_without_reentering_the_provider() {
    with_session(Policy::default(), |mut session, control| {
        session.try_destroy().unwrap();
        assert!(session
            .handle_event(ZrRuntimeEventV1::window_destroyed(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                zircon_runtime_interface::ZrRuntimeViewportHandle::new(1),
            ))
            .is_err());
        assert_eq!(teardown_calls(control), [Call::Destroy]);
    });
}

#[test]
fn native_ime_retirement_precedes_destroy_and_is_not_repeated_after_destroy() {
    with_session(
        Policy {
            unbind_failures: 1,
            ..Policy::default()
        },
        |session, control| {
            let mut app = bound_app(session);
            app.ime_composition_producer = Some(RuntimeImeCompositionProducer::new(
                ZrRuntimeImeCompositionNegotiation::from_peer(
                    ZIRCON_RUNTIME_API_VERSION_V8,
                    [ZR_RUNTIME_IME_COMPOSITION_V2_CAPABILITY],
                ),
                RuntimeImeCompositionContextAllocator::new(app.ime_window_generation).unwrap(),
            ));
            app.native_ime_composition_started = true;
            assert!(!app.teardown_primary_window());
            assert!(!app.native_ime_composition_started);
            assert_eq!(
                teardown_calls(control),
                [Call::Event, Call::Unbind, Call::Unbind, Call::Destroy],
            );
            assert!(app.teardown_primary_window());
            drop(app);
            assert_eq!(
                teardown_calls(control),
                [Call::Event, Call::Unbind, Call::Unbind, Call::Destroy],
            );
        },
    );
}

const FATAL_DESTROY_CHILD_ENV: &str = "ZR_APP_SURFACE_DESTROY_FATAL_CHILD";
const FATAL_DESTROY_ENTERED: &str = "ZR_APP_SURFACE_DESTROY_ENTERED";
const FATAL_DESTROY_RETURNED: &str = "ZR_APP_SURFACE_DESTROY_RETURNED";
const FATAL_DESTROY_PANICKED: &str = "ZR_APP_SURFACE_DESTROY_PANICKED";

#[test]
fn primary_teardown_destroy_failure_uses_the_existing_process_fatal_boundary() {
    let test_module = module_path!()
        .split_once("::")
        .expect("test module includes the crate name")
        .1;
    let child_name = format!("{test_module}::primary_teardown_destroy_failure_child");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", child_name.as_str(), "--nocapture"])
        .env(FATAL_DESTROY_CHILD_ENV, "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("run the isolated fatal-destroy test process");
    let started = std::time::Instant::now();
    const WAIT_LIMIT: std::time::Duration = std::time::Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < WAIT_LIMIT => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            status => {
                // This test owns the exact child handle; a regression must not leave it running.
                let kill_result = child.kill();
                let termination_started = std::time::Instant::now();
                const TERMINATION_LIMIT: std::time::Duration = std::time::Duration::from_secs(1);
                while termination_started.elapsed() < TERMINATION_LIMIT {
                    match child.try_wait() {
                        Ok(Some(exit)) => panic!(
                            "fatal-destroy child missed {WAIT_LIMIT:?}: {status:?}; kill={kill_result:?}; exited={exit}"
                        ),
                        Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
                        Err(error) => panic!(
                            "fatal-destroy child missed {WAIT_LIMIT:?}: {status:?}; kill={kill_result:?}; status error={error}"
                        ),
                    }
                }
                panic!(
                    "fatal-destroy child missed {WAIT_LIMIT:?}: {status:?}; kill={kill_result:?}; child {} exit not observed within {TERMINATION_LIMIT:?}",
                    child.id()
                );
            }
        }
    }
    let output = child
        .wait_with_output()
        .expect("collect the exited fatal-destroy test process output");
    assert!(
        !output.status.success(),
        "failed terminal destroy must abort"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(FATAL_DESTROY_ENTERED), "{stderr}");
    assert!(!stderr.contains(FATAL_DESTROY_RETURNED), "{stderr}");
    assert!(!stderr.contains(FATAL_DESTROY_PANICKED), "{stderr}");
    assert!(
        stderr.contains("fatal runtime session teardown failure:"),
        "{stderr}"
    );
    assert!(
        stderr.contains("aborting before dynamic library unload"),
        "{stderr}"
    );
}

#[test]
fn primary_teardown_destroy_failure_child() {
    if std::env::var(FATAL_DESTROY_CHILD_ENV).as_deref() != Ok("1") {
        return;
    }
    // A panic must fail this child directly, before unwinding can invoke an unrelated Drop abort.
    std::panic::set_hook(Box::new(|panic| {
        eprintln!("{FATAL_DESTROY_PANICKED}: {panic}");
        std::process::exit(87);
    }));
    with_session(
        Policy {
            unbind_failures: usize::MAX,
            destroy_failures: usize::MAX,
            ..Policy::default()
        },
        |session, _control| {
            let mut app = bound_app(session);
            eprintln!("{FATAL_DESTROY_ENTERED}");
            let _ = app.teardown_primary_window();
            // Bypass Drop: its abort must not make a returning teardown falsely pass this test.
            eprintln!("{FATAL_DESTROY_RETURNED}");
            std::process::exit(86);
        },
    );
}
