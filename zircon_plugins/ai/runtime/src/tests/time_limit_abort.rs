use std::cell::RefCell;

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor, AiBehaviorTreeId,
    AiDecisionStatus, AiManager,
};
use zircon_runtime::core::framework::navigation::NAV_MESH_AGENT_COMPONENT_TYPE;
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::plugin::PluginModuleId;

use crate::behavior_tree::{
    BehaviorNodeCatalog, BehaviorNodeCategory::Task, BehaviorNodeDescriptor, BehaviorNodeRuntime,
    BehaviorNodeSemantics::External, BehaviorNodeTickContext,
};
use crate::DefaultAiManager;

use super::{nav_target, navigation_world, spawn_nav_agent, tick_request, tick_with_world_host};

const HALF_LIMIT: f32 = 0.5;
const MOVE_DESTINATION: [f32; 3] = [9.0, 0.0, 0.0];

#[test]
fn bt162_time_limit_stops_running_move_to_at_deadline() {
    let manager = DefaultAiManager::default();
    let tree = manager
        .register_behavior_tree(limited_tree("bt162_move_to").with_node(move_to_task("child")))
        .expect("registered time-limited MoveTo");
    let mut world = navigation_world();
    let entity = spawn_nav_agent(&mut world);
    let position = Vec3::new(2.0, 0.0, 1.0);
    world
        .update_transform(entity, Transform::from_translation(position))
        .expect("position the navigation agent");
    assert_eq!(nav_target(&world, entity), None);

    let before_deadline = tick_with_world_host(&manager, tree, entity, &mut world, HALF_LIMIT);
    assert_eq!(before_deadline, AiDecisionStatus::Running);
    assert_eq!(nav_target(&world, entity), Some(MOVE_DESTINATION));

    // 没有导航反馈，超时这一 tick 本身就必须撤回正在使用的目标。
    let at_deadline = tick_with_world_host(&manager, tree, entity, &mut world, HALF_LIMIT);
    assert_eq!(at_deadline, AiDecisionStatus::Failed);
    assert_eq!(
        nav_target(&world, entity),
        Some(position.to_array()),
        "timeout must immediately replace the acquired destination with the agent's position"
    );
}

#[test]
fn bt162_time_limit_preserves_unowned_destination_for_move_to_placeholder() {
    let manager = DefaultAiManager::default();
    let sequence = AiBehaviorNodeDescriptor::new("child", AiBehaviorNodeKind::Sequence, "Sequence")
        .with_child("placeholder");
    let placeholder = move_to_task("placeholder").with_parameter("result", "running");
    let tree = manager
        .register_behavior_tree(
            limited_tree("bt162_move_to_placeholder")
                .with_node(sequence)
                .with_node(placeholder),
        )
        .expect("registered time-limited placeholder sequence");
    let mut world = navigation_world();
    let entity = spawn_nav_agent(&mut world);
    let other_destination = [3.0, 4.0, 5.0];
    let path =
        ComponentPropertyPath::new(NAV_MESH_AGENT_COMPONENT_TYPE, vec!["destination".into()])
            .expect("navigation destination path");
    world
        .set_property(entity, &path, ScenePropertyValue::Vec3(other_destination))
        .expect("set a destination owned by another system");

    let before_deadline = tick_with_world_host(&manager, tree, entity, &mut world, HALF_LIMIT);
    assert_eq!(before_deadline, AiDecisionStatus::Running);
    assert_eq!(nav_target(&world, entity), Some(other_destination));

    let at_deadline = tick_with_world_host(&manager, tree, entity, &mut world, HALF_LIMIT);
    assert_eq!(at_deadline, AiDecisionStatus::Failed);
    assert_eq!(
        nav_target(&world, entity),
        Some(other_destination),
        "an explicit result bypasses MoveTo and must not cancel an unowned destination"
    );
}

#[test]
fn bt162_time_limit_aborts_before_selector_fallback() {
    TRACE.with_borrow_mut(Vec::clear);
    let manager = manager_with_probes();
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("bt162_abort_order", "Abort order", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Selector")
                        .with_child("limit")
                        .with_child("fallback"),
                )
                .with_node(time_limit())
                .with_node(task("child", "test.bt162_running"))
                .with_node(task("fallback", "test.bt162_fallback")),
        )
        .expect("registered time-limited selector");

    assert_eq!(tick_probe(&manager, tree), AiDecisionStatus::Running);
    assert_eq!(TRACE.take(), ["child start"]);

    assert_eq!(tick_probe(&manager, tree), AiDecisionStatus::Succeeded);
    // 只验证同步撤销先于 fallback，不模拟 UE 的 latent-abort 等待。
    assert_eq!(
        TRACE.take(),
        ["child tick (running)", "child abort", "fallback tick"]
    );

    assert_eq!(tick_probe(&manager, tree), AiDecisionStatus::Running);
    assert_eq!(TRACE.take(), ["child start"]);
}

#[test]
fn bt162_time_limit_does_not_abort_child_that_finishes_at_deadline() {
    TRACE.with_borrow_mut(Vec::clear);
    let manager = manager_with_probes();
    let tree = manager
        .register_behavior_tree(
            limited_tree("bt162_natural_completion")
                .with_node(task("child", "test.bt162_finishes")),
        )
        .expect("registered time-limited finishing task");

    assert_eq!(tick_probe(&manager, tree), AiDecisionStatus::Running);
    assert_eq!(TRACE.take(), ["child start"]);

    assert_eq!(tick_probe(&manager, tree), AiDecisionStatus::Succeeded);
    assert_eq!(TRACE.take(), ["child finish"]);
}

fn time_limit() -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new("limit", AiBehaviorNodeKind::Decorator, "Time Limit")
        .with_implementation("time_limit")
        .with_parameter("time_limit_seconds", 1.0_f32)
        .with_child("child")
}

fn task(id: &str, implementation: &str) -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new(id, AiBehaviorNodeKind::Task, id)
        .with_implementation(implementation)
}

fn move_to_task(id: &str) -> AiBehaviorNodeDescriptor {
    task(id, "move_to").with_parameter("target", Vec3::from_array(MOVE_DESTINATION))
}

fn limited_tree(id: &str) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new(id, "Time-limited task", "limit").with_node(time_limit())
}

std::thread_local! {
    static TRACE: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug)]
enum ProbeKind {
    Running,
    Finishes,
    Fallback,
}

#[derive(Debug)]
struct Probe {
    kind: ProbeKind,
    started: bool,
}

impl BehaviorNodeRuntime for Probe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        let (event, status) = match (self.kind, self.started) {
            (ProbeKind::Fallback, _) => ("fallback tick", AiDecisionStatus::Succeeded),
            (_, false) => ("child start", AiDecisionStatus::Running),
            (ProbeKind::Running, true) => ("child tick (running)", AiDecisionStatus::Running),
            (ProbeKind::Finishes, true) => ("child finish", AiDecisionStatus::Succeeded),
        };
        self.started = true;
        TRACE.with_borrow_mut(|events| events.push(event));
        status
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        let event = match self.kind {
            ProbeKind::Fallback => "fallback abort",
            _ if self.started => "child abort",
            _ => "unticked child abort",
        };
        TRACE.with_borrow_mut(|events| events.push(event));
    }
}

fn boxed_probe(kind: ProbeKind) -> Box<dyn BehaviorNodeRuntime> {
    Box::new(Probe {
        kind,
        started: false,
    })
}

fn manager_with_probes() -> DefaultAiManager {
    let mut catalog = BehaviorNodeCatalog::with_standard_nodes().expect("standard node catalog");
    type Factory = fn() -> Box<dyn BehaviorNodeRuntime>;
    let probes: [(&str, Factory); 3] = [
        ("test.bt162_running", || boxed_probe(ProbeKind::Running)),
        ("test.bt162_finishes", || boxed_probe(ProbeKind::Finishes)),
        ("test.bt162_fallback", || boxed_probe(ProbeKind::Fallback)),
    ];
    for (id, factory) in probes {
        catalog
            .add_node(
                PluginModuleId::from_raw(162),
                BehaviorNodeDescriptor::new(id, id, Task, External).with_factory(factory),
            )
            .expect("registered time-limit probe");
    }
    DefaultAiManager::with_behavior_node_catalog(catalog)
}

fn tick_probe(manager: &DefaultAiManager, tree: AiBehaviorTreeId) -> AiDecisionStatus {
    manager
        .tick_agent(tick_request(tree, 1, HALF_LIMIT))
        .expect("time-limit probe tick")
        .status
}
