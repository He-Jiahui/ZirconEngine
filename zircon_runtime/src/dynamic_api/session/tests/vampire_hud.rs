use crate::core::diagnostics::collect_runtime_diagnostics;
use crate::core::math::Vec3;
use zircon_runtime_interface::{
    ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};
use super::vampire_runtime_support::*;

#[ignore = "real ZrVM coverage moved to the zr_vm_language plugin owner"]
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

#[ignore = "real ZrVM coverage moved to the zr_vm_language plugin owner"]
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

#[ignore = "real ZrVM coverage moved to the zr_vm_language plugin owner"]
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
        render_stats.last_ui_command_count,
        0,
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
        "captured vampire frame should contain colored in-scene health-bar pixels, found {world_hud_bar_pixels}; hud_region={hud_pixel_summary}; capture_report={:?}; graph_passes={:?}; executor_ids={:?}; particle_passes={} particle_alive={} mesh_draws={}",
        render_stats.last_capture_report,
        render_stats.last_graph_executed_passes,
        render_stats.last_graph_executed_executor_ids,
        render_stats.last_particle_graph_executed_pass_count,
        render_stats.last_particle_gpu_alive_count,
        render_stats.last_mesh_draw_count,
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
