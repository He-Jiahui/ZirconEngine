//! Ignored physical Window owner regression; no RHI/GPU/DLL worker is constructed.
//! Only the parent test should be selected. Its two exact children each own one event loop.

use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{Arc, Weak},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{
        pump_events::{EventLoopExtPumpEvents, PumpStatus},
        ActiveEventLoop, EventLoop,
    },
    platform::windows::EventLoopBuilderExtWindows,
    window::{Window, WindowId},
};
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowResolution};
use zircon_runtime_interface::{ZrRuntimeNativeSurfaceTargetV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::{RuntimeEntryApp, RuntimeEntryAppConfig, RuntimeEntryAppFailureState};
use crate::entry::runtime_entry_app::window_surface::runtime_native_surface_target;
use crate::entry::runtime_library::surface_teardown_fixture::{
    current_native_thread_id, native_hwnd_is_live, with_session, Call, NativeWindowObservation,
    Policy, ProviderControl,
};

const CHILD_MODE_ENV: &str = "ZR_APP_PHYSICAL_WINDOW_CHILD";
const OUTPUT_ENV: &str = "ZR_APP_PHYSICAL_WINDOW_TEST_OUTPUT";
const ALLOWED_OUTPUT_ROOT: &str =
    r"D:\cargo-targets\zircon-local\evidence\engine-audit-20260930-01a0f06a";
const ENTERED: &str = "ZR_APP_NATIVE_WINDOW_CLEANUP_ENTERED";
const RETURNED: &str = "ZR_APP_NATIVE_WINDOW_CLEANUP_RETURNED";
const OS_DESTROYED: &str = "ZR_APP_NATIVE_WINDOW_OS_DESTROYED";
const OUTER_GUARD: &str = "ZR_APP_NATIVE_WINDOW_OUTER_GUARD";
const INTERNAL_GUARD: &str = "ZR_APP_NATIVE_WINDOW_INTERNAL_GUARD";
const COMPLETE: &str = "ZR_APP_NATIVE_WINDOW_COMPLETE";
const PANICKED: &str = "ZR_APP_NATIVE_WINDOW_PANICKED";
const OWNER_VIOLATION: &str = "ZR_APP_NATIVE_WINDOW_OWNER_VIOLATION";
const EARLY_RELEASE: &str = "ZR_APP_NATIVE_WINDOW_COUNTERFACTUAL_RELEASED";
const CHILD_WAIT: Duration = Duration::from_secs(15);
const TERMINATION_WAIT: Duration = Duration::from_secs(1);
const PUMP_WAIT: Duration = Duration::from_secs(5);
const PUMP_SLICE: Duration = Duration::from_millis(10);
const LOG_LIMIT: u64 = 64 * 1024;

#[test]
#[ignore = "creates hidden native Windows; run this exact parent with an approved output directory"]
fn hidden_window_is_owned_through_runtime_destroy_and_real_destroyed_does_not_reenter() {
    let output = approved_output();
    let test_module = module_path!()
        .split_once("::")
        .expect("test module includes crate name")
        .1;
    let child_name = format!("{test_module}::hidden_window_cleanup_child");
    run_exact_child(&output, &child_name, "preserved");
    run_exact_child(&output, &child_name, "released");
}

fn approved_output() -> PathBuf {
    let configured = std::env::var_os(OUTPUT_ENV).expect("set ZR_APP_PHYSICAL_WINDOW_TEST_OUTPUT");
    let output =
        std::fs::canonicalize(configured).expect("the configured output directory must exist");
    let allowed =
        std::fs::canonicalize(ALLOWED_OUTPUT_ROOT).expect("the approved evidence root must exist");
    let drive = std::fs::canonicalize(r"D:\").expect("approved physical drive root");
    let physical_drive_d = match drive.components().next() {
        Some(std::path::Component::Prefix(prefix)) => matches!(
            prefix.kind(),
            std::path::Prefix::Disk(b'D') | std::path::Prefix::VerbatimDisk(b'D')
        ),
        _ => false,
    };
    assert!(
        physical_drive_d,
        "the approved drive may not resolve through an alias"
    );
    assert_eq!(
        allowed,
        drive.join(r"cargo-targets\zircon-local\evidence\engine-audit-20260930-01a0f06a"),
        "the approved evidence root may not be redirected"
    );
    assert!(
        output.starts_with(&allowed),
        "all test output must physically remain under {allowed:?}"
    );
    output
}

fn run_exact_child(output: &Path, test_name: &str, mode: &str) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let log_path = output.join(format!(
        "native-window-{}-{stamp}-{mode}.stderr",
        std::process::id()
    ));
    let log = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&log_path)
        .expect("create the isolated child's log in the approved output directory");
    let mut child = Command::new(std::env::current_exe().expect("current libtest executable"))
        .args([
            "--exact",
            test_name,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_MODE_ENV, mode)
        .env(OUTPUT_ENV, output)
        .env("TEMP", output)
        .env("TMP", output)
        .env("TMPDIR", output)
        .env_remove("ZIRCON_RUNTIME_MVP_INPUT_PROBE")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(
            log.try_clone().expect("clone the log file handle"),
        ))
        .spawn()
        .expect("start only the exact isolated physical-window child");
    let status = wait_for_exact_child(&mut child);
    drop(log);
    let file = File::open(&log_path).expect("read the exited child's log");
    assert!(
        file.metadata().expect("log metadata").len() <= LOG_LIMIT,
        "child log exceeded the bounded diagnostic size: {log_path:?}"
    );
    let mut bytes = Vec::new();
    file.take(LOG_LIMIT)
        .read_to_end(&mut bytes)
        .expect("bounded child log read");
    let stderr = String::from_utf8_lossy(&bytes);
    assert!(
        stderr.contains(ENTERED),
        "child did not reach actual Window cleanup: {stderr}"
    );
    assert!(
        stderr.contains(RETURNED),
        "the selected cleanup did not return: {stderr}"
    );
    if mode == "preserved" {
        assert!(
            status.success(),
            "normal owner child failed: {status}: {stderr}"
        );
        for marker in [OS_DESTROYED, OUTER_GUARD, INTERNAL_GUARD, COMPLETE] {
            assert!(stderr.contains(marker), "missing {marker}: {stderr}");
        }
        assert!(
            !stderr.contains(PANICKED),
            "panic cannot be accepted through Drop: {stderr}"
        );
        assert!(!stderr.contains(OWNER_VIOLATION), "{stderr}");
    } else {
        assert_eq!(
            status.code(),
            Some(87),
            "counterfactual must fail before Rust Drop: {stderr}"
        );
        assert!(stderr.contains(EARLY_RELEASE), "{stderr}");
        assert!(
            stderr.contains(&format!("{OWNER_VIOLATION} strong_count=0")),
            "{stderr}"
        );
        assert!(
            stderr.contains(PANICKED),
            "owner assertion must be the exact failure: {stderr}"
        );
        assert!(
            !stderr.contains(COMPLETE),
            "a released App owner cannot pass this oracle: {stderr}"
        );
        assert!(
            !stderr.contains("fatal runtime session teardown failure:"),
            "Drop cannot validate the oracle: {stderr}"
        );
        assert!(
            !stderr.contains("fatal fixture API lifetime failure:"),
            "fixture Drop cannot validate the oracle: {stderr}"
        );
    }
}

fn wait_for_exact_child(child: &mut Child) -> ExitStatus {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) if started.elapsed() < CHILD_WAIT => std::thread::sleep(PUMP_SLICE),
            result => {
                // This parent owns this exact child handle. Timeout/status errors are always failures.
                let kill = child.kill();
                let termination = Instant::now();
                while termination.elapsed() < TERMINATION_WAIT {
                    match child.try_wait() {
                        Ok(Some(status)) => panic!("child missed {CHILD_WAIT:?}: {result:?}; kill={kill:?}; exited={status}"),
                        Ok(None) => std::thread::sleep(PUMP_SLICE),
                        Err(error) => panic!("child status failed: {result:?}; kill={kill:?}; error={error}"),
                    }
                }
                panic!(
                    "child {} missed deadline; kill={kill:?}; termination was not observed",
                    child.id()
                );
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Mode {
    Preserved,
    Released,
}

#[test]
#[ignore = "exact child entry; only the physical-window parent supplies its mode"]
fn hidden_window_cleanup_child() {
    let mode = match std::env::var(CHILD_MODE_ENV).as_deref() {
        Ok("preserved") => Mode::Preserved,
        Ok("released") => Mode::Released,
        _ => panic!("this helper must be launched by the exact parent"),
    };
    // Fail immediately, before unwinding can call App/Session/fixture Drop and produce a false pass.
    std::panic::set_hook(Box::new(|panic| {
        eprintln!("{PANICKED}: {panic}");
        std::process::exit(87);
    }));
    with_session(
        Policy {
            unbind_failures: usize::MAX,
            ..Policy::default()
        },
        |session, control| {
            let configuration = RuntimeEntryAppConfig::default().with_window_descriptor(
                WindowDescriptor::default()
                    .with_title("Zircon App owner regression")
                    .with_resolution(WindowResolution::new(320, 200))
                    .with_visible(false)
                    .with_focused(false),
            );
            let app = RuntimeEntryApp::new(
                session,
                configuration,
                RuntimeEntryAppFailureState::default(),
            );
            assert_eq!(app.session.creator_thread(), std::thread::current().id());
            let mut driver = WindowDriver {
                app,
                control,
                mode,
                window_id: None,
                owner: None,
                target: None,
                destroyed: false,
            };
            let mut builder = EventLoop::builder();
            builder.with_any_thread(true);
            let mut event_loop = builder
                .build()
                .expect("create this child's only event loop");
            let started = Instant::now();
            while !driver.destroyed {
                assert!(
                    started.elapsed() < PUMP_WAIT,
                    "no real OS Destroyed before {PUMP_WAIT:?}"
                );
                let status = event_loop.pump_app_events(Some(PUMP_SLICE), &mut driver);
                if let PumpStatus::Exit(code) = status {
                    assert_eq!(code, 0, "event-loop exit code");
                    assert!(
                        driver.destroyed,
                        "event loop exited without the real target's Destroyed"
                    );
                }
            }
            let target = driver.target.expect("the actual bind target");
            assert!(
                !native_hwnd_is_live(target),
                "HWND remains live after the real message pump returned"
            );
            assert_eq!(
                driver
                    .owner
                    .as_ref()
                    .expect("actual App Weak")
                    .strong_count(),
                0
            );
            let calls = control.snapshot().calls;
            assert!(
                driver.app.teardown_primary_window(),
                "repeat completed teardown is idempotent"
            );
            assert_eq!(
                control.snapshot().calls,
                calls,
                "repeat cleanup reentered the provider"
            );
            drop(driver.app);
            assert_eq!(
                control.snapshot().calls,
                calls,
                "App/Session Drop reentered the provider"
            );
            assert!(!control.snapshot().active);
            assert!(!control.snapshot().bound);
        },
    );
    // The real App, Session, API table owner and provider registration have already dropped.
    eprintln!("{COMPLETE}");
}

struct WindowDriver<'a> {
    app: RuntimeEntryApp,
    control: &'a ProviderControl,
    mode: Mode,
    window_id: Option<WindowId>,
    owner: Option<Weak<dyn Window>>,
    target: Option<ZrRuntimeNativeSurfaceTargetV1>,
    destroyed: bool,
}

impl ApplicationHandler for WindowDriver<'_> {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window_id.is_some() {
            return;
        }
        // Run the actual production admission/create/bind path from Winit's admitted callback.
        ApplicationHandler::can_create_surfaces(&mut self.app, event_loop);
        assert!(
            !self.app.failure_state.is_recorded(),
            "real App window creation failed"
        );
        assert!(self.app.surface_present_enabled);
        assert!(
            self.app.presenter.is_none(),
            "a presenter must not retain a second Window Arc"
        );
        let (id, owner, target) = {
            let window = self
                .app
                .window
                .as_ref()
                .expect("actual hidden App-owned Window");
            (
                window.id(),
                Arc::downgrade(window),
                runtime_native_surface_target(window.as_ref())
                    .expect("actual Win32 raw handle pair"),
            )
        };
        assert_eq!(
            target,
            ZrRuntimeNativeSurfaceTargetV1::win32(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                target.window_handle,
                target.display_handle
            )
        );
        assert_ne!(target.window_handle, 0);
        assert_eq!(
            self.control.last_bound_target(),
            Some(target),
            "provider must receive that actual Window target"
        );
        let initial = self.control.watch_native_window(owner.clone());
        assert_owned_window(initial);
        self.window_id = Some(id);
        self.owner = Some(owner);
        self.target = Some(target);
        // Real creation already delivered viewport-resized; compare only the subsequent cleanup calls.
        let cleanup_start = self.control.snapshot().calls.len();
        if self.mode == Mode::Released {
            // Deliberate old-order counterfactual. No local Arc clone delays this release.
            drop(self.app.window.take());
            assert_eq!(self.owner.as_ref().unwrap().strong_count(), 0);
            eprintln!("{EARLY_RELEASE}");
        }
        eprintln!("{ENTERED}");
        let released = self.app.teardown_primary_window();
        eprintln!("{RETURNED}");
        assert!(
            !released,
            "persistent unbind failure stays a product failure"
        );
        assert!(self.app.failure_state.is_recorded());
        assert_eq!(
            self.app
                .failure_state
                .failure_ledger()
                .snapshot()
                .records()
                .len(),
            1
        );
        assert!(!self.control.snapshot().active);
        assert!(!self.control.snapshot().bound);
        assert!(self.app.window.is_none());
        assert_eq!(self.owner.as_ref().unwrap().strong_count(), 0);
        let observations = self.control.native_window_observations();
        assert_eq!(
            observations
                .iter()
                .map(|probe| probe.call)
                .collect::<Vec<_>>(),
            [Call::Unbind, Call::Unbind, Call::Destroy]
        );
        let calls = self.control.snapshot().calls;
        let cleanup_calls = calls[cleanup_start..]
            .iter()
            .copied()
            .filter(|call| matches!(call, Call::Unbind | Call::Destroy | Call::Event))
            .collect::<Vec<_>>();
        assert_eq!(cleanup_calls, [Call::Unbind, Call::Unbind, Call::Destroy]);
        for observation in observations {
            if observation.strong_count != 1 {
                eprintln!(
                    "{OWNER_VIOLATION} strong_count={} hwnd_live={} call={:?}",
                    observation.strong_count, observation.hwnd_live, observation.call
                );
            }
            assert_owned_window(observation);
        }
        // No synthetic Destroyed is dispatched. Subsequent pumps must deliver WM_DESTROY.
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window_id != Some(window_id) || !matches!(event, WindowEvent::Destroyed) {
            return;
        }
        assert!(!self.destroyed, "duplicate actual OS Destroyed");
        let calls = self.control.snapshot().calls;
        let failures = self
            .app
            .failure_state
            .failure_ledger()
            .snapshot()
            .records()
            .len();
        // Covers the actual production outer hook for this real OS event after terminal destroy.
        ApplicationHandler::window_event(&mut self.app, event_loop, window_id, event);
        assert_eq!(self.control.snapshot().calls, calls);
        assert_eq!(
            self.app
                .failure_state
                .failure_ledger()
                .snapshot()
                .records()
                .len(),
            failures
        );
        eprintln!("{OUTER_GUARD}");
        // The outer guard normally suppresses this inner consumer. Call its real method here,
        // inside the observed OS callback, to check its defensive cleanup/event guard separately.
        self.app.handle_window_destroyed(event_loop);
        assert_eq!(self.control.snapshot().calls, calls);
        assert_eq!(
            self.app
                .failure_state
                .failure_ledger()
                .snapshot()
                .records()
                .len(),
            failures
        );
        eprintln!("{INTERNAL_GUARD}");
        self.destroyed = true;
        eprintln!("{OS_DESTROYED}");
    }
    // No App frame pump or RedrawRequested delegation: this gate never calls genuine RHI slots.
}

fn assert_owned_window(observation: NativeWindowObservation) {
    assert_eq!(
        observation.strong_count, 1,
        "the actual App Arc must cover every callback: {observation:?}"
    );
    assert!(
        observation.hwnd_live,
        "borrowed HWND must stay live: {observation:?}"
    );
    assert!(
        !observation.visible,
        "the physical test Window must stay hidden: {observation:?}"
    );
    assert_ne!(observation.window_thread, 0, "valid HWND creation thread");
    assert_eq!(observation.window_thread, observation.owner_thread);
    assert_eq!(observation.callback_thread, observation.owner_thread);
    assert_eq!(observation.callback_thread, current_native_thread_id());
}
