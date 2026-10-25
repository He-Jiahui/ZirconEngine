//! Runtime-owned JSON state must coexist with a nonempty plugin descriptor catalog.

use serde_json::json;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::scene::{NodeKind, SceneError, World};

const PLUGIN_TYPE: &str = "test.gameplay.Tag";
const GAMEPLAY_TYPES: [&str; 5] = [
    "gameplay.hud_text",
    "gameplay.menu_state",
    "gameplay.control_state",
    "gameplay.particle_sprites",
    "gameplay.world_hud_bars",
];

fn install_plugin_descriptor(world: &mut World) {
    world
        .register_component_type(ComponentTypeDescriptor::new(
            PLUGIN_TYPE,
            "test.gameplay",
            "Tag",
        ))
        .unwrap();
    assert!(world.component_type_descriptor(PLUGIN_TYPE).is_some());
    assert!(world
        .type_registry()
        .registration(PLUGIN_TYPE)
        .unwrap()
        .is_plugin_owned());
}

#[test]
fn runtime_owned_hud_text_remains_writable_with_a_plugin_descriptor() {
    let mut world = World::empty();
    install_plugin_descriptor(&mut world);
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    let text = json!("Score: 12");
    assert!(world
        .set_dynamic_component(entity, "gameplay.hud_text", text.clone())
        .expect("set_hud_text's runtime-owned component must pass the type gate"));
    assert_eq!(
        world.dynamic_component(entity, "gameplay.hud_text"),
        Some(&text)
    );
}

#[test]
fn runtime_owned_menu_and_control_state_remain_writable_with_a_plugin_descriptor() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    // An authored menu can already exist when the plugin catalog becomes nonempty.
    world
        .set_dynamic_component(entity, "gameplay.menu_state", json!({"state": "start"}))
        .unwrap();
    install_plugin_descriptor(&mut world);
    for (component_id, value) in [
        ("gameplay.control_state", json!("start_game")),
        ("gameplay.menu_state", json!({"state": "game_over"})),
    ] {
        assert!(world
            .set_dynamic_component(entity, component_id, value.clone())
            .expect(
                "menu state and the menu action's control-state write must pass the type gate"
            ));
        assert_eq!(world.dynamic_component(entity, component_id), Some(&value));
    }
}

#[test]
fn runtime_owned_gameplay_ids_are_hidden_non_plugin_components() {
    let mut world = World::empty();
    install_plugin_descriptor(&mut world);
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    for component_id in GAMEPLAY_TYPES {
        let registration = world
            .type_registry()
            .registration(component_id)
            .expect("each runtime-owned gameplay id must have a built-in registration");
        assert!(registration.is_component());
        assert!(!registration.is_plugin_owned());
        assert!(!registration.editor_visible);
        assert!(world.component_type_descriptor(component_id).is_none());
        assert!(world
            .set_dynamic_component(entity, component_id, json!({}))
            .unwrap());
    }
}

#[test]
fn runtime_owned_registration_preserves_unknown_component_rejection() {
    let mut world = World::empty();
    install_plugin_descriptor(&mut world);
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    assert!(world
        .set_dynamic_component(entity, PLUGIN_TYPE, json!({}))
        .unwrap());
    // Ad-hoc project ids need Claude's policy decision; do not grant a namespace exemption.
    for component_id in [
        "unknown.Component",
        "gameplay.custom_state",
        "orb.run_state",
    ] {
        assert!(matches!(
            world.set_dynamic_component(entity, component_id, json!({})),
            Err(SceneError::UnregisteredDynamicComponentType { component_id: rejected })
                if rejected == component_id
        ));
        assert!(world.dynamic_component(entity, component_id).is_none());
    }
}

#[cfg(all(feature = "script", feature = "test-support"))]
#[test]
fn runtime_owned_hud_and_menu_writes_succeed_through_gameplay_host_exports() {
    use zircon_runtime::core::framework::script::ScriptHostValue;
    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::scene::DefaultLevelManager;
    use zircon_runtime::script::{
        register_gameplay_host_module, with_script_runtime_test_context, CapabilitySet,
        HostExportRegistry, HostRegistry, ScriptRuntimeTestContext,
    };

    let core = CoreRuntime::new();
    let level = DefaultLevelManager::default().create_default_level();
    let entity = level.with_world_mut(|world| {
        install_plugin_descriptor(world);
        world.spawn_node(NodeKind::Empty).unwrap()
    });
    let exports = HostExportRegistry::new(HostRegistry::default());
    register_gameplay_host_module(&exports).unwrap();
    let capabilities = CapabilitySet::default().with("gameplay.entity");
    with_script_runtime_test_context(
        ScriptRuntimeTestContext::new(core.weak(), level.clone(), entity, 0.016),
        || {
            for (function, arguments) in [
                (
                    "set_hud_text",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("Score: 12".into()),
                    ],
                ),
                (
                    "set_component_json",
                    vec![
                        ScriptHostValue::Int(entity as i64),
                        ScriptHostValue::String("gameplay.menu_state".into()),
                        ScriptHostValue::String(r#"{"state":"start"}"#.into()),
                    ],
                ),
            ] {
                assert_eq!(
                    exports
                        .call_with_capabilities(
                            "zr.zircon.gameplay",
                            function,
                            arguments,
                            &capabilities,
                        )
                        .expect(
                            "runtime-owned writes must succeed through the real host callbacks"
                        ),
                    ScriptHostValue::Bool(true)
                );
            }
        },
    );
    level.with_world(|world| {
        assert_eq!(
            world.dynamic_component(entity, "gameplay.hud_text"),
            Some(&json!("Score: 12"))
        );
        assert_eq!(
            world.dynamic_component(entity, "gameplay.menu_state"),
            Some(&json!({"state": "start"}))
        );
    });
}

#[cfg(feature = "dynamic-api")]
#[path = "runtime_gameplay_component_ids/menu_session.rs"]
mod menu_session;
