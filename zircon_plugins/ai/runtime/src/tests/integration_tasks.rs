use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
    AiDecisionStatus, AiManager,
};
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::framework::script::{
    ScriptBehaviorBridge, ScriptBehaviorCallbackRef, ScriptHostError, ScriptHostValue,
};
use zircon_runtime::core::math::Vec3;
use zircon_runtime::core::resource::{
    AnimationGraphMarker, AnimationStateMachineMarker, ResourceHandle, ResourceId,
};
use zircon_runtime::plugin::{BridgeImport, RuntimeExtensionRegistry};
use zircon_runtime::scene::components::{
    AnimationGraphPlayerComponent, AnimationStateMachinePlayerComponent,
};
use zircon_runtime::scene::{NodeKind, World};

use crate::behavior_tree::RuntimeBehaviorIntegrationHost;
use crate::DefaultAiManager;

#[test]
fn move_to_maps_arrival_and_failure_to_node_result() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "move_to_contract",
        "move_to",
        [("target", Vec3::new(4.0, 0.0, 2.0).into())],
    );
    let mut world = navigation_world();
    let arrived = spawn_nav_agent(&mut world);

    let running = tick_with_world_host(&manager, tree, arrived, &mut world, 0.1);
    assert_eq!(running, AiDecisionStatus::Running);
    assert_eq!(nav_target(&world, arrived), Some([4.0, 0.0, 2.0]));

    publish_navigation_arrival(&mut world, arrived, [4.0, 0.0, 2.0]);
    let succeeded = tick_with_world_host(&manager, tree, arrived, &mut world, 0.1);
    assert_eq!(succeeded, AiDecisionStatus::Succeeded);
    assert_eq!(nav_target(&world, arrived), Some([0.0, 0.0, 0.0]));

    let failed = spawn_nav_agent(&mut world);
    assert_eq!(
        tick_with_world_host(&manager, tree, failed, &mut world, 0.1),
        AiDecisionStatus::Running
    );
    publish_navigation_no_path(&mut world, failed, [4.0, 0.0, 2.0]);
    let failure = tick_with_world_host(&manager, tree, failed, &mut world, 0.1);
    assert_eq!(failure, AiDecisionStatus::Failed);
    assert_eq!(nav_target(&world, failed), Some([0.0, 0.0, 0.0]));
}

#[test]
fn move_to_ignores_stale_same_target_outcome_on_task_start() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "move_to_stale_outcome",
        "move_to",
        [("target", Vec3::new(2.0, 0.0, 0.0).into())],
    );
    let mut world = navigation_world();
    let entity = spawn_nav_agent(&mut world);
    publish_navigation_no_path(&mut world, entity, [2.0, 0.0, 0.0]);

    let first = tick_with_world_host(&manager, tree, entity, &mut world, 0.1);

    assert_eq!(first, AiDecisionStatus::Running);
    assert_eq!(nav_target(&world, entity), Some([2.0, 0.0, 0.0]));
}

#[test]
fn move_to_blocks_when_navigation_runtime_is_unavailable() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "move_to_without_navigation",
        "move_to",
        [("target", Vec3::new(1.0, 0.0, 0.0).into())],
    );
    let mut world = World::new();
    let entity = spawn_nav_agent(&mut world);

    let status = tick_with_world_host(&manager, tree, entity, &mut world, 0.1);

    assert_eq!(status, AiDecisionStatus::Blocked);
    assert_eq!(nav_target(&world, entity), None);
}

#[test]
fn move_to_abort_clears_pending_navigation_target() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "move_to_abort",
        "move_to",
        [("target", Vec3::new(9.0, 0.0, 0.0).into())],
    );
    let mut world = navigation_world();
    let entity = spawn_nav_agent(&mut world);
    assert_eq!(
        tick_with_world_host(&manager, tree, entity, &mut world, 0.1),
        AiDecisionStatus::Running
    );

    let mut host = RuntimeBehaviorIntegrationHost::new(&mut world, None);
    let report = manager
        .tick_agent_with_integration_host(
            AiAgentTickRequest {
                world: WorldHandle::new(1),
                entity,
                behavior_tree: None,
                blackboard_schema: None,
                delta_seconds: 0.1,
                blackboard: Vec::new(),
                perception: None,
            },
            &mut host,
        )
        .expect("abort active MoveTo");
    drop(host);

    assert_eq!(report.status, AiDecisionStatus::Idle);
    assert_eq!(nav_target(&world, entity), Some([0.0, 0.0, 0.0]));
}

#[test]
fn play_animation_sets_parameter_and_completes() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "play_animation_trigger",
        "play_animation",
        [("trigger", "attack".into())],
    );
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_animation_state_machine_player(
            entity,
            Some(AnimationStateMachinePlayerComponent {
                state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(
                    ResourceId::from_stable_label("res://animation/combat.state_machine.zranim"),
                ),
                parameters: BTreeMap::new(),
                active_state: None,
                playing: false,
            }),
        )
        .expect("state machine player");

    let status = tick_with_world_host(&manager, tree, entity, &mut world, 0.1);

    assert_eq!(status, AiDecisionStatus::Succeeded);
    let player = world
        .animation_state_machine_player(entity)
        .expect("updated state machine player");
    assert_eq!(
        player.parameters.get("attack"),
        Some(&AnimationParameterValue::Trigger)
    );
    assert!(player.playing);
}

#[test]
fn play_animation_writes_typed_graph_parameter() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "play_animation_speed",
        "play_animation",
        [("parameter", "speed".into()), ("value", 2.5_f32.into())],
    );
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .set_animation_graph_player(
            entity,
            Some(AnimationGraphPlayerComponent {
                graph: ResourceHandle::<AnimationGraphMarker>::new(ResourceId::from_stable_label(
                    "res://animation/locomotion.graph.zranim",
                )),
                parameters: BTreeMap::new(),
                playing: false,
            }),
        )
        .expect("graph player");

    let status = tick_with_world_host(&manager, tree, entity, &mut world, 0.1);

    assert_eq!(status, AiDecisionStatus::Succeeded);
    let player = world
        .animation_graph_player(entity)
        .expect("updated graph player");
    assert_eq!(
        player.parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(2.5))
    );
    assert!(player.playing);
}

#[test]
fn play_animation_parameter_requires_typed_value() {
    let manager = DefaultAiManager::default();
    let node = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Integration")
        .with_implementation("play_animation")
        .with_parameter("parameter", "speed");

    let error = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("missing_animation_value", "Integration", "root")
                .with_node(node),
        )
        .expect_err("parameter writes require an explicit typed value");

    assert!(matches!(
        error,
        zircon_runtime::core::framework::ai::AiManagerError::InvalidBehaviorNodeParameter {
            key,
            actual: "missing",
            ..
        } if key == "value"
    ));
}

#[test]
fn integration_task_without_host_is_blocked() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "move_to_without_host",
        "move_to",
        [("target", Vec3::new(1.0, 0.0, 0.0).into())],
    );

    let report = manager
        .tick_agent(tick_request(tree, 41, 0.1))
        .expect("public no-host tick returns a typed result");

    assert_eq!(report.status, AiDecisionStatus::Blocked);
    assert!(report
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("integration host is unavailable")));
}

#[test]
fn script_task_round_trips_through_mock_vm() {
    let manager = DefaultAiManager::default();
    let tree = register_integration_tree(
        &manager,
        "script_task_callback",
        "script_task",
        [("callback", "combat_plugin::combat.attack".into())],
    );
    let mut world = World::new();
    let entity = world.spawn_node(NodeKind::Empty);
    let (script_import, bridge) = bound_script_behavior_bridge();
    let mut host = RuntimeBehaviorIntegrationHost::new(&mut world, Some(script_import));

    let report = manager
        .tick_agent_with_integration_host(tick_request(tree, entity, 0.25), &mut host)
        .expect("script task tick");
    drop(host);

    assert_eq!(report.status, AiDecisionStatus::Succeeded);
    let calls = bridge
        .calls
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0.package_id(), "combat_plugin");
    assert_eq!(calls[0].0.node_id(), "combat.attack");
    assert_eq!(
        calls[0].1,
        vec![
            ScriptHostValue::HostHandle(entity),
            ScriptHostValue::Float(0.25),
        ]
    );
}

#[derive(Default)]
struct MockScriptBehaviorBridge {
    calls: Mutex<Vec<(ScriptBehaviorCallbackRef, Vec<ScriptHostValue>)>>,
}

impl ScriptBehaviorBridge for MockScriptBehaviorBridge {
    fn invoke(
        &self,
        callback: &ScriptBehaviorCallbackRef,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, ScriptHostError> {
        self.calls
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((callback.clone(), arguments.to_vec()));
        Ok(Some(ScriptHostValue::Bool(true)))
    }
}

fn bound_script_behavior_bridge() -> (
    BridgeImport<dyn ScriptBehaviorBridge>,
    Arc<MockScriptBehaviorBridge>,
) {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("test.script.runtime")
        .unwrap();
    let provider = Arc::new(MockScriptBehaviorBridge::default());
    let exported: Arc<dyn ScriptBehaviorBridge> = provider.clone();
    registry
        .export_interface::<dyn ScriptBehaviorBridge>(owner, exported)
        .unwrap();
    let imported = registry
        .import_interface::<dyn ScriptBehaviorBridge>(owner)
        .unwrap();
    registry.finalize();
    (imported, provider)
}

fn navigation_world() -> World {
    let mut world = World::new();
    world.register_event::<NavAgentTickReport>();
    world
}

fn spawn_nav_agent(world: &mut World) -> u64 {
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            r#"{"stopping_distance":0.25,"destination":null}"#
                .parse()
                .expect("nav agent value"),
        )
        .expect("nav agent component");
    entity
}

fn nav_target(world: &World, entity: u64) -> Option<[f32; 3]> {
    let values = world
        .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
        .expect("nav agent component")
        .get("destination")?
        .as_array()?;
    Some([
        values.first()?.as_f64()? as f32,
        values.get(1)?.as_f64()? as f32,
        values.get(2)?.as_f64()? as f32,
    ])
}

fn publish_navigation_arrival(world: &mut World, entity: u64, destination: [f32; 3]) {
    world.send_event(NavAgentTickReport {
        arrived_agents: vec![(entity, destination)],
        ..NavAgentTickReport::default()
    });
    world.update_events::<NavAgentTickReport>();
}

fn publish_navigation_no_path(world: &mut World, entity: u64, destination: [f32; 3]) {
    world.send_event(NavAgentTickReport {
        no_path_agents: vec![(entity, destination)],
        ..NavAgentTickReport::default()
    });
    world.update_events::<NavAgentTickReport>();
}

fn tick_with_world_host(
    manager: &DefaultAiManager,
    tree: zircon_runtime::core::framework::ai::AiBehaviorTreeId,
    entity: u64,
    world: &mut World,
    delta_seconds: f32,
) -> AiDecisionStatus {
    let mut host = RuntimeBehaviorIntegrationHost::new(world, None);
    manager
        .tick_agent_with_integration_host(tick_request(tree, entity, delta_seconds), &mut host)
        .expect("integration task tick")
        .status
}

fn tick_request(
    tree: zircon_runtime::core::framework::ai::AiBehaviorTreeId,
    entity: u64,
    delta_seconds: f32,
) -> AiAgentTickRequest {
    AiAgentTickRequest {
        world: WorldHandle::new(1),
        entity,
        behavior_tree: Some(tree),
        blackboard_schema: None,
        delta_seconds,
        blackboard: Vec::new(),
        perception: None,
    }
}

fn register_integration_tree<const N: usize>(
    manager: &DefaultAiManager,
    tree_id: &str,
    implementation: &str,
    parameters: [(
        &str,
        zircon_runtime::core::framework::ai::AiBehaviorNodeParameterValue,
    ); N],
) -> zircon_runtime::core::framework::ai::AiBehaviorTreeId {
    let mut node = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Integration")
        .with_implementation(implementation);
    for (key, value) in parameters {
        node = node.with_parameter(key, value);
    }
    manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new(tree_id, "Integration", "root").with_node(node),
        )
        .expect("integration tree")
}
