//! Headless native Play fixture: real project/catalog/VM, no GPU or staged DLL.
//! ZE_ORBRUN_TEST_ROOT must name an explicitly owned fixture directory. Retain
//! fixtures for failure diagnosis instead of writing into the example or /tmp.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::builtin::manifest_with_mode_baseline;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::math::Vec3;
use zircon_runtime::dynamic_api::{create_linked_runtime_play_session, zircon_runtime_get_api_v8};
use zircon_runtime::scene::{LevelSystem, SystemStage, World};
use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeApiV8, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

pub(super) const PLAYER: u64 = 4;
pub(super) const FIRST_ORB: u64 = 10;
pub(super) const CLOCK: u64 = 19;
pub(super) const SCORE: u64 = 20;
const VIEWPORT: ZrRuntimeViewportHandle = ZrRuntimeViewportHandle::new(1);

pub(super) struct Play {
    api: &'static ZrRuntimeApiV8,
    handle: ZrRuntimeSessionHandle,
    observed: Arc<Mutex<Option<LevelSystem>>>,
    pub root: PathBuf,
}

impl Play {
    pub fn example(name: &str) -> Self {
        Self::new(name, None)
    }

    pub fn failing_binding(phase: &str) -> Self {
        Self::new(phase, Some(phase))
    }

    pub fn missing_binding(fixed: bool) -> Self {
        let phase = if fixed {
            "missing_fixed"
        } else {
            "missing_start"
        };
        Self::new(phase, Some(phase))
    }

    fn new(name: &str, failing_phase: Option<&str>) -> Self {
        let fixture_root = PathBuf::from(std::env::var_os("ZE_ORBRUN_TEST_ROOT").expect(
            "set ZE_ORBRUN_TEST_ROOT to an explicitly owned directory for retained Play fixtures",
        ));
        assert!(fixture_root.is_absolute(), "fixture root must be absolute");
        let root = fixture_root.join(format!(
            "{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/orbrun");
        for relative in [
            "zircon-project.toml",
            "assets/scenes/main.scene.toml",
            "scripts/orb_run/plugin.toml",
            "scripts/orb_run/plugin.zrp",
            "scripts/orb_run/main.zr",
        ] {
            let destination = root.join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(source.join(relative), destination).unwrap();
        }
        let mut project = ProjectManifest::load(root.join("zircon-project.toml")).unwrap();
        let scene = if let Some(phase) = failing_phase {
            install_failure_probe(&root, phase);
            project.scripts.startup_packages.push("b3_probe".into());
            project.save(root.join("zircon-project.toml")).unwrap();
            "assets/scenes/u03.scene.toml"
        } else {
            "assets/scenes/main.scene.toml"
        };
        // Same project resolution and providers as AppPlaySessionFactory::create_linked.
        let manifest =
            manifest_with_mode_baseline(RuntimeTargetMode::ClientRuntime, Some(&project.plugins));
        let catalog =
            crate::entry::first_party_runtime_plugins::first_party_runtime_catalog_for_manifest(
                RuntimeTargetMode::ClientRuntime,
                &manifest,
            );
        let mut registrations = catalog
            .runtime_plugins
            .into_registrations_if_required_resolved_where(|selection| {
                !selection.is_runtime_builtin_domain()
            })
            .unwrap();
        for registration in registrations.iter_mut() {
            registration.project_selection.enabled = true;
        }
        // A read-only First observer gives assertions access to the actual session's
        // World, including untyped Play-only rows absent from editor projections.
        let observed = Arc::new(Mutex::new(None));
        let rendering = registrations
            .iter_mut()
            .find(|report| report.package_manifest.id == "rendering")
            .unwrap();
        let owner = rendering
            .extensions
            .intern_plugin_module("rendering.runtime")
            .unwrap();
        let output = observed.clone();
        rendering
            .extensions
            .register_runtime_scene_system(
                owner,
                "rendering.b3_orbrun_observer",
                SystemStage::First,
                move || {
                    let output = output.clone();
                    move |context| {
                        *output.lock().unwrap() = Some(context.level.clone());
                        Ok(())
                    }
                },
            )
            .register()
            .unwrap();
        // SAFETY: null selects the supported built-in host; the linked table has
        // process lifetime and remains loaded for every call made by this owner.
        let api = unsafe { zircon_runtime_get_api_v8(std::ptr::null()) };
        assert!(!api.is_null());
        // SAFETY: checked non-null process-lifetime table from the linked runtime.
        let api = unsafe { &*api };
        let handle = create_linked_runtime_play_session(
            b"headless",
            &root,
            &RelPath::parse(scene).unwrap(),
            registrations.into_iter().collect(),
            catalog.runtime_plugin_features,
        )
        .unwrap();
        let play = Self {
            api,
            handle,
            observed,
            root,
        };
        play.event(ZrRuntimeEventV1::viewport_resized(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            VIEWPORT,
            ZrRuntimeViewportSizeV1::new(640, 360),
        ));
        play
    }

    pub fn read<R>(&self, read: impl FnOnce(&World) -> R) -> R {
        self.observed
            .lock()
            .unwrap()
            .as_ref()
            .expect("First observer must have run")
            .with_world(read)
    }

    pub fn write<R>(&self, write: impl FnOnce(&mut World) -> R) -> R {
        self.observed
            .lock()
            .unwrap()
            .as_ref()
            .expect("First observer must have run")
            .with_world_mut(write)
    }

    pub fn fixed_elapsed(&self) -> Duration {
        self.observed
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .world_time()
            .fixed_time()
            .elapsed()
    }

    pub fn tick_result(&self) -> Result<(), String> {
        // Allow a nonzero native wall-clock delta without replacing the product clock.
        std::thread::sleep(Duration::from_millis(2));
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        // SAFETY: live session, creator thread, exclusive initialized output.
        status_result(unsafe { self.api.tick_frame.unwrap()(self.handle, &mut demand) })
    }

    pub fn tick(&self) {
        self.tick_result().unwrap();
    }

    fn event(&self, event: ZrRuntimeEventV1) {
        // SAFETY: live session consumes this valid event synchronously. No event
        // contains borrowed payload storage beyond this call.
        status_result(unsafe { self.api.handle_event.unwrap()(self.handle, event) }).unwrap();
    }

    pub fn click(&self) {
        // Center of the 640x360 fallback menu's actual Start/Retry rectangle.
        self.event(ZrRuntimeEventV1::pointer_moved(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            VIEWPORT,
            320.0,
            248.0,
        ));
        for state in [
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        ] {
            self.event(ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                VIEWPORT,
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                state,
                320.0,
                248.0,
            ));
        }
    }

    pub fn key(&self, key: u8, down: bool) {
        self.event(ZrRuntimeEventV1::keyboard(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            VIEWPORT,
            if down {
                ZR_RUNTIME_KEY_ACTION_PRESSED_V1
            } else {
                ZR_RUNTIME_KEY_ACTION_RELEASED_V1
            },
            u32::from(key),
            0,
            ZrByteSlice::empty(),
        ));
    }

    pub fn component(&self, entity: u64, name: &str) -> Value {
        self.read(|world| {
            world
                .dynamic_component(entity, name)
                .cloned()
                .unwrap_or(Value::Null)
        })
    }

    pub fn state(&self) -> Value {
        self.component(PLAYER, "orb_run.run_state")
    }

    pub fn position(&self, entity: u64) -> Vec3 {
        self.read(|world| world.local_transform(entity).unwrap().translation)
    }

    pub fn set_position(&self, entity: u64, position: Vec3) {
        self.write(|world| {
            let mut transform = world.local_transform(entity).unwrap();
            transform.translation = position;
            world.update_transform(entity, transform).unwrap();
        });
    }

    pub fn set_component(&self, entity: u64, name: &str, value: Value) {
        self.write(|world| world.set_dynamic_component(entity, name, value).unwrap());
    }

    pub fn hp(&self, entity: u64) -> f64 {
        self.component(entity, "script.bindings")[0]["properties"]["hp"]
            .as_f64()
            .unwrap()
    }

    pub fn set_hp(&self, entity: u64, hp: f64) {
        let mut bindings = self.component(entity, "script.bindings");
        bindings[0]["properties"]["hp"] = json!(hp);
        self.set_component(entity, "script.bindings", bindings);
    }

    pub fn start(&self) {
        self.tick();
        assert_eq!(self.read(|world| world.node_records().len()), 19);
        assert_eq!(self.state(), "start_menu");
        assert_eq!(
            self.component(PLAYER, "gameplay.menu_state")["button"],
            "Start"
        );
        self.click();
        assert_eq!(
            self.component(PLAYER, "gameplay.control_state"),
            "start_game"
        );
        self.tick();
        assert_eq!(self.state(), "reset");
        assert_eq!(self.component(PLAYER, "gameplay.control_state"), "none");
        assert_eq!(self.component(PLAYER, "gameplay.menu_state"), Value::Null);
        self.tick();
        assert_eq!(self.state(), "playing");
    }
}

impl Drop for Play {
    fn drop(&mut self) {
        // Release the observation clone before creator-thread session retirement.
        self.observed.lock().unwrap().take();
        // SAFETY: sole destruction authority, live linked session, no borrowed outputs.
        let result = status_result(unsafe { self.api.destroy_session.unwrap()(self.handle) });
        if !std::thread::panicking() {
            result.unwrap();
        }
    }
}

fn status_result(status: ZrStatus) -> Result<(), String> {
    if status.status_code() == ZrStatusCode::Ok {
        return Ok(());
    }
    // SAFETY: copy the linked provider's bounded synchronous diagnostic before
    // any subsequent ABI call can overwrite its thread-local diagnostic buffer.
    let bytes = unsafe { status.diagnostics.checked_slice(64 * 1024) }.unwrap();
    Err(format!(
        "{:?}: {}",
        status.status_code(),
        String::from_utf8_lossy(bytes)
    ))
}

fn install_failure_probe(root: &Path, phase: &str) {
    std::fs::create_dir_all(root.join("scripts/b3_probe")).unwrap();
    std::fs::write(
        root.join("scripts/b3_probe/plugin.toml"),
        r#"
name = "b3_probe"
version = "0.1.0"
entry = "main"
backend = "zr_vm:project"
[capabilities]
capabilities = ["gameplay.entity"]
[zr_vm]
project = "plugin.zrp"
entry_module = "main"
execution_mode = "interp"
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("scripts/b3_probe/plugin.zrp"),
        r#"{"name":"b3_probe","source":".","binary":"bin","entry":"main"}"#,
    )
    .unwrap();
    std::fs::write(
        root.join("scripts/b3_probe/main.zr"),
        r#"
let gameplay = import("zr.zircon.gameplay");
pub fn activate(): int { return 0; }
pub fn deactivate(): int { return 0; }
pub fn onStart(entity: int, dt: float): int {
    gameplay.translate(entity, 1.0, 0.0, 0.0);
    if (gameplay.script_property_matches(entity, "fail", "start")) {
        gameplay.set_component_json(entity, "b3_probe.marker", "{");
    }
    return 0;
}
pub fn onUpdate(entity: int, dt: float): int {
    gameplay.translate(entity, 0.0, 1.0, 0.0);
    if (gameplay.script_property_matches(entity, "fail", "update")) {
        gameplay.set_component_json(entity, "b3_probe.marker", "{");
    }
    return 0;
}
pub fn onFixedUpdate(entity: int, dt: float): int {
    gameplay.translate(entity, 0.0, 0.0, 1.0);
    if (gameplay.script_property_matches(entity, "fail", "fixed")) {
        gameplay.set_component_json(entity, "b3_probe.marker", "{");
    }
    return 0;
}
"#,
    )
    .unwrap();
    let mut scene = String::new();
    for entity in 30..=32 {
        let fail = if entity == 31 { phase } else { "none" };
        let fixed = phase == "fixed" || phase == "missing_fixed";
        let package = if entity == 31 && phase.starts_with("missing_") {
            "b3_missing"
        } else {
            "b3_probe"
        };
        scene.push_str(&format!(
            r#"
[[entities]]
entity = {entity}
name = "Probe {entity}"
mobility = "Dynamic"
transform = {{ translation = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }}
[[entities.script_bindings]]
package = "{package}"
module = "main"
enabled = true
update = {}
fixed_update = {fixed}
[entities.script_bindings.properties]
fail = "{fail}"
"#,
            !fixed
        ));
    }
    std::fs::write(root.join("assets/scenes/u03.scene.toml"), scene).unwrap();
}
