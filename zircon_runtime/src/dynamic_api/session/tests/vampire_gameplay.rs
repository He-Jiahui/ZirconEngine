use std::path::Path;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::Vec3;
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
};

use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};
use super::vampire_runtime_support::*;

#[cfg_attr(
    not(feature = "zr-vm-real-backend"),
    ignore = "requires zr-vm-real-backend and ZR_VM_RUST_BINDING_LIB_DIR"
)]
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
        particles
            .as_array()
            .is_some_and(|sprites| !sprites.is_empty())
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
