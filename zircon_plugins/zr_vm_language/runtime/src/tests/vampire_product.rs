use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::{ProjectManifest, ProjectScriptManifest};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::dynamic_api::{create_linked_runtime_session, zircon_runtime_get_api_v8};
use zircon_runtime_interface::world_sync::{WorldQuery, WorldQueryResult};
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse, ZrByteSlice,
    ZrOwnedResultV2, ZrRuntimeApiV8, ZrRuntimeEventV1, ZrRuntimeFrameDemandV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_KEY_ACTION_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

use crate::plugin_registration;

const VAMPIRE_SCRIPT_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../examples/vampire/scripts/vampire_game/main.zr"
));
const VIEWPORT_WIDTH: u32 = 640;
const VIEWPORT_HEIGHT: u32 = 360;
const PLAYER_ENTITY: u64 = 2;
const FRAGILE_ENEMY_ENTITY: u64 = 20;
const CHASER_ENTITY: u64 = 21;

#[test]
fn real_zrvm_vampire_product_runs_gameplay_menu_hud_and_diagnostics_through_public_abi() {
    let fixture = VampireProductFixture::new();
    let session = RuntimeSession::new(&fixture.root);

    let chaser_before_menu = session.transform(CHASER_ENTITY);
    session.tick();
    assert_vec3_close(
        session.transform(CHASER_ENTITY),
        chaser_before_menu,
        "the start menu must pause enemy behavior",
    );
    assert!(session.hierarchy_entities().contains(&FRAGILE_ENEMY_ENTITY));
    let start_menu = session.capture();

    session.click_start_button();
    session.tick();
    let player_before_input = session.transform(PLAYER_ENTITY);
    let chaser_before_gameplay = session.transform(CHASER_ENTITY);
    let distance_before = planar_distance(chaser_before_gameplay, player_before_input);

    session.keyboard(b'W', ZR_RUNTIME_KEY_ACTION_PRESSED_V1);
    session.tick();
    session.keyboard(b'W', ZR_RUNTIME_KEY_ACTION_RELEASED_V1);

    let player_after_input = session.transform(PLAYER_ENTITY);
    let chaser_after_gameplay = session.transform(CHASER_ENTITY);
    let distance_after = planar_distance(chaser_after_gameplay, player_after_input);
    assert!(
        player_after_input[2] < player_before_input[2],
        "W input must move the real ZrVM player toward -Z: before={player_before_input:?} after={player_after_input:?}"
    );
    assert!(
        distance_after < distance_before,
        "the real ZrVM enemy must chase the player: before={distance_before} after={distance_after}"
    );
    assert!(
        !session.hierarchy_entities().contains(&FRAGILE_ENEMY_ENTITY),
        "the automatic Blood Bolt must remove the one-hit enemy through the production gameplay host"
    );

    let gameplay = session.capture();
    assert_eq!(
        gameplay.rgba.len(),
        (VIEWPORT_WIDTH * VIEWPORT_HEIGHT * 4) as usize
    );
    assert_ne!(
        start_menu.rgba, gameplay.rgba,
        "starting gameplay must change the captured product frame"
    );
    assert!(
        count_world_hud_pixels(&gameplay.rgba) > 16,
        "the gameplay capture must contain scene-following Vampire health-bar pixels"
    );

    let diagnostics = session.runtime_diagnostics();
    assert_eq!(
        diagnostics.project_identity.as_deref(),
        Some("ZrVM Vampire Product Fixture")
    );
    assert_eq!(
        diagnostics.scene_uri.as_deref(),
        Some("res://scenes/main.scene.toml")
    );
    assert!(diagnostics.frame_index >= 3);
    assert!(diagnostics.input.viewport_resize_count >= 1);
    assert!(diagnostics.input.pointer_move_count >= 1);
    assert!(diagnostics.input.mouse_button_press_count >= 1);
    assert!(diagnostics.input.mouse_button_release_count >= 1);
    assert!(diagnostics.input.keyboard_press_count >= 1);
    assert!(diagnostics.input.keyboard_release_count >= 1);
}

struct VampireProductFixture {
    root: PathBuf,
}

impl VampireProductFixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time follows the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-zr-vm-vampire-product-{}-{nonce}",
            std::process::id()
        ));
        let scene_dir = root.join("assets/scenes");
        let script_dir = root.join("scripts/vampire_game");
        fs::create_dir_all(&scene_dir).expect("create Vampire scene fixture directory");
        fs::create_dir_all(script_dir.join("bin"))
            .expect("create Vampire script output fixture directory");

        let mut manifest = ProjectManifest::new(
            "ZrVM Vampire Product Fixture",
            AssetUri::parse("res://scenes/main.scene.toml")
                .expect("fixture scene URI is canonical"),
            1,
        );
        manifest.scripts = ProjectScriptManifest {
            package_roots: vec!["scripts".to_string()],
            startup_packages: vec!["vampire_game".to_string()],
        };
        manifest
            .save(root.join("zircon-project.toml"))
            .expect("write Vampire project fixture manifest");
        fs::write(scene_dir.join("main.scene.toml"), VAMPIRE_SCENE)
            .expect("write Vampire scene fixture");
        fs::write(script_dir.join("plugin.toml"), VAMPIRE_PLUGIN_MANIFEST)
            .expect("write Vampire script package manifest");
        fs::write(script_dir.join("plugin.zrp"), VAMPIRE_PROJECT_MANIFEST)
            .expect("write Vampire ZrVM project manifest");
        fs::write(script_dir.join("main.zr"), VAMPIRE_SCRIPT_SOURCE)
            .expect("write immutable Vampire script fixture");

        Self { root }
    }
}

impl Drop for VampireProductFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct RuntimeSession {
    api: &'static ZrRuntimeApiV8,
    handle: ZrRuntimeSessionHandle,
}

impl RuntimeSession {
    fn new(project_root: &std::path::Path) -> Self {
        let api = runtime_api();
        let handle = create_linked_runtime_session(
            b"runtime",
            Some(project_root),
            vec![plugin_registration()],
        )
        .expect("create linked Vampire runtime session with the real ZrVM plugin");
        Self { api, handle }
    }

    fn tick(&self) {
        let tick = self.api.tick_frame.expect("runtime tick entry point");
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        let status = unsafe { tick(self.handle, &mut demand) };
        assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    }

    fn event(&self, event: ZrRuntimeEventV1) {
        let handle_event = self.api.handle_event.expect("runtime event entry point");
        let status = unsafe { handle_event(self.handle, event) };
        assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    }

    fn click_start_button(&self) {
        let viewport = ZrRuntimeViewportHandle::new(1);
        self.event(ZrRuntimeEventV1::viewport_resized(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZrRuntimeViewportSizeV1::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        ));
        let x = VIEWPORT_WIDTH as f32 * 0.5;
        let y = VIEWPORT_HEIGHT as f32 * 0.5 + 68.0;
        self.event(ZrRuntimeEventV1::pointer_moved(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            x,
            y,
        ));
        self.event(ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
            x,
            y,
        ));
        self.event(ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
            x,
            y,
        ));
    }

    fn keyboard(&self, key: u8, action: u32) {
        self.event(ZrRuntimeEventV1::keyboard(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            action,
            u32::from(key),
            0,
            ZrByteSlice::empty(),
        ));
    }

    fn query(&self, query: WorldQuery) -> WorldQueryResult {
        let query_world = self
            .api
            .query_world
            .expect("runtime world query entry point");
        let request = serde_json::to_vec(&query).expect("encode world query");
        let mut output = ZrOwnedResultV2::empty();
        let status = unsafe {
            query_world(
                self.handle,
                ZrByteSlice {
                    data: request.as_ptr(),
                    len: request.len(),
                },
                &mut output,
            )
        };
        assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
        self.decode_and_release(output)
    }

    fn transform(&self, entity: u64) -> [f32; 3] {
        match self.query(WorldQuery::transform_snapshot(entity)) {
            WorldQueryResult::TransformSnapshot { transform, .. } => [
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            ],
            result => panic!("entity {entity} has no transform snapshot: {result:?}"),
        }
    }

    fn hierarchy_entities(&self) -> Vec<u64> {
        match self.query(WorldQuery::hierarchy(None)) {
            WorldQueryResult::HierarchyRows { rows, .. } => {
                rows.into_iter().map(|row| row.entity).collect()
            }
            result => panic!("runtime did not return hierarchy rows: {result:?}"),
        }
    }

    fn capture(&self) -> CapturedFrame {
        let capture = self
            .api
            .capture_frame
            .expect("runtime frame capture entry point");
        let mut frame = ZrRuntimeFrameV2::empty(ZIRCON_RUNTIME_ABI_VERSION_V2);
        let status = unsafe {
            capture(
                self.handle,
                ZrRuntimeFrameRequestV1::new(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZrRuntimeViewportHandle::new(1),
                    ZrRuntimeViewportSizeV1::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
                ),
                &mut frame,
            )
        };
        assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
        let rgba = owned_bytes(&frame.rgba);
        self.release(frame.rgba);
        CapturedFrame { rgba }
    }

    fn runtime_diagnostics(&self) -> zircon_runtime_interface::RuntimeDiagnosticsSnapshot {
        let profile_control = self
            .api
            .profile_control
            .expect("runtime profile control entry point");
        let request = serde_json::to_vec(&ProfileControlRequest {
            command: ProfileControlCommand::RuntimeDiagnosticsSnapshot,
            config: None,
        })
        .expect("encode runtime diagnostics request");
        let mut output = ZrOwnedResultV2::empty();
        let status = unsafe {
            profile_control(
                self.handle,
                ZrByteSlice {
                    data: request.as_ptr(),
                    len: request.len(),
                },
                &mut output,
            )
        };
        assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
        let response: ProfileControlResponse = self.decode_and_release(output);
        assert_eq!(response.status, "ok", "{}", response.message);
        response
            .runtime_diagnostics
            .expect("runtime diagnostics snapshot is present")
    }

    fn decode_and_release<T: serde::de::DeserializeOwned>(&self, output: ZrOwnedResultV2) -> T {
        let bytes = owned_bytes(&output);
        self.release(output);
        serde_json::from_slice(&bytes).expect("decode runtime-owned JSON result")
    }

    fn release(&self, output: ZrOwnedResultV2) {
        let release = self
            .api
            .release_allocation
            .expect("runtime allocation release entry point");
        let status = unsafe { release(self.handle, output.allocation) };
        assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    }
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        if let Some(destroy) = self.api.destroy_session {
            let _ = unsafe { destroy(self.handle) };
        }
    }
}

struct CapturedFrame {
    rgba: Vec<u8>,
}

fn runtime_api() -> &'static ZrRuntimeApiV8 {
    let api = unsafe { zircon_runtime_get_api_v8(core::ptr::null()) };
    assert!(!api.is_null(), "runtime rejected the default host API");
    unsafe { &*api }
}

fn owned_bytes(output: &ZrOwnedResultV2) -> Vec<u8> {
    let len =
        usize::try_from(output.len).expect("runtime result length fits the host address space");
    unsafe { core::slice::from_raw_parts(output.data, len) }.to_vec()
}

fn planar_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    let x = left[0] - right[0];
    let z = left[2] - right[2];
    (x * x + z * z).sqrt()
}

fn assert_vec3_close(actual: [f32; 3], expected: [f32; 3], message: &str) {
    let delta = actual
        .into_iter()
        .zip(expected)
        .map(|(actual, expected)| (actual - expected).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        delta <= 0.001,
        "{message}: actual={actual:?} expected={expected:?}"
    );
}

fn count_world_hud_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| {
            let red =
                pixel[0] >= 170 && (90..=170).contains(&pixel[1]) && (95..=190).contains(&pixel[2]);
            let green = (110..=200).contains(&pixel[0]) && pixel[1] >= 150 && pixel[2] <= 140;
            let blue = pixel[0] <= 120 && pixel[1] >= 120 && pixel[2] >= 160;
            pixel[3] >= 180 && (red || green || blue)
        })
        .count()
}

const VAMPIRE_PLUGIN_MANIFEST: &str = r#"name = "vampire_game"
version = "0.1.0"
entry = "main"
backend = "zr_vm:project"

[capabilities]
capabilities = [
  "foundation.log",
  "foundation.time",
  "gameplay.input",
  "gameplay.entity",
  "gameplay.navigation",
]

[zr_vm]
project = "plugin.zrp"
entry_module = "main"
execution_mode = "interp"
"#;

const VAMPIRE_PROJECT_MANIFEST: &str = r#"{
  "name": "vampire_game",
  "source": ".",
  "binary": "bin",
  "entry": "main"
}
"#;

const VAMPIRE_SCENE: &str = r#"[[entities]]
entity = 1
name = "Follow Camera"
active = true
render_layer_mask = 1
mobility = "Dynamic"

[entities.transform]
translation = [0.0, 5.35, -7.25]
rotation = [0.0, 0.9673612, 0.2534013, 0.0]
scale = [1.0, 1.0, 1.0]

[entities.camera]
fov_y_radians = 0.88
z_near = 0.1
z_far = 240.0
hdr = true
exposure_ev100 = 9.2

[[entities]]
entity = 2
name = "Count Veyr - Player"
active = true
render_layer_mask = 1
mobility = "Dynamic"

[entities.transform]
translation = [0.0, 0.0, 0.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]

[[entities.script_bindings]]
package = "vampire_game"
module = "main"
enabled = true
fixed_update = false

[entities.script_bindings.properties]
role = "player"
archetype = "vampire"
hp = 120
move_speed = 5.2

[[entities]]
entity = 20
name = "One-Hit Skeleton"
active = true
render_layer_mask = 1
mobility = "Dynamic"

[entities.transform]
translation = [0.0, 0.0, -3.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]

[[entities.script_bindings]]
package = "vampire_game"
module = "main"
enabled = true
fixed_update = false

[entities.script_bindings.properties]
role = "enemy"
archetype = "skeleton"
behavior_tree = "graveyard_enemy_bt"
hp = 1
move_speed = 3.3
contact_damage = 8
xp = 1

[[entities]]
entity = 21
name = "Skeleton Chaser"
active = true
render_layer_mask = 1
mobility = "Dynamic"

[entities.transform]
translation = [5.0, 0.0, -4.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]

[[entities.script_bindings]]
package = "vampire_game"
module = "main"
enabled = true
fixed_update = false

[entities.script_bindings.properties]
role = "enemy"
archetype = "skeleton"
behavior_tree = "graveyard_enemy_bt"
hp = 24
move_speed = 3.3
contact_damage = 8
xp = 1

[[entities]]
entity = 118
name = "Player Blood Aura Light"
active = true
render_layer_mask = 1
mobility = "Dynamic"

[entities.transform]
translation = [0.0, 1.2, 0.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]

[entities.point_light]
color = [0.85, 0.04, 0.08]
intensity = 2.6
range = 4.4
"#;
