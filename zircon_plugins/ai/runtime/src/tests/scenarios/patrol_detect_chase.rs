use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorNodeKind, AiBlackboardSchemaDescriptor, AiDecisionStatus,
    AiManager, AiPerceptionSense,
};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::scene::{ComponentTypeDescriptor, WorldHandle};
use zircon_runtime::core::math::Transform;
use zircon_runtime::plugin::RuntimePluginRegistrationReport;
use zircon_runtime::scene::{
    create_default_level, module_descriptor as scene_module_descriptor, NodeKind, SCENE_MODULE_NAME,
};

use crate::behavior_tree::RuntimeBehaviorIntegrationHost;
use crate::perception::{AiPerceptionChannels, AiPerceptionSource};
use crate::{AiRuntimePlugin, DefaultAiManager};

use self::fixtures::*;

mod fixtures;

const PATROL_TARGET: [f32; 3] = [4.0, 0.0, 0.0];
const CHASE_TARGET: [f32; 3] = [0.0, 0.0, -8.0];

#[test]
fn patrol_detect_chase_scenario() {
    let runtime = zircon_runtime::core::CoreRuntime::new();
    runtime
        .register_module(scene_module_descriptor())
        .expect("register scene module");
    runtime
        .activate_module(SCENE_MODULE_NAME)
        .expect("activate scene module");
    let level = create_default_level(&runtime.handle()).expect("create scene level");
    let plugin = AiRuntimePlugin::new();
    let manager = plugin.manager();
    let mut registration = RuntimePluginRegistrationReport::from_plugin(&plugin);
    assert!(registration.is_success(), "{:?}", registration.diagnostics);

    let world_handle = level.world_handle();
    let (agent, target) = level.with_world_mut(|world| {
        registration
            .extensions
            .apply_to_world(world)
            .expect("apply AI runtime extensions");
        world.register_event::<NavAgentTickReport>();
        world
            .register_component_type(ComponentTypeDescriptor::new(
                NAV_MESH_AGENT_COMPONENT_TYPE,
                "navigation",
                "Nav Mesh Agent",
            ))
            .expect("register neutral navigation agent descriptor");
        let agent = spawn_nav_agent(world);
        let target = world.spawn_node(NodeKind::Empty);
        world
            .update_transform(target, Transform::from_translation(vec3(CHASE_TARGET)))
            .expect("position chase target");
        (agent, target)
    });
    let tree = manager
        .register_behavior_tree(patrol_detect_chase_tree(target))
        .expect("register patrol/detect/chase tree");

    let patrol = level.with_world_mut(|world| {
        let mut host = RuntimeBehaviorIntegrationHost::new(world, None);
        manager
            .tick_agent_with_integration_host(
                tick_request(world_handle, agent, tree, None),
                &mut host,
            )
            .expect("start patrol branch")
    });
    assert_eq!(patrol.status, AiDecisionStatus::Running);
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));
    level.with_world(|world| {
        assert_eq!(nav_target(world, agent), Some(PATROL_TARGET));
    });

    level.with_world_mut(|world| {
        world
            .insert(
                target,
                AiPerceptionSource {
                    channels: AiPerceptionChannels::SIGHT,
                    strength: 1.0,
                },
            )
            .expect("enable chase target sight stimulus");
    });
    tick_level(&runtime, &level);

    let detected = manager
        .perception_snapshot(world_handle, agent)
        .expect("perception system publishes agent snapshot");
    assert!(detected.stimuli.iter().any(|stimulus| {
        stimulus.source == target && stimulus.sense == AiPerceptionSense::Sight
    }));
    level.with_world(|world| {
        assert_eq!(nav_target(world, agent), Some(CHASE_TARGET));
    });

    level.with_world_mut(|world| {
        world.send_event(NavAgentTickReport {
            arrived_agents: vec![(agent, CHASE_TARGET)],
            ..NavAgentTickReport::default()
        });
    });
    tick_level(&runtime, &level);

    let completed = level.with_world_mut(|world| {
        assert_eq!(nav_target(world, agent), Some([0.0, 0.0, 0.0]));
        world.update_events::<AiAgentTickReport>();
        world
            .events::<AiAgentTickReport>()
            .into_iter()
            .flat_map(|events| events.iter())
            .find(|report| report.entity == agent)
            .cloned()
            .expect("behavior system publishes chase completion")
    });
    assert_eq!(completed.status, AiDecisionStatus::Succeeded);
    assert_eq!(completed.active_node.as_deref(), Some("chase_move"));
}

#[test]
fn reactive_perception_aborts_patrol_before_starting_chase() {
    let manager = DefaultAiManager::default();
    let target = 900;
    let tree = manager
        .register_behavior_tree(patrol_detect_chase_tree(target))
        .expect("register patrol/detect/chase tree");
    let world = WorldHandle::new(77);
    let agent = 31;
    let mut host = RecordingIntegrationHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("start patrol branch");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));

    let chase = manager
        .tick_agent_with_integration_host(
            tick_request(
                world,
                agent,
                tree,
                Some(zircon_runtime::core::framework::ai::AiPerceptionSnapshot {
                    agent,
                    stimuli: vec![zircon_runtime::core::framework::ai::AiPerceptionStimulus {
                        source: target,
                        sense: AiPerceptionSense::Sight,
                        position: vec3(CHASE_TARGET),
                        strength: 1.0,
                        age_seconds: 0.0,
                    }],
                }),
            ),
            &mut host,
        )
        .expect("switch to chase branch");

    assert_eq!(chase.active_node.as_deref(), Some("chase_move"));
    assert_eq!(
        host.steps,
        ["move:patrol_move", "abort:patrol_move", "move:chase_move"]
    );
}

#[test]
fn nested_sequence_aborts_patrol_before_starting_chase() {
    let manager = DefaultAiManager::default();
    let target = 901;
    let tree = manager
        .register_behavior_tree(nested_sequence_tree(target))
        .expect("register nested reactive sequence");
    let world = WorldHandle::new(78);
    let agent = 32;
    let mut host = RecordingIntegrationHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("start nested-sequence patrol branch");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));

    let chase = manager
        .tick_agent_with_integration_host(
            tick_request(world, agent, tree, Some(sight_snapshot(agent, target))),
            &mut host,
        )
        .expect("switch nested sequence to chase");

    assert_eq!(chase.active_node.as_deref(), Some("chase_move"));
    assert_eq!(
        host.steps,
        ["move:patrol_move", "abort:patrol_move", "move:chase_move"]
    );
}

#[test]
fn nested_selector_does_not_preempt_from_unrelated_fallback() {
    assert_nested_composite_ignores_unrelated_fallback(AiBehaviorNodeKind::Selector);
}

#[test]
fn nested_parallel_does_not_preempt_from_unrelated_fallback() {
    assert_nested_composite_ignores_unrelated_fallback(AiBehaviorNodeKind::Parallel);
}

#[test]
fn sequence_preceding_none_guard_blocks_reactive_preemption() {
    let manager = DefaultAiManager::default();
    let schema = manager
        .register_blackboard_schema(
            AiBlackboardSchemaDescriptor::new("scenario_guard", "Scenario Guard")
                .with_key("enabled", "bool", true),
        )
        .expect("register scenario guard schema");
    let target = 903;
    let tree = manager
        .register_behavior_tree(sequence_with_preceding_none_guard_tree(target))
        .expect("register sequence with preceding non-reactive guard");
    let world = WorldHandle::new(80);
    let agent = 34;
    let mut host = RecordingIntegrationHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(
            tick_request_with_blackboard(world, agent, tree, schema, false, None),
            &mut host,
        )
        .expect("start patrol after preceding guard fails");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));

    let resumed = manager
        .tick_agent_with_integration_host(
            tick_request_with_blackboard(
                world,
                agent,
                tree,
                schema,
                false,
                Some(sight_snapshot(agent, target)),
            ),
            &mut host,
        )
        .expect("keep patrol while preceding guard remains false");

    assert_eq!(resumed.active_node.as_deref(), Some("patrol_move"));
    assert_eq!(host.steps, ["move:patrol_move", "move:patrol_move"]);
}

#[test]
fn parallel_failure_policy_blocks_reactive_preemption() {
    let manager = DefaultAiManager::default();
    let target = 904;
    let tree = manager
        .register_behavior_tree(parallel_with_stable_failure_tree(target))
        .expect("register policy-determined parallel failure");
    let world = WorldHandle::new(81);
    let agent = 35;
    let mut host = RecordingIntegrationHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("start patrol after parallel failure");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));

    let resumed = manager
        .tick_agent_with_integration_host(
            tick_request(world, agent, tree, Some(sight_snapshot(agent, target))),
            &mut host,
        )
        .expect("keep patrol after parallel remains failed");

    assert_eq!(resumed.active_node.as_deref(), Some("patrol_move"));
    assert_eq!(host.steps, ["move:patrol_move", "move:patrol_move"]);
}

#[test]
fn parallel_reactive_guard_recovers_after_failed_tick() {
    let manager = DefaultAiManager::default();
    let target = 906;
    let tree = manager
        .register_behavior_tree(parallel_recovery_tree(target))
        .expect("register recoverable parallel branch");
    let world = WorldHandle::new(83);
    let agent = 37;
    let mut host = RecordingIntegrationHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("start patrol after all parallel branches fail");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));

    let recovered = manager
        .tick_agent_with_integration_host(
            tick_request(world, agent, tree, Some(sight_snapshot(agent, target))),
            &mut host,
        )
        .expect("reactive parallel branch recovers");

    assert_eq!(recovered.status, AiDecisionStatus::Running);
    assert_eq!(recovered.active_node.as_deref(), Some("chase_move"));
    assert_eq!(
        host.steps,
        ["move:patrol_move", "abort:patrol_move", "move:chase_move"]
    );
}

#[test]
fn parallel_known_false_reactive_sibling_blocks_preemption() {
    let manager = DefaultAiManager::default();
    let visible_target = 907;
    let hidden_target = 908;
    let tree = manager
        .register_behavior_tree(parallel_two_reactive_guards_tree(
            visible_target,
            hidden_target,
        ))
        .expect("register parallel branch with two reactive guards");
    let world = WorldHandle::new(84);
    let agent = 38;
    let mut host = RecordingIntegrationHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("start patrol after both reactive guards fail");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));

    let resumed = manager
        .tick_agent_with_integration_host(
            tick_request(
                world,
                agent,
                tree,
                Some(sight_snapshot(agent, visible_target)),
            ),
            &mut host,
        )
        .expect("keep patrol while one failure still determines Parallel result");

    assert_eq!(resumed.active_node.as_deref(), Some("patrol_move"));
    assert_eq!(host.steps, ["move:patrol_move", "move:patrol_move"]);
}

#[test]
fn random_selector_ignores_unselected_reactive_guard() {
    let manager = DefaultAiManager::default();
    let target = 905;
    let tree = manager
        .register_behavior_tree(random_selector_with_unselected_guard_tree(target))
        .expect("register random selector with an unselected reactive guard");
    let world = WorldHandle::new(82);
    let agent = 36;
    let mut host = RecordingIntegrationHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("start patrol after selected branch fails");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));

    let resumed = manager
        .tick_agent_with_integration_host(
            tick_request(world, agent, tree, Some(sight_snapshot(agent, target))),
            &mut host,
        )
        .expect("keep patrol while reactive guard stays unselected");

    assert_eq!(resumed.active_node.as_deref(), Some("patrol_move"));
    assert_eq!(host.steps, ["move:patrol_move", "move:patrol_move"]);
}

fn assert_nested_composite_ignores_unrelated_fallback(kind: AiBehaviorNodeKind) {
    let manager = DefaultAiManager::default();
    let target = 902;
    let tree = manager
        .register_behavior_tree(nested_composite_tree(target, kind))
        .expect("register nested reactive composite");
    let world = WorldHandle::new(79);
    let agent = 33;
    let mut host = ChangingFallbackHost::default();

    let patrol = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("start patrol after nested composite fails");
    assert_eq!(patrol.active_node.as_deref(), Some("patrol_move"));
    assert_eq!(host.fallback_calls, 1);

    let resumed = manager
        .tick_agent_with_integration_host(tick_request(world, agent, tree, None), &mut host)
        .expect("resume patrol while nested reactive guard remains false");

    assert_eq!(resumed.active_node.as_deref(), Some("patrol_move"));
    assert_eq!(host.fallback_calls, 1);
    assert_eq!(
        host.steps,
        [
            "move:unrelated_fallback",
            "move:patrol_move",
            "move:patrol_move"
        ]
    );
}
