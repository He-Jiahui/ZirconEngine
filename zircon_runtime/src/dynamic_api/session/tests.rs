use std::{env, path::Path};

use crate::core::diagnostics::collect_runtime_diagnostics;
use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::{Transform, Vec3};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

use super::{
    extract_stats::{EXTRACT_OUTPUT_BYTES_DIAGNOSTIC, EXTRACT_REBUILD_CLONES_DIAGNOSTIC},
    runtime_session_error, RuntimeDynamicSession, RuntimeDynamicSessionProfile,
    RuntimeProjectConfig,
};

fn vampire_project_config() -> RuntimeProjectConfig {
    let root = env::var_os("ZR_VAMPIRE_PROJECT_ROOT")
        .map(Into::into)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("workspace root")
                .join("examples")
                .join("vampire")
        });
    let root = root.to_string_lossy();
    RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
        data: root.as_ptr(),
        len: root.len(),
    })
    .unwrap()
    .unwrap()
}

#[test]
fn vampire_project_session_w_key_moves_player_before_input_clear() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    let before = entity_position(&session, 2);
    let pose_before = vampire_actor_node_local_transforms(&session);
    let status = session.handle_event(ZrRuntimeEventV1::keyboard(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
        u32::from(b'W'),
        0,
        ZrByteSlice::empty(),
    ));

    assert!(status.is_ok(), "{status:?}");
    session.tick_frame().unwrap();

    let after = entity_position(&session, 2);
    let pose_after = vampire_actor_node_local_transforms(&session);
    let animation_parameters = animation_state_machine_parameters(&session, 2);
    assert!(
        after.z < before.z,
        "W input should move vampire player forward toward the third-person camera view on -Z: before={before:?} after={after:?}"
    );
    assert_eq!(
        animation_parameters.get("moving"),
        Some(&AnimationParameterValue::Bool(true)),
        "W input should drive the player locomotion animation parameter"
    );
    assert_eq!(
        animation_parameters.get("attacking"),
        Some(&AnimationParameterValue::Bool(true)),
        "movement can overlap automatic Blood Bolt when an enemy is in range"
    );
    assert!(
        pose_after
            .iter()
            .any(|(entity, transform)| pose_before.get(entity) != Some(transform)),
        "movement should make the animation state machine pose visible on vampire body nodes: before={pose_before:?} after={pose_after:?}"
    );
}

#[test]
fn vampire_project_session_wasd_axes_match_third_person_camera() {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join("examples/vampire/scripts/vampire_game/main.zr"),
    )
    .unwrap();
    for marker in [
        "if (gameplay.key_pressed(\"W\")) {\n            dz = -1.0;",
        "if (gameplay.key_pressed(\"S\")) {\n            dz = 1.0;",
        "if (gameplay.key_pressed(\"A\")) {\n            dx = 1.0;",
        "if (gameplay.key_pressed(\"D\")) {\n            dx = -1.0;",
    ] {
        assert!(
            source.contains(marker),
            "vampire ZR script should map WASD to the corrected third-person axes, missing marker: {marker}"
        );
    }
}

#[test]
fn vampire_project_session_starts_paused_until_start_button_click() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    let player_before = entity_position(&session, 2);
    let enemy_before = entity_position(&session, 20);

    session.tick_frame().unwrap();

    assert_eq!(
        dynamic_component_string(&session, 2, "vampire.run_state").as_deref(),
        Some("start_menu"),
        "first tick should put the vampire example into the start menu"
    );
    assert_eq!(
        dynamic_component_value(&session, 2, "gameplay.menu_state")
            .as_ref()
            .and_then(|value| value.get("state"))
            .and_then(serde_json::Value::as_str),
        Some("start")
    );
    capture_vampire_frame_for_env(&mut session, "ZR_VAMPIRE_START_MENU_CAPTURE_PNG");
    assert_eq!(entity_position(&session, 2), player_before);
    assert_eq!(
        entity_position(&session, 20),
        enemy_before,
        "enemy should not chase while the start menu is active"
    );

    click_vampire_menu_button(&mut session, 640, 360);
    session.tick_frame().unwrap();

    assert_eq!(
        dynamic_component_string(&session, 2, "vampire.run_state").as_deref(),
        Some("playing")
    );
    assert!(
        dynamic_component_value(&session, 2, "gameplay.menu_state")
            .as_ref()
            .map(serde_json::Value::is_null)
            .unwrap_or(true),
        "start menu state should clear after clicking Start Game"
    );
}

#[test]
fn vampire_project_session_game_over_menu_retries_to_playing() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    set_script_binding_number(&session, 2, "hp", 0.1);
    set_entity_position(&session, 20, Vec3::new(0.4, 0.0, 0.0));

    session.tick_frame().unwrap();

    assert_eq!(
        dynamic_component_string(&session, 2, "vampire.run_state").as_deref(),
        Some("game_over")
    );
    assert_eq!(
        dynamic_component_value(&session, 2, "gameplay.menu_state")
            .as_ref()
            .and_then(|value| value.get("state"))
            .and_then(serde_json::Value::as_str),
        Some("game_over")
    );
    capture_vampire_frame_for_env(&mut session, "ZR_VAMPIRE_GAME_OVER_CAPTURE_PNG");

    click_vampire_menu_button(&mut session, 640, 360);
    session.tick_frame().unwrap();

    assert_eq!(
        dynamic_component_string(&session, 2, "vampire.run_state").as_deref(),
        Some("playing")
    );
    assert_eq!(
        script_binding_number(&session, 2, "hp"),
        Some(120.0),
        "Retry should reset the player health"
    );
    assert_vec3_close(entity_position(&session, 2), Vec3::ZERO);
}

#[test]
fn vampire_project_session_auto_blood_bolt_damages_nearest_enemy() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    let before = script_binding_number(&session, 20, "hp").unwrap();

    session.tick_frame().unwrap();

    let after = script_binding_number(&session, 20, "hp").unwrap();
    let player_action_state = dynamic_component_i64(&session, 2, "vampire.action_state");
    let particles = dynamic_component_value(&session, 2, "render.particle_sprites")
        .expect("automatic Blood Bolt should emit a particle component on the player");
    let animation_parameters = animation_state_machine_parameters(&session, 2);
    assert!(
        after < before,
        "automatic Blood Bolt should damage nearest enemy 20: before={before} after={after}"
    );
    assert_eq!(
        player_action_state,
        Some(2),
        "automatic Blood Bolt should put the player in attack action state"
    );
    assert_eq!(
        animation_parameters.get("attacking"),
        Some(&AnimationParameterValue::Bool(true)),
        "automatic Blood Bolt should drive the player attack animation parameter"
    );
    assert!(
        particles.as_array().is_some_and(|sprites| !sprites.is_empty())
            || particles
                .get("sprites")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|sprites| !sprites.is_empty()),
        "Blood Bolt should author particle sprites through render.particle_sprites, got {particles:?}"
    );
}

#[test]
fn vampire_project_session_enemy_behavior_tree_chases_player() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    let player_before = entity_position(&session, 2);
    let enemy_before = entity_position(&session, 20);
    let distance_before = planar_distance(enemy_before, player_before);

    session.tick_frame().unwrap();

    let player_after = entity_position(&session, 2);
    let enemy_after = entity_position(&session, 20);
    let distance_after = planar_distance(enemy_after, player_after);
    assert_eq!(
        dynamic_component_i64(&session, 20, "vampire.action_state"),
        Some(1),
        "enemy 20 should enter the behavior-tree chase/run action state"
    );
    assert_eq!(
        dynamic_component_i64(&session, 20, "vampire.behavior_node"),
        Some(31),
        "enemy 20 should report the chase behavior node after evaluating the behavior tree"
    );
    assert!(
        distance_after < distance_before,
        "enemy 20 should move toward the player through nav-preferred chase: before={distance_before} after={distance_after}"
    );
}

#[test]
fn vampire_project_session_writes_world_hud_for_scene_authored_enemies() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    let scene_enemies = script_property_entities(&session, "role", "enemy");
    assert!(
        scene_enemies.len() >= 3,
        "the stable real-VM slice should use authored script-driven enemy actors instead of per-frame script spawning"
    );

    for _ in 0..3 {
        session.tick_frame().unwrap();
    }

    let hud = dynamic_component_string(&session, 2, "gameplay.hud_text");
    let player_health_bar = world_hud_bar(&session, 2)
        .expect("player health must be authored as scene-following world HUD data");
    let enemies_after = script_property_entities(&session, "role", "enemy");
    let enemy_with_world_hud = enemies_after
        .iter()
        .copied()
        .find(|entity| world_hud_bar(&session, *entity).is_some());
    let health_bar_fills = script_property_entities(&session, "role", "health_bar_fill");
    let health_bar_backs = script_property_entities(&session, "role", "health_bar_back");
    let player_position = entity_position(&session, 2);
    assert!(
        hud.is_none(),
        "project gameplay should not write a screen-space upper-left combat HUD text, got: {hud:?}"
    );
    assert_world_hud_bar_tracks_position(
        &player_health_bar,
        player_position + Vec3::new(0.0, 1.92, 0.0),
        "player health world HUD should follow the player",
    );
    assert!(
        enemy_with_world_hud.is_some(),
        "at least one enemy should author scene-following world health HUD data"
    );
    assert!(
        enemies_after.len() == scene_enemies.len(),
        "the real-VM hot path should keep the authored enemy set stable: before={} after={}",
        scene_enemies.len(),
        enemies_after.len()
    );
    assert!(
        health_bar_fills.is_empty() && health_bar_backs.is_empty(),
        "health bars must be scene UI HUD state, not spawned mesh entities: fills={} backs={}",
        health_bar_fills.len(),
        health_bar_backs.len()
    );
}

#[test]
fn vampire_project_session_simple_loop_kills_enemy_without_mesh_health_bars() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    remove_script_entities_by_role_except(&session, "enemy", Some(20));
    remove_script_entities_by_role_except(&session, "boss", None);
    set_entity_position(&session, 20, Vec3::new(0.0, 0.0, -3.0));
    set_script_binding_number(&session, 20, "hp", 1.0);

    session.tick_frame().unwrap();

    assert!(
        session
            .level
            .with_world(|world| world.find_node(20).is_none()),
        "automatic attack should remove a killed enemy through the scripted damage path"
    );
    assert!(script_property_entities(&session, "role", "health_bar_fill").is_empty());
    assert!(script_property_entities(&session, "role", "health_bar_back").is_empty());
}

#[test]
fn vampire_project_session_keeps_hud_after_runtime_ticks() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);

    for _ in 0..12 {
        session.tick_frame().unwrap();
    }

    let hp = script_binding_number(&session, 2, "hp")
        .expect("player entity should remain alive for the playable demo loop");
    let player_health_bar = world_hud_bar(&session, 2)
        .expect("player world HUD should remain attached after extended runtime");
    let player_position = entity_position(&session, 2);
    let screen_hud = dynamic_component_string(&session, 2, "gameplay.hud_text");
    assert_world_hud_bar_tracks_position(
        &player_health_bar,
        player_position + Vec3::new(0.0, 1.92, 0.0),
        "runtime player world HUD should continue tracking the player",
    );
    assert!(
        screen_hud.is_none() && hp >= 1.0,
        "runtime should keep scene-following HUD data and the player entity alive without screen HUD text, hud={screen_hud:?}, hp={hp}"
    );
}

#[test]
fn vampire_project_session_capture_frame_draws_world_hud_bars() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);

    for _ in 0..vampire_capture_tick_count() {
        session.tick_frame().unwrap();
    }

    let frame = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            vampire_capture_viewport_size(),
        ))
        .unwrap();
    let rgba = if frame.rgba.data.is_null() || frame.rgba.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(frame.rgba.data.cast_const(), frame.rgba.len) }
    };
    let top_left_panel_pixels = count_hud_panel_pixels(rgba, frame.width, frame.height);
    let world_hud_bar_pixels = count_world_hud_bar_pixels(rgba, frame.width, frame.height);
    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let render_stats = diagnostics
        .render
        .stats
        .as_ref()
        .expect("render stats should be available after capture");
    let hud_pixel_summary = summarize_hud_region(rgba, frame.width, frame.height);
    let player_world_hud = world_hud_bar(&session, 2);
    let enemy_world_hud_count = script_property_entities(&session, "role", "enemy")
        .into_iter()
        .filter(|entity| world_hud_bar(&session, *entity).is_some())
        .count();
    export_vampire_capture_frame_if_requested(
        "ZR_VAMPIRE_CAPTURE_PNG",
        rgba,
        frame.width,
        frame.height,
    );

    assert_eq!(
        render_stats.last_ui_command_count, 0,
        "captured vampire frame should not rely on screen-space combat HUD commands; color-only top-left heuristic matched {top_left_panel_pixels} scene pixels; hud_region={hud_pixel_summary}; ui_passes={} executed_passes={:?} executor_ids={:?}",
        render_stats.last_ui_graph_executed_pass_count,
        render_stats.last_graph_executed_passes,
        render_stats.last_graph_executed_executor_ids,
    );
    assert!(
        player_world_hud.is_some() && enemy_world_hud_count > 0,
        "captured vampire frame should carry scene-following world HUD bars for player and enemies: player={player_world_hud:?} enemy_count={enemy_world_hud_count}"
    );
    assert!(
        world_hud_bar_pixels > 48,
        "captured vampire frame should contain colored in-scene health-bar pixels, found {world_hud_bar_pixels}; hud_region={hud_pixel_summary}"
    );
    assert!(
        render_stats.last_shadow_graph_executed_pass_count > 0
            || render_stats.last_mesh_shadow_caster_draw_count > 0,
        "captured vampire frame should exercise the shadow path, shadow_passes={} shadow_casters={}",
        render_stats.last_shadow_graph_executed_pass_count,
        render_stats.last_mesh_shadow_caster_draw_count
    );
    assert!(
        render_stats.last_particle_graph_executed_pass_count > 0
            || render_stats.last_particle_gpu_alive_count > 0
            || dynamic_component_value(&session, 2, "render.particle_sprites").is_some(),
        "captured vampire frame should carry attack particle data, particle_passes={} alive={}",
        render_stats.last_particle_graph_executed_pass_count,
        render_stats.last_particle_gpu_alive_count
    );
}

#[test]
fn headless_session_capture_records_frame_extract_diagnostics() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless session");

    session
        .capture_frame(small_headless_frame_request())
        .expect("headless capture");

    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let rebuild_clones =
        diagnostic_current(&diagnostics, EXTRACT_REBUILD_CLONES_DIAGNOSTIC).unwrap_or_default();
    let output_bytes =
        diagnostic_current(&diagnostics, EXTRACT_OUTPUT_BYTES_DIAGNOSTIC).unwrap_or_default();

    assert_eq!(
        rebuild_clones, 1.0,
        "headless capture should record one current full extract rebuild clone"
    );
    assert!(
        output_bytes > 0.0,
        "headless capture should record a non-empty extract output byte estimate"
    );
}

#[test]
fn frame_extract_rebuild_skips_unchanged_entities() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless session");

    session
        .capture_frame(small_headless_frame_request())
        .expect("first headless capture");
    session
        .capture_frame(small_headless_frame_request())
        .expect("second headless capture");

    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let rebuilds = diagnostic_series(&diagnostics, EXTRACT_REBUILD_CLONES_DIAGNOSTIC)
        .expect("extract rebuild diagnostics");
    let output_bytes = diagnostic_series(&diagnostics, EXTRACT_OUTPUT_BYTES_DIAGNOSTIC)
        .expect("extract output byte diagnostics");

    assert_eq!(
        rebuilds.history.len(),
        2,
        "current baseline records one extract rebuild sample per capture"
    );
    assert!(
        rebuilds.history.iter().all(|sample| sample.value == 1.0),
        "current baseline still rebuilds the full extract for unchanged captures"
    );
    assert_eq!(output_bytes.history.len(), 2);
    assert!(output_bytes.history[0].value > 0.0);
    assert_eq!(
        output_bytes.history[0].value, output_bytes.history[1].value,
        "unchanged headless captures should keep the extract output byte baseline stable"
    );
}

#[test]
fn vampire_project_session_reports_runtime_fps_and_render_work() {
    let mut session = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(vampire_project_config()),
    )
    .unwrap();
    start_vampire_game(&mut session);
    let tick_count = vampire_diagnostic_tick_count();

    for _ in 0..tick_count {
        session.tick_frame().unwrap();
    }

    let frame = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            vampire_capture_viewport_size(),
        ))
        .unwrap();
    let diagnostics = collect_runtime_diagnostics(&session.runtime.handle());
    let fps = diagnostic_current(&diagnostics, "time.fps");
    let frame_ms = diagnostic_current(&diagnostics, "time.frame_time");
    let render_stats = diagnostics
        .render
        .stats
        .as_ref()
        .expect("render stats should be available after capture");

    println!(
        "vampire_runtime_perf ticks={} capture={}x{} fps_current={:?} frame_ms_current={:?} submitted_frames={} graph_passes={} ui_passes={} particle_passes={} shadow_passes={} mesh_draws={} ui_commands={}",
        tick_count,
        frame.width,
        frame.height,
        fps,
        frame_ms,
        render_stats.submitted_frames,
        render_stats.last_graph_executed_pass_count,
        render_stats.last_ui_graph_executed_pass_count,
        render_stats.last_particle_graph_executed_pass_count,
        render_stats.last_shadow_graph_executed_pass_count,
        render_stats.last_mesh_draw_count,
        render_stats.last_ui_command_count,
    );

    assert!(
        render_stats.submitted_frames > 0,
        "diagnostic run should submit at least one rendered frame"
    );
    let fps = fps.expect("runtime diagnostics should report time.fps for the vampire scene");
    assert!(
        fps >= 60.0,
        "vampire runtime diagnostics should remain at or above 60 FPS after hot-path trimming, fps={fps:?} frame_ms={frame_ms:?}"
    );
}

fn script_binding_number(
    session: &RuntimeDynamicSession,
    entity: u64,
    property: &str,
) -> Option<f64> {
    session.level.with_world(|world| {
        world
            .dynamic_component(entity, "script.bindings")?
            .as_array()?
            .iter()
            .find_map(|binding| binding.get("properties")?.get(property)?.as_f64())
    })
}

fn script_property_entities(
    session: &RuntimeDynamicSession,
    property: &str,
    expected: &str,
) -> Vec<u64> {
    session.level.with_world(|world| {
        world
            .node_records()
            .into_iter()
            .filter(|node| {
                world
                    .dynamic_component(node.id, "script.bindings")
                    .is_some_and(|bindings| {
                        script_binding_property_matches(bindings, property, expected)
                    })
            })
            .map(|node| node.id)
            .collect()
    })
}

fn script_binding_property_matches(
    bindings: &serde_json::Value,
    property: &str,
    expected: &str,
) -> bool {
    bindings.as_array().is_some_and(|bindings| {
        bindings.iter().any(|binding| {
            binding
                .get("enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
                && binding
                    .get("properties")
                    .and_then(|properties| properties.get(property))
                    .and_then(serde_json::Value::as_str)
                    == Some(expected)
        })
    })
}

fn set_entity_position(session: &RuntimeDynamicSession, entity: u64, position: Vec3) {
    session.level.with_world_mut(|world| {
        let mut transform = world.world_transform(entity).unwrap_or_default();
        transform.translation = position;
        world
            .update_transform(entity, transform)
            .expect("test entity transform should be mutable");
    });
}

fn set_script_binding_number(
    session: &RuntimeDynamicSession,
    entity: u64,
    property: &str,
    value: f64,
) {
    session.level.with_world_mut(|world| {
        let mut bindings = world
            .dynamic_component(entity, "script.bindings")
            .cloned()
            .unwrap_or_else(empty_vampire_script_bindings);
        let binding = bindings
            .as_array_mut()
            .and_then(|bindings| bindings.first_mut())
            .expect("test script bindings should contain one binding");
        let properties = binding
            .get_mut("properties")
            .and_then(serde_json::Value::as_object_mut)
            .expect("test script binding should contain properties");
        properties.insert(property.to_string(), serde_json::json!(value));
        world
            .set_dynamic_component(entity, "script.bindings", bindings)
            .expect("test script bindings should be writable");
    });
}

fn remove_script_entities_by_role_except(
    session: &RuntimeDynamicSession,
    role: &str,
    keep: Option<u64>,
) {
    let entities = script_property_entities(session, "role", role);
    session.level.with_world_mut(|world| {
        for entity in entities {
            if Some(entity) != keep {
                world.remove_entity(entity);
            }
        }
    });
}

fn empty_vampire_script_bindings() -> serde_json::Value {
    serde_json::json!([{
        "package": "vampire_game",
        "module": "main",
        "enabled": true,
        "properties": {}
    }])
}

fn dynamic_component_i64(
    session: &RuntimeDynamicSession,
    entity: u64,
    component_id: &str,
) -> Option<i64> {
    session.level.with_world(|world| {
        world
            .dynamic_component(entity, component_id)
            .and_then(serde_json::Value::as_i64)
    })
}

fn dynamic_component_value(
    session: &RuntimeDynamicSession,
    entity: u64,
    component_id: &str,
) -> Option<serde_json::Value> {
    session
        .level
        .with_world(|world| world.dynamic_component(entity, component_id).cloned())
}

fn dynamic_component_string(
    session: &RuntimeDynamicSession,
    entity: u64,
    component_id: &str,
) -> Option<String> {
    session.level.with_world(|world| {
        world
            .dynamic_component(entity, component_id)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    })
}

fn start_vampire_game(session: &mut RuntimeDynamicSession) {
    session.tick_frame().unwrap();
    click_vampire_menu_button(session, 640, 360);
    session.tick_frame().unwrap();
    session.tick_frame().unwrap();
    assert_eq!(
        dynamic_component_string(session, 2, "vampire.run_state").as_deref(),
        Some("playing"),
        "vampire test helper should enter gameplay before assertions"
    );
}

fn click_vampire_menu_button(session: &mut RuntimeDynamicSession, width: u32, height: u32) {
    let pointer_x = width as f32 * 0.5;
    let pointer_y = height as f32 * 0.5 + 68.0;
    let moved = session.handle_event(ZrRuntimeEventV1::pointer_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        pointer_x,
        pointer_y,
    ));
    assert!(moved.is_ok(), "{moved:?}");
    let pressed = session.handle_event(ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        pointer_x,
        pointer_y,
    ));
    assert!(pressed.is_ok(), "{pressed:?}");
    let released = session.handle_event(ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        pointer_x,
        pointer_y,
    ));
    assert!(released.is_ok(), "{released:?}");
}

fn world_hud_bar(session: &RuntimeDynamicSession, entity: u64) -> Option<serde_json::Value> {
    dynamic_component_value(session, entity, "render.world_hud_bars")
        .and_then(|value| {
            value
                .get("bars")
                .and_then(serde_json::Value::as_array)
                .cloned()
        })
        .and_then(|bars| bars.first().cloned())
}

fn assert_world_hud_bar_tracks_position(
    bar: &serde_json::Value,
    expected_position: Vec3,
    message: &str,
) {
    let position = bar
        .get("position")
        .and_then(serde_json::Value::as_array)
        .expect("world HUD bar must carry a position");
    let actual = Vec3::new(
        position
            .first()
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN) as f32,
        position
            .get(1)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN) as f32,
        position
            .get(2)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN) as f32,
    );
    let delta = actual - expected_position;
    assert!(
        delta.length() <= 0.001,
        "{message}: actual={actual:?} expected={expected_position:?} bar={bar:?}"
    );
    assert!(
        bar.get("ratio")
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|ratio| (0.0..=1.0).contains(&ratio)),
        "{message}: world HUD bar must carry a normalized ratio, bar={bar:?}"
    );
}

fn animation_state_machine_parameters(
    session: &RuntimeDynamicSession,
    entity: u64,
) -> std::collections::BTreeMap<String, AnimationParameterValue> {
    session
        .level
        .with_world(|world| {
            world
                .animation_state_machine_player(entity)
                .map(|player| player.parameters.clone())
        })
        .unwrap_or_default()
}

fn vampire_actor_node_local_transforms(
    session: &RuntimeDynamicSession,
) -> std::collections::BTreeMap<u64, Transform> {
    session.level.with_world(|world| {
        [202, 203, 204, 205, 206, 207]
            .into_iter()
            .filter_map(|entity| world.find_node(entity).map(|node| (entity, node.transform)))
            .collect()
    })
}

fn entity_position(session: &RuntimeDynamicSession, entity: u64) -> Vec3 {
    session
        .level
        .with_world(|world| world.world_transform(entity).unwrap().translation)
}

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    let delta = actual - expected;
    assert!(
        delta.length() <= 0.001,
        "expected vector close to {expected:?}, got {actual:?}"
    );
}

fn planar_distance(a: Vec3, b: Vec3) -> f32 {
    let delta = a - b;
    Vec3::new(delta.x, 0.0, delta.z).length()
}

fn count_hud_panel_pixels(rgba: &[u8], width: u32, height: u32) -> usize {
    let width = width as usize;
    let height = height as usize;
    let y_start = 16usize.min(height);
    let y_end = 80usize.min(height);
    let x_start = 16usize.min(width);
    let x_end = 260usize.min(width);
    let mut count = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            if pixel[0] <= 70 && pixel[1] <= 90 && pixel[2] <= 105 && pixel[3] >= 180 {
                count += 1;
            }
        }
    }
    count
}

fn count_world_hud_bar_pixels(rgba: &[u8], width: u32, height: u32) -> usize {
    let width = width as usize;
    let height = height as usize;
    let mut count = 0;
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            let red_bar =
                pixel[0] >= 170 && (90..=170).contains(&pixel[1]) && (95..=190).contains(&pixel[2]);
            let green_bar = (110..=200).contains(&pixel[0]) && pixel[1] >= 150 && pixel[2] <= 140;
            let blue_bar = pixel[0] <= 120 && pixel[1] >= 120 && pixel[2] >= 160;
            let purple_slot = pixel[0] >= 120 && pixel[1] <= 150 && pixel[2] >= 130;
            if pixel[3] >= 180 && (red_bar || green_bar || blue_bar || purple_slot) {
                count += 1;
            }
        }
    }
    count
}

fn capture_vampire_frame_for_env(session: &mut RuntimeDynamicSession, env_var: &str) {
    if std::env::var(env_var).is_err() {
        return;
    }
    let frame = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            vampire_capture_viewport_size(),
        ))
        .unwrap();
    let rgba = if frame.rgba.data.is_null() || frame.rgba.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(frame.rgba.data.cast_const(), frame.rgba.len) }
    };
    export_vampire_capture_frame_if_requested(env_var, rgba, frame.width, frame.height);
}

fn export_vampire_capture_frame_if_requested(env_var: &str, rgba: &[u8], width: u32, height: u32) {
    let Ok(path) = std::env::var(env_var) else {
        return;
    };
    let Some(image) = image::RgbaImage::from_raw(width, height, rgba.to_vec()) else {
        panic!("captured vampire frame rgba buffer does not match {width}x{height}");
    };
    image
        .save_with_format(path, image::ImageFormat::Png)
        .expect("failed to export vampire capture frame png");
}

fn vampire_capture_viewport_size() -> zircon_runtime_interface::ZrRuntimeViewportSizeV1 {
    let width = std::env::var("ZR_VAMPIRE_CAPTURE_WIDTH")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(640);
    let height = std::env::var("ZR_VAMPIRE_CAPTURE_HEIGHT")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(360);
    zircon_runtime_interface::ZrRuntimeViewportSizeV1::new(width.max(1), height.max(1))
}

fn vampire_capture_tick_count() -> usize {
    std::env::var("ZR_VAMPIRE_CAPTURE_TICKS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
}

fn vampire_diagnostic_tick_count() -> usize {
    std::env::var("ZR_VAMPIRE_DIAGNOSTIC_TICKS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(12)
}

fn diagnostic_current(
    diagnostics: &crate::core::diagnostics::RuntimeDiagnosticsSnapshot,
    path: &str,
) -> Option<f64> {
    diagnostics
        .store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}

fn diagnostic_series<'a>(
    diagnostics: &'a crate::core::diagnostics::RuntimeDiagnosticsSnapshot,
    path: &str,
) -> Option<&'a crate::core::diagnostics::DiagnosticSeriesSnapshot> {
    diagnostics
        .store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
}

fn small_headless_frame_request() -> ZrRuntimeFrameRequestV1 {
    ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        zircon_runtime_interface::ZrRuntimeViewportSizeV1::new(64, 48),
    )
}

fn summarize_hud_region(rgba: &[u8], width: u32, height: u32) -> String {
    let width = width as usize;
    let height = height as usize;
    let sample_points = [
        (20, 20),
        (80, 24),
        (220, 50),
        (20, height.saturating_sub(24)),
    ];
    let samples = sample_points
        .into_iter()
        .filter_map(|(x, y)| {
            let index = (y * width + x) * 4;
            rgba.get(index..index + 4)
                .map(|pixel| format!("({x},{y})={:?}", pixel))
        })
        .collect::<Vec<_>>();
    let mut min_rgb = [u8::MAX; 3];
    let mut max_rgb = [u8::MIN; 3];
    let mut opaque = 0usize;
    let y_end = 80usize.min(height);
    let x_end = 260usize.min(width);
    for y in 16usize.min(height)..y_end {
        for x in 16usize.min(width)..x_end {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            for channel in 0..3 {
                min_rgb[channel] = min_rgb[channel].min(pixel[channel]);
                max_rgb[channel] = max_rgb[channel].max(pixel[channel]);
            }
            if pixel[3] >= 180 {
                opaque += 1;
            }
        }
    }
    format!(
        "samples=[{}] min_rgb={min_rgb:?} max_rgb={max_rgb:?} opaque={opaque}",
        samples.join(", ")
    )
}

#[test]
fn runtime_session_error_preserves_step_when_inner_error_is_empty() {
    assert_eq!(
        runtime_session_error("load default level", ""),
        "load default level failed without additional diagnostics"
    );
    assert_eq!(
        runtime_session_error("load default level", "scene asset missing"),
        "load default level: scene asset missing"
    );
}
