use std::time::Duration;

use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind,
    AiBehaviorTreeDescriptor, AiBlackboardEntry, AiBlackboardValue, AiDecisionStatus,
    AiPerceptionSense,
};
use zircon_runtime::core::framework::navigation::NAV_MESH_AGENT_COMPONENT_TYPE;
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::{NodeKind, World};

use crate::behavior_tree::{
    BehaviorIntegrationHost, BehaviorIntegrationTaskContext, IntegrationTaskResult,
};
use crate::perception::{AiPerceptionChannels, AiPerceptionReceiver};

use super::{CHASE_TARGET, PATROL_TARGET};

#[derive(Default)]
pub(super) struct RecordingIntegrationHost {
    pub(super) steps: Vec<String>,
}

#[derive(Default)]
pub(super) struct ChangingFallbackHost {
    pub(super) steps: Vec<String>,
    pub(super) fallback_calls: usize,
}

impl BehaviorIntegrationHost for ChangingFallbackHost {
    fn move_to(&mut self, context: &BehaviorIntegrationTaskContext<'_>) -> IntegrationTaskResult {
        self.steps.push(format!("move:{}", context.node_id));
        let status = if context.node_id == "unrelated_fallback" {
            self.fallback_calls += 1;
            if self.fallback_calls == 1 {
                AiDecisionStatus::Failed
            } else {
                AiDecisionStatus::Running
            }
        } else {
            AiDecisionStatus::Running
        };
        IntegrationTaskResult {
            status,
            diagnostic: None,
        }
    }

    fn play_animation(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult {
        unsupported_in_scenario(context)
    }

    fn script_task(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult {
        unsupported_in_scenario(context)
    }

    fn abort(&mut self, context: &BehaviorIntegrationTaskContext<'_>) {
        self.steps.push(format!("abort:{}", context.node_id));
    }
}

impl BehaviorIntegrationHost for RecordingIntegrationHost {
    fn move_to(&mut self, context: &BehaviorIntegrationTaskContext<'_>) -> IntegrationTaskResult {
        self.steps.push(format!("move:{}", context.node_id));
        IntegrationTaskResult {
            status: AiDecisionStatus::Running,
            diagnostic: None,
        }
    }

    fn play_animation(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult {
        unsupported_in_scenario(context)
    }

    fn script_task(
        &mut self,
        context: &BehaviorIntegrationTaskContext<'_>,
    ) -> IntegrationTaskResult {
        unsupported_in_scenario(context)
    }

    fn abort(&mut self, context: &BehaviorIntegrationTaskContext<'_>) {
        self.steps.push(format!("abort:{}", context.node_id));
    }
}

fn unsupported_in_scenario(context: &BehaviorIntegrationTaskContext<'_>) -> IntegrationTaskResult {
    IntegrationTaskResult {
        status: AiDecisionStatus::Blocked,
        diagnostic: Some(format!(
            "scenario does not provide integration task `{}`",
            context.node_id
        )),
    }
}

pub(super) fn patrol_detect_chase_tree(target: u64) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("patrol_detect_chase", "Patrol Detect Chase", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("target_visible")
                .with_child("patrol_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "target_visible",
                AiBehaviorNodeKind::Decorator,
                "Target Visible",
            )
            .with_parameter("perception_sense", "sight")
            .with_parameter("perception_source", target)
            .with_child("chase_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("chase_move", AiBehaviorNodeKind::Task, "Chase Target")
                .with_implementation("move_to")
                .with_parameter("target", vec3(CHASE_TARGET)),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("patrol_move", AiBehaviorNodeKind::Task, "Patrol Route")
                .with_implementation("move_to")
                .with_parameter("target", vec3(PATROL_TARGET)),
        )
}

pub(super) fn nested_sequence_tree(target: u64) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("nested_sequence_chase", "Nested Sequence Chase", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("reactive_sequence")
                .with_child("patrol_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "reactive_sequence",
                AiBehaviorNodeKind::Sequence,
                "Reactive Sequence",
            )
            .with_child("target_visible")
            .with_child("chase_move"),
        )
        .with_node(perception_guard(target))
        .with_node(succeeded_guard_task())
        .with_node(chase_move_node())
        .with_node(patrol_move_node())
}

pub(super) fn nested_composite_tree(
    target: u64,
    kind: AiBehaviorNodeKind,
) -> AiBehaviorTreeDescriptor {
    let tree_id = match &kind {
        AiBehaviorNodeKind::Selector => "nested_selector_fallback",
        AiBehaviorNodeKind::Parallel => "nested_parallel_fallback",
        _ => unreachable!("test only covers selector and parallel"),
    };
    let is_parallel = kind == AiBehaviorNodeKind::Parallel;
    let mut nested = AiBehaviorNodeDescriptor::new("nested", kind, "Nested Reactive Composite")
        .with_child("target_visible")
        .with_child("unrelated_fallback");
    if is_parallel {
        nested = nested
            .with_parameter("success_policy", "any")
            .with_parameter("failure_policy", "all");
    }
    AiBehaviorTreeDescriptor::new(tree_id, "Nested Composite Fallback", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("nested")
                .with_child("patrol_move"),
        )
        .with_node(nested)
        .with_node(perception_guard(target))
        .with_node(succeeded_guard_task())
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "unrelated_fallback",
                AiBehaviorNodeKind::Task,
                "Unrelated Fallback",
            )
            .with_implementation("move_to")
            .with_parameter("target", vec3([2.0, 0.0, 0.0])),
        )
        .with_node(patrol_move_node())
}

pub(super) fn sequence_with_preceding_none_guard_tree(target: u64) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("guarded_sequence", "Guarded Sequence", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("guarded_sequence")
                .with_child("patrol_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "guarded_sequence",
                AiBehaviorNodeKind::Sequence,
                "Guarded Sequence",
            )
            .with_child("enabled")
            .with_child("target_visible"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("enabled", AiBehaviorNodeKind::Decorator, "Enabled")
                .with_parameter("blackboard_key", "enabled")
                .with_parameter("equals_bool", true)
                .with_abort_policy(AiBehaviorAbortPolicy::None)
                .with_child("enabled_ready"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "enabled_ready",
                AiBehaviorNodeKind::Task,
                "Enabled Ready",
            )
            .with_parameter("result", "succeeded"),
        )
        .with_node(perception_guard(target))
        .with_node(succeeded_guard_task())
        .with_node(patrol_move_node())
}

pub(super) fn parallel_with_stable_failure_tree(target: u64) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("stable_parallel_failure", "Stable Parallel Failure", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("parallel_gate")
                .with_child("patrol_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "parallel_gate",
                AiBehaviorNodeKind::Parallel,
                "Parallel Gate",
            )
            .with_parameter("success_policy", "all")
            .with_parameter("failure_policy", "any")
            .with_child("stable_failure")
            .with_child("target_visible"),
        )
        .with_node(failed_task("stable_failure"))
        .with_node(perception_guard(target))
        .with_node(succeeded_guard_task())
        .with_node(patrol_move_node())
}

pub(super) fn parallel_recovery_tree(target: u64) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("parallel_recovery", "Parallel Recovery", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("parallel_gate")
                .with_child("patrol_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "parallel_gate",
                AiBehaviorNodeKind::Parallel,
                "Parallel Gate",
            )
            .with_parameter("success_policy", "any")
            .with_parameter("failure_policy", "all")
            .with_child("stable_failure")
            .with_child("target_visible"),
        )
        .with_node(failed_task("stable_failure"))
        .with_node(perception_move_guard(target))
        .with_node(chase_move_node())
        .with_node(patrol_move_node())
}

pub(super) fn parallel_two_reactive_guards_tree(
    visible_target: u64,
    hidden_target: u64,
) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("parallel_two_guards", "Parallel Two Guards", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("parallel_gate")
                .with_child("patrol_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "parallel_gate",
                AiBehaviorNodeKind::Parallel,
                "Parallel Gate",
            )
            .with_parameter("success_policy", "all")
            .with_parameter("failure_policy", "any")
            .with_child("visible_guard")
            .with_child("hidden_guard"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "visible_guard",
                AiBehaviorNodeKind::Decorator,
                "Visible Guard",
            )
            .with_parameter("perception_sense", "sight")
            .with_parameter("perception_source", visible_target)
            .with_child("chase_move"),
        )
        .with_node(chase_move_node())
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "hidden_guard",
                AiBehaviorNodeKind::Decorator,
                "Hidden Guard",
            )
            .with_parameter("perception_sense", "sight")
            .with_parameter("perception_source", hidden_target)
            .with_child("hidden_ready"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("hidden_ready", AiBehaviorNodeKind::Task, "Hidden Ready")
                .with_parameter("result", "succeeded"),
        )
        .with_node(patrol_move_node())
}

pub(super) fn random_selector_with_unselected_guard_tree(target: u64) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("unselected_random_guard", "Unselected Random Guard", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("random_gate")
                .with_child("patrol_move"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new(
                "random_gate",
                AiBehaviorNodeKind::Parallel,
                "Random Gate",
            )
            .with_implementation("random_selector")
            .with_parameter("weight.stable_failure", 1.0_f32)
            .with_parameter("weight.target_visible", 0.0_f32)
            .with_child("stable_failure")
            .with_child("target_visible"),
        )
        .with_node(failed_task("stable_failure"))
        .with_node(perception_guard(target))
        .with_node(succeeded_guard_task())
        .with_node(patrol_move_node())
}

fn perception_guard(target: u64) -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new(
        "target_visible",
        AiBehaviorNodeKind::Decorator,
        "Target Visible",
    )
    .with_parameter("perception_sense", "sight")
    .with_parameter("perception_source", target)
    .with_child("guard_ready")
}

fn perception_move_guard(target: u64) -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new(
        "target_visible",
        AiBehaviorNodeKind::Decorator,
        "Target Visible",
    )
    .with_parameter("perception_sense", "sight")
    .with_parameter("perception_source", target)
    .with_child("chase_move")
}

fn succeeded_guard_task() -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new("guard_ready", AiBehaviorNodeKind::Task, "Guard Ready")
        .with_parameter("result", "succeeded")
}

fn failed_task(id: &str) -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new(id, AiBehaviorNodeKind::Task, "Stable Failure")
        .with_parameter("result", "failed")
}

fn chase_move_node() -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new("chase_move", AiBehaviorNodeKind::Task, "Chase Target")
        .with_implementation("move_to")
        .with_parameter("target", vec3(CHASE_TARGET))
}

fn patrol_move_node() -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new("patrol_move", AiBehaviorNodeKind::Task, "Patrol Route")
        .with_implementation("move_to")
        .with_parameter("target", vec3(PATROL_TARGET))
}

pub(super) fn sight_snapshot(
    agent: u64,
    target: u64,
) -> zircon_runtime::core::framework::ai::AiPerceptionSnapshot {
    zircon_runtime::core::framework::ai::AiPerceptionSnapshot {
        agent,
        stimuli: vec![zircon_runtime::core::framework::ai::AiPerceptionStimulus {
            source: target,
            sense: AiPerceptionSense::Sight,
            position: vec3(CHASE_TARGET),
            strength: 1.0,
            age_seconds: 0.0,
        }],
    }
}

pub(super) fn spawn_nav_agent(world: &mut World) -> u64 {
    let agent = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(agent, Transform::from_translation(Vec3::ZERO))
        .expect("position AI agent");
    world
        .insert(
            agent,
            AiPerceptionReceiver {
                sight_fov_degrees: 90.0,
                sight_range: 20.0,
                hearing_radius: 20.0,
                forget_seconds: 1.0,
            },
        )
        .expect("attach perception receiver");
    world
        .set_dynamic_component(
            agent,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            r#"{"stopping_distance":0.25,"destination":null}"#
                .parse()
                .expect("nav agent component value"),
        )
        .expect("attach navigation agent");
    agent
}

pub(super) fn nav_target(world: &World, entity: u64) -> Option<[f32; 3]> {
    let values = world
        .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)?
        .get("destination")?
        .as_array()?;
    Some([
        values.first()?.as_f64()? as f32,
        values.get(1)?.as_f64()? as f32,
        values.get(2)?.as_f64()? as f32,
    ])
}

pub(super) fn tick_request(
    world: WorldHandle,
    entity: u64,
    tree: zircon_runtime::core::framework::ai::AiBehaviorTreeId,
    perception: Option<zircon_runtime::core::framework::ai::AiPerceptionSnapshot>,
) -> AiAgentTickRequest {
    AiAgentTickRequest {
        world,
        entity,
        behavior_tree: Some(tree),
        blackboard_schema: None,
        delta_seconds: 1.0 / 60.0,
        blackboard: Vec::new(),
        perception,
    }
}

pub(super) fn tick_request_with_blackboard(
    world: WorldHandle,
    entity: u64,
    tree: zircon_runtime::core::framework::ai::AiBehaviorTreeId,
    schema: zircon_runtime::core::framework::ai::AiBlackboardSchemaId,
    enabled: bool,
    perception: Option<zircon_runtime::core::framework::ai::AiPerceptionSnapshot>,
) -> AiAgentTickRequest {
    AiAgentTickRequest {
        world,
        entity,
        behavior_tree: Some(tree),
        blackboard_schema: Some(schema),
        delta_seconds: 1.0 / 60.0,
        blackboard: vec![AiBlackboardEntry::new(
            "enabled",
            AiBlackboardValue::Bool(enabled),
        )],
        perception,
    }
}

pub(super) fn tick_level(
    runtime: &zircon_runtime::core::CoreRuntime,
    level: &zircon_runtime::scene::LevelSystem,
) {
    let advance = runtime.advance_time_by(Duration::from_millis(16), 8);
    level
        .tick(&runtime.handle(), advance)
        .expect("tick patrol/detect/chase scene");
}

pub(super) fn vec3(value: [f32; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
}
