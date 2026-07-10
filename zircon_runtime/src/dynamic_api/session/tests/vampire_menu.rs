use crate::core::math::Vec3;

use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};
use super::vampire_runtime_support::*;

#[cfg_attr(
    not(feature = "backend-zr-vm"),
    ignore = "requires backend-zr-vm and ZR_VM_RUST_BINDING_LIB_DIR"
)]
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

#[cfg_attr(
    not(feature = "backend-zr-vm"),
    ignore = "requires backend-zr-vm and ZR_VM_RUST_BINDING_LIB_DIR"
)]
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
