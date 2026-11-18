//! Public linked-session regression: real input routing and World mutation, no GPU.

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use zircon_runtime::{
    asset::{project::ProjectPaths, AssetUri, ProjectManifest},
    dynamic_api::{create_linked_runtime_session, zircon_runtime_get_api_v8},
    input::{InputEvent, InputIngressPhase, TouchPhase, TouchPoint},
};
use zircon_runtime_interface::{
    ZrRuntimeApiV8, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeSessionHandle, ZrRuntimeViewportSizeV1, ZrStatus,
    ZIRCON_RUNTIME_ABI_VERSION_V1 as ABI, ZIRCON_RUNTIME_API_VERSION_V8,
    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1 as VIEWPORT, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
};

#[path = "touch_cancel_public/owner.rs"]
mod owner;

use owner::{Inbox, Observation};

use Edge::{Enter, Leave, Touch};
use TouchPhase::{Cancelled, Ended, Moved, Started};

const SCENE: &str = "res://main.scene.toml";
const IMPORTER: &str = "ui_document_importer.zui_document";
const FINGER: u64 = 17;
const SIZE: ZrRuntimeViewportSizeV1 = ZrRuntimeViewportSizeV1::new(640, 360);
// At this viewport the fallback layout's button is (180, 220), 280 x 48.
// Use a center and a near miss still within the panel; Ended tests the hit region.
const INSIDE: [f32; 2] = [320.0, 244.0];
const OUTSIDE: [f32; 2] = [320.0, 216.0];
const START: (&str, &str) = ("start", "start_game");
const RETRY: (&str, &str) = ("game_over", "retry_game");

const SCENE_TEXT: &str = r#"
[[entities]]
entity = 1
name = "Touch fixture camera"
mobility = "Dynamic"

[entities.transform]
translation = [0.0, 0.0, 5.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]

[entities.camera]
z_near = 0.1
z_far = 100.0
"#;

struct ProjectDir {
    path: PathBuf,
    preserve: bool,
}

impl ProjectDir {
    fn new() -> Self {
        // Check isolation before calling the ordinary rlib (no cfg(test) path overrides).
        let declared = |name: &str| {
            let path = PathBuf::from(
                env::var_os(name)
                    .unwrap_or_else(|| panic!("set {name} in the isolated test process")),
            );
            assert!(path.is_absolute(), "{name} must be absolute");
            path
        };
        let temp = ProjectPaths::resolve_existing_path(declared("ZIRCON_TOUCH_TEST_ROOT"))
            .expect("owned test root");
        let effective_temp =
            ProjectPaths::resolve_existing_path(env::temp_dir()).expect("owned TEMP");
        assert_eq!(effective_temp, temp, "TEMP must select the owned test root");
        let logs = ProjectPaths::resolve_existing_path(declared("ZIRCON_LOG_ROOT"))
            .expect("owned log directory");
        assert!(
            logs.is_dir() && logs.starts_with(&temp),
            "log directory must be inside the test root"
        );
        let config = declared("ZIRCON_CONFIG_PATH");
        assert!(config.parent().is_some_and(Path::is_dir) && !config.is_dir());
        let physical = ProjectPaths::resolve_root(&config).expect("resolve owned config file");
        assert!(
            physical != temp && physical.starts_with(&temp),
            "config must be inside the test root"
        );
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = temp.join(format!("z-touch-{}-{stamp:x}-{id}", std::process::id()));
        fs::create_dir(&path).expect("create exclusive project directory");
        let dir = Self {
            path,
            preserve: false,
        };
        let paths = ProjectPaths::from_root(&dir.path).expect("project paths");
        let manifest =
            ProjectManifest::new("Touch cancel fixture", AssetUri::parse(SCENE).unwrap(), 1);
        paths
            .ensure_layout(&manifest.asset_roots)
            .expect("project layout");
        fs::write(
            paths
                .asset_root(&manifest.asset_roots[0])
                .join("main.scene.toml"),
            SCENE_TEXT,
        )
        .expect("author camera-only scene");
        manifest
            .save(paths.manifest_path())
            .expect("save no-root project");
        dir
    }
}

impl Drop for ProjectDir {
    fn drop(&mut self) {
        if !self.preserve {
            if let Err(error) = fs::remove_dir_all(&self.path) {
                eprintln!(
                    "failed to remove owned touch fixture {:?}: {error}",
                    self.path
                );
            }
        }
    }
}

fn assert_ok(step: &str, status: ZrStatus) {
    if !status.is_ok() {
        // Runtime lends these bytes until the next ABI call on this thread.
        let diagnostic =
            unsafe { status.diagnostics.checked_slice(64 * 1024) }.map(String::from_utf8_lossy);
        panic!("{step}: {:?} {diagnostic:?}", status.status_code());
    }
}

struct Fixture {
    api: &'static ZrRuntimeApiV8,
    handle: ZrRuntimeSessionHandle,
    inbox: Inbox,
    project: ProjectDir,
}

impl Fixture {
    fn new(state: &'static str) -> (Self, Observation) {
        let api = unsafe { zircon_runtime_get_api_v8(std::ptr::null()).as_ref() }
            .expect("linked V8 table");
        assert_eq!(api.abi_version, ZIRCON_RUNTIME_API_VERSION_V8);
        assert!(api.destroy_session.is_some());
        let project = ProjectDir::new();
        let inbox = Inbox::default();
        let reports = vec![
            zircon_plugin_ui_document_importer_runtime::plugin_registration(),
            owner::registration(state, &inbox),
        ];
        for report in &reports {
            assert!(
                report.is_success(),
                "plugin registration: {:?}",
                report.diagnostics
            );
        }
        let handle = create_linked_runtime_session(b"minimal", Some(&project.path), reports)
            .unwrap_or_else(|error| {
                panic!("start real linked project {:?}: {error}", project.path)
            });
        let fixture = Self {
            api,
            handle,
            inbox,
            project,
        };
        fixture.event(ZrRuntimeEventV1::viewport_resized(ABI, VIEWPORT, SIZE));
        let baseline = fixture.tick();
        eprintln!("PUBLIC_TOUCH_CANCEL_SETUP_V1 state={state} size=640x360 {baseline:?}");
        let receipt = baseline.project.as_ref().expect("owner bootstrap receipt");
        assert_eq!(receipt.root, fixture.project.path, "installed PM root");
        assert_eq!(
            receipt.scene.as_deref(),
            Some(SCENE),
            "installed level source"
        );
        assert_eq!(receipt.ui_roots, 0, "fallback requires no authored roots");
        assert_eq!(receipt.importer, IMPORTER, "real active .zui importer");
        assert!(
            receipt.importer_available,
            "active .zui provider must be executable"
        );
        assert_ne!(
            baseline.owner,
            thread::current().id(),
            "read on Runtime owner"
        );
        assert_eq!(baseline.slot.menu.as_deref(), Some(state));
        assert_eq!(baseline.slot.command.as_deref(), Some("none"));
        assert!(baseline.slot.menu_generation > 0 && baseline.slot.command_generation > 0);
        assert!(baseline.ingress.generation.is_some());
        assert!(baseline.active.is_empty() && baseline.journal.is_empty());
        assert!(baseline.recording.enabled && baseline.recording.discarded_records == 0);
        // Present runs the real CPU fallback extraction before Minimal skips the render bridge.
        // Ok is not a readback, nor proof of any particular command being in the extract.
        let present = api.present_viewport.expect("V8 present");
        assert_ok("CPU menu extraction (not framebuffer)", unsafe {
            present(handle, ZrRuntimeFrameRequestV1::new(ABI, VIEWPORT, SIZE))
        });
        eprintln!("PUBLIC_TOUCH_CANCEL_CPU_PRESENT_V1 state={state} submitted_to_gpu=false");
        (fixture, baseline)
    }

    fn event(&self, event: ZrRuntimeEventV1) {
        let send = self.api.handle_event.expect("V8 handle_event");
        assert_ok("ABI input/resize", unsafe { send(self.handle, event) });
    }

    fn tick(&self) -> Observation {
        let tick = self.api.tick_frame.expect("V8 tick_frame");
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        assert_ok("ABI tick_frame (owner observation)", unsafe {
            tick(self.handle, &mut demand)
        });
        assert!(demand.is_valid(), "invalid V8 frame demand: {demand:?}");
        self.inbox
            .lock()
            .unwrap()
            .take()
            .expect("one PostUpdate owner observation")
    }

    fn gesture(
        &self,
        step: &str,
        before: &Observation,
        edges: &[Edge],
        active: Option<TouchPoint>,
        command: &str,
    ) -> Observation {
        let mut expected = Vec::new();
        let mut inside = before.cursor_inside;
        // No tick while a complete pointer group is in flight.
        for &edge in edges {
            let event = match edge {
                Enter => {
                    inside = true;
                    expected.push(InputEvent::CursorEntered);
                    ZrRuntimeEventV1::cursor_entered(ABI, VIEWPORT)
                }
                Leave => {
                    inside = false;
                    expected.push(InputEvent::CursorLeft);
                    ZrRuntimeEventV1::cursor_left(ABI, VIEWPORT)
                }
                Touch(phase, [x, y]) => {
                    expected.push(InputEvent::CursorMoved { x, y });
                    expected.push(InputEvent::Touch {
                        id: FINGER,
                        phase,
                        x,
                        y,
                    });
                    ZrRuntimeEventV1::touch(ABI, VIEWPORT, FINGER, touch_code(phase), x, y)
                }
            };
            self.event(event);
        }
        let after = self.tick();
        eprintln!("PUBLIC_TOUCH_CANCEL_SAMPLE_V1 menu={:?} step={step} expected={command:?} prior_command_generation={} {after:?}", before.slot.menu, before.slot.command_generation);
        assert!(after.project.is_none(), "{step}: fixture must not reseed");
        assert_eq!(after.owner, before.owner, "{step}: same Runtime owner");
        assert_eq!(after.slot.world, before.slot.world, "{step}: same level");
        assert_eq!(
            after.slot.epoch, before.slot.epoch,
            "{step}: no World replacement"
        );
        assert_eq!(
            after.slot.entity, before.slot.entity,
            "{step}: same command entity"
        );
        assert_eq!(
            after.slot.menu, before.slot.menu,
            "{step}: no menu state mutation"
        );
        assert_eq!(
            after.slot.menu_generation, before.slot.menu_generation,
            "{step}: same menu generation"
        );
        assert_eq!(after.ingress.generation, before.ingress.generation);
        assert_eq!(after.ingress.phase, InputIngressPhase::Receiving);
        assert_eq!(
            after.ingress.rejected_batches,
            before.ingress.rejected_batches
        );
        assert_eq!(
            after.ingress.accepted_batches,
            before.ingress.accepted_batches + edges.len() as u64
        );
        assert_eq!(after.ingress.frame_events, expected.len());
        assert!(after.recording.enabled && after.recording.discarded_records == 0);
        assert_eq!(after.journal.len(), expected.len());
        let first = before
            .journal
            .last()
            .map_or(1, |record| record.sequence + 1);
        for (index, (actual, expected)) in after.journal.iter().zip(&expected).enumerate() {
            assert_eq!(actual.sequence, first + index as u64);
            assert_eq!(&actual.event, expected, "{step}: exact physical journal");
        }
        assert_eq!(
            after.cursor_inside, inside,
            "{step}: window cursor presence"
        );
        assert_eq!(
            after.active.as_slice(),
            active.as_slice(),
            "{step}: active touch snapshot"
        );
        eprintln!(
            "PUBLIC_TOUCH_CANCEL_PHYSICAL_OK_V1 menu={:?} step={step}",
            before.slot.menu
        );
        assert_eq!(
            after.slot.command.as_deref(),
            Some(command),
            "{step}: World command slot"
        );
        let delta = (before.slot.command.as_deref() != Some(command)) as u64;
        assert_eq!(
            after.slot.command_generation,
            before.slot.command_generation + delta,
            "{step}: World command generation"
        );
        after
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let destroy = self.api.destroy_session.expect("V8 destroy_session");
        let status = unsafe { destroy(self.handle) };
        if !status.is_ok() {
            self.project.preserve = true; // A failed owner teardown may still be using this root.
            if thread::panicking() {
                eprintln!(
                    "V8 destroy failed ({:?}); preserving {:?}",
                    status.status_code(),
                    self.project.path
                );
            } else {
                assert_ok("ABI destroy_session", status);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Edge {
    Enter,
    Leave,
    Touch(TouchPhase, [f32; 2]),
}

fn touch_code(phase: TouchPhase) -> u32 {
    match phase {
        Started => ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
        Moved => ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
        Ended => ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
        Cancelled => ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    }
}

fn cancel_inside(state: &'static str) {
    // Observe the live pre-Cancel point in a different session to avoid adding a tick to the
    // actual Started -> CursorLeft -> Cancelled group. There is deliberately no Ended in it.
    {
        let (prefix, setup) = Fixture::new(state);
        prefix.gesture(
            "prefix-through-left",
            &setup,
            &[Enter, Touch(Started, INSIDE), Leave],
            Some(TouchPoint {
                id: FINGER,
                position: INSIDE,
                phase: Started,
            }),
            "none",
        );
    }
    let (fixture, setup) = Fixture::new(state);
    fixture.gesture(
        "inside-cancelled",
        &setup,
        &[
            Enter,
            Touch(Started, INSIDE),
            Leave,
            Touch(Cancelled, INSIDE),
        ],
        None,
        "none",
    );
}

#[test]
#[ignore = "requires an isolated linked project; run the touch_cancel_public target with --ignored"]
fn public_touch_cancel_start_stays_inert() {
    cancel_inside(START.0);
}

#[test]
#[ignore = "requires an isolated linked project; run the touch_cancel_public target with --ignored"]
fn public_touch_cancel_retry_stays_inert() {
    cancel_inside(RETRY.0);
}

#[test]
#[ignore = "requires an isolated linked project; run the touch_cancel_public target with --ignored"]
fn public_touch_cancel_ended_activates_real_menu() {
    for (state, command) in [START, RETRY] {
        let (fixture, setup) = Fixture::new(state);
        fixture.gesture(
            "inside-ended",
            &setup,
            &[Enter, Touch(Started, INSIDE), Touch(Ended, INSIDE)],
            None,
            command,
        );
    }
}

#[test]
#[ignore = "requires an isolated linked project; run the touch_cancel_public target with --ignored"]
fn public_touch_cancel_outside_is_inert_and_finger_is_reusable() {
    for (state, command) in [START, RETRY] {
        let (fixture, setup) = Fixture::new(state);
        let ended = fixture.gesture(
            "outside-ended",
            &setup,
            &[
                Enter,
                Touch(Started, INSIDE),
                Touch(Moved, OUTSIDE),
                Touch(Ended, OUTSIDE),
            ],
            None,
            "none",
        );
        let cancelled = fixture.gesture(
            "outside-cancelled",
            &ended,
            &[
                Enter,
                Touch(Started, INSIDE),
                Touch(Moved, OUTSIDE),
                Leave,
                Touch(Cancelled, OUTSIDE),
            ],
            None,
            "none",
        );
        fixture.gesture(
            "same-finger-ended-after-cancel",
            &cancelled,
            &[Enter, Touch(Started, INSIDE), Touch(Ended, INSIDE)],
            None,
            command,
        );
    }
}
