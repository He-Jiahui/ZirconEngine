use std::sync::atomic::{AtomicUsize, Ordering};
use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiAgentTickRequest, AiBehaviorNodeDescriptor, AiBehaviorNodeKind,
    AiBehaviorTreeDescriptor, AiDecisionStatus, AiManager,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::plugin::PluginModuleId;

use crate::behavior_tree::{
    BehaviorNodeCatalog, BehaviorNodeCategory, BehaviorNodeDescriptor, BehaviorNodeRuntime,
    BehaviorNodeSemantics, BehaviorNodeTickContext, SelectorRecheckPolicy,
};
use crate::DefaultAiManager;

static SUCCESS_TICKS: AtomicUsize = AtomicUsize::new(0);
static FAILURE_TICKS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct CountingSuccess;

impl BehaviorNodeRuntime for CountingSuccess {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        SUCCESS_TICKS.fetch_add(1, Ordering::SeqCst);
        AiDecisionStatus::Succeeded
    }
}

#[derive(Debug)]
struct CountingFailure;

impl BehaviorNodeRuntime for CountingFailure {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        FAILURE_TICKS.fetch_add(1, Ordering::SeqCst);
        AiDecisionStatus::Failed
    }
}

fn success_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(CountingSuccess)
}

fn failure_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(CountingFailure)
}

#[derive(Debug, Default)]
struct FailThenSucceed {
    ticks: u32,
}

impl BehaviorNodeRuntime for FailThenSucceed {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        self.ticks += 1;
        if self.ticks == 1 {
            AiDecisionStatus::Failed
        } else {
            AiDecisionStatus::Succeeded
        }
    }
}

fn fail_then_succeed_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::<FailThenSucceed>::default()
}

#[test]
fn selector_rechecks_explicitly_reactive_external_condition() {
    let mut catalog = BehaviorNodeCatalog::with_standard_nodes().expect("standard catalog");
    catalog
        .add_node(
            PluginModuleId::from_raw(91),
            BehaviorNodeDescriptor::new(
                "test.reactive_condition",
                "Reactive Condition",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
            .with_selector_recheck_policy(SelectorRecheckPolicy::RecheckWhileLowerPriorityRuns)
            .with_factory(fail_then_succeed_factory),
        )
        .expect("reactive condition node");
    let manager = DefaultAiManager::with_behavior_node_catalog(catalog);
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("reactive_selector", "Reactive Selector", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                        .with_child("condition")
                        .with_child("fallback"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "condition",
                        AiBehaviorNodeKind::Task,
                        "Condition",
                    )
                    .with_implementation("test.reactive_condition"),
                )
                .with_node(result_task("fallback", AiDecisionStatus::Running)),
        )
        .expect("reactive selector tree");

    assert_eq!(
        tick(&manager, tree, 24, 0.1).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, 24, 0.1).status,
        AiDecisionStatus::Succeeded
    );
}

#[test]
fn selector_rechecks_cooldown_until_high_priority_branch_is_ready() {
    let manager = DefaultAiManager::default();
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("cooldown_selector", "Cooldown Selector", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                        .with_child("cooldown")
                        .with_child("fallback"),
                )
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "cooldown",
                        AiBehaviorNodeKind::Decorator,
                        "Cooldown",
                    )
                    .with_implementation("cooldown")
                    .with_parameter("cooldown_seconds", 0.2_f32)
                    .with_child("ready"),
                )
                .with_node(result_task("ready", AiDecisionStatus::Succeeded))
                .with_node(result_task("fallback", AiDecisionStatus::Running)),
        )
        .expect("cooldown selector tree");

    assert_eq!(
        tick(&manager, tree, 25, 0.1).status,
        AiDecisionStatus::Succeeded
    );
    assert_eq!(
        tick(&manager, tree, 25, 0.1).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, 25, 0.1).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, 25, 0.1).status,
        AiDecisionStatus::Succeeded
    );
}

#[test]
fn node_semantics_matrix_covers_standard_node_implementations() {
    for implementation in [
        "selector",
        "sequence",
        "parallel",
        "random_selector",
        "blackboard_condition",
        "cooldown",
        "time_limit",
        "loop",
        "inverter",
        "force_result",
        "update_blackboard_distance",
        "wait",
        "move_to",
        "play_animation",
        "set_blackboard",
        "emit_event",
        "run_subtree",
        "script_task",
    ] {
        for input in [
            AiDecisionStatus::Succeeded,
            AiDecisionStatus::Failed,
            AiDecisionStatus::Running,
        ] {
            if implementation == "run_subtree" {
                assert_run_subtree_status(input);
                continue;
            }
            let expected = expected_status(implementation, &input);
            let manager = DefaultAiManager::default();
            let tree_id = manager
                .register_behavior_tree(matrix_tree(implementation, &input))
                .unwrap_or_else(|error| panic!("{implementation} matrix tree: {error}"));
            let report = tick(&manager, tree_id, 1, 0.1);
            assert_eq!(
                report.status, expected,
                "standard implementation `{implementation}` with input {input:?}"
            );
        }
    }
}

fn assert_run_subtree_status(status: AiDecisionStatus) {
    let manager = DefaultAiManager::default();
    manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("matrix_child", "Child", "root")
                .with_node(result_task("root", status.clone())),
        )
        .expect("matrix subtree target");
    let parent = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("matrix_parent", "Parent", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Subtree")
                    .with_implementation("run_subtree")
                    .with_parameter("behavior_tree", "matrix_child"),
            ),
        )
        .expect("matrix subtree parent");
    assert_eq!(tick(&manager, parent, 22, 0.1).status, status);
}

#[test]
fn wait_cooldown_time_limit_and_loop_keep_per_agent_runtime_state() {
    let manager = DefaultAiManager::default();
    let wait = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("timed_wait", "Timed Wait", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Wait")
                    .with_implementation("wait")
                    .with_parameter("duration_seconds", 1.0_f32),
            ),
        )
        .expect("timed wait tree");
    assert_eq!(
        tick(&manager, wait, 10, 0.4).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, wait, 10, 0.4).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, wait, 10, 0.2).status,
        AiDecisionStatus::Succeeded
    );

    let cooldown = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("cooldown", "Cooldown", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "root",
                        AiBehaviorNodeKind::Decorator,
                        "Cooldown",
                    )
                    .with_implementation("cooldown")
                    .with_parameter("cooldown_seconds", 1.0_f32)
                    .with_child("child"),
                )
                .with_node(result_task("child", AiDecisionStatus::Succeeded)),
        )
        .expect("cooldown tree");
    assert_eq!(
        tick(&manager, cooldown, 11, 0.1).status,
        AiDecisionStatus::Succeeded
    );
    assert_eq!(
        tick(&manager, cooldown, 11, 0.4).status,
        AiDecisionStatus::Failed
    );
    assert_eq!(
        tick(&manager, cooldown, 11, 0.6).status,
        AiDecisionStatus::Failed
    );
    assert_eq!(
        tick(&manager, cooldown, 11, 0.1).status,
        AiDecisionStatus::Succeeded
    );

    let time_limit = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("time_limit", "Time Limit", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "root",
                        AiBehaviorNodeKind::Decorator,
                        "Time Limit",
                    )
                    .with_implementation("time_limit")
                    .with_parameter("time_limit_seconds", 1.0_f32)
                    .with_child("child"),
                )
                .with_node(result_task("child", AiDecisionStatus::Running)),
        )
        .expect("time-limit tree");
    assert_eq!(
        tick(&manager, time_limit, 12, 0.4).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, time_limit, 12, 0.6).status,
        AiDecisionStatus::Failed
    );

    let loop_tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("loop", "Loop", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Decorator, "Loop")
                        .with_implementation("loop")
                        .with_parameter("count", 2_i64)
                        .with_child("child"),
                )
                .with_node(result_task("child", AiDecisionStatus::Succeeded)),
        )
        .expect("loop tree");
    assert_eq!(
        tick(&manager, loop_tree, 13, 0.1).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, loop_tree, 13, 0.1).status,
        AiDecisionStatus::Succeeded
    );
}

#[test]
fn random_selector_respects_zero_weight_and_selects_a_stable_running_branch() {
    let manager = DefaultAiManager::default();
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("weighted", "Weighted", "root")
                .with_node(
                    AiBehaviorNodeDescriptor::new(
                        "root",
                        AiBehaviorNodeKind::Parallel,
                        "Random Selector",
                    )
                    .with_implementation("random_selector")
                    .with_parameter("weight.failed", 0.0_f32)
                    .with_parameter("weight.running", 1.0_f32)
                    .with_child("failed")
                    .with_child("running"),
                )
                .with_node(result_task("failed", AiDecisionStatus::Failed))
                .with_node(result_task("running", AiDecisionStatus::Running)),
        )
        .expect("weighted random-selector tree");

    let first = tick(&manager, tree, 14, 0.1);
    let second = tick(&manager, tree, 14, 0.1);

    assert_eq!(first.status, AiDecisionStatus::Running);
    assert_eq!(first.active_node.as_deref(), Some("running"));
    assert_eq!(second.active_node, first.active_node);
}

#[test]
fn rust_plugin_node_evaluator_runs_through_the_injected_typed_catalog() {
    #[derive(Debug, Default)]
    struct ExternalNode {
        ticks: u32,
    }

    impl BehaviorNodeRuntime for ExternalNode {
        fn tick(&mut self, context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
            self.ticks += 1;
            assert_eq!(context.delta_seconds(), 0.1);
            assert!(context.blackboard().is_empty());
            if self.ticks == 1 {
                AiDecisionStatus::Running
            } else {
                AiDecisionStatus::Succeeded
            }
        }
    }

    fn external_factory() -> Box<dyn BehaviorNodeRuntime> {
        Box::<ExternalNode>::default()
    }

    let mut catalog = BehaviorNodeCatalog::with_standard_nodes().expect("standard node catalog");
    catalog
        .add_node(
            PluginModuleId::from_raw(77),
            BehaviorNodeDescriptor::new(
                "test.external",
                "External Test",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
            .with_factory(external_factory),
        )
        .expect("external node registration");
    let manager = DefaultAiManager::with_behavior_node_catalog(catalog);
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("external", "External", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "External")
                    .with_implementation("test.external"),
            ),
        )
        .expect("tree compiled against injected catalog");

    assert_eq!(
        tick(&manager, tree, 15, 0.1).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, 15, 0.1).status,
        AiDecisionStatus::Succeeded
    );
}

#[test]
fn composite_running_resume_does_not_replay_terminal_sibling_side_effects() {
    for (implementation, first_implementation, counter) in [
        ("sequence", "test.success", &SUCCESS_TICKS),
        ("selector", "test.failure", &FAILURE_TICKS),
        ("parallel", "test.success", &SUCCESS_TICKS),
    ] {
        SUCCESS_TICKS.store(0, Ordering::SeqCst);
        FAILURE_TICKS.store(0, Ordering::SeqCst);
        let mut catalog =
            BehaviorNodeCatalog::with_standard_nodes().expect("standard node catalog");
        catalog
            .add_node(
                PluginModuleId::from_raw(88),
                BehaviorNodeDescriptor::new(
                    "test.success",
                    "Success",
                    BehaviorNodeCategory::Task,
                    BehaviorNodeSemantics::External,
                )
                .with_factory(success_factory),
            )
            .expect("success node");
        catalog
            .add_node(
                PluginModuleId::from_raw(88),
                BehaviorNodeDescriptor::new(
                    "test.failure",
                    "Failure",
                    BehaviorNodeCategory::Task,
                    BehaviorNodeSemantics::External,
                )
                .with_factory(failure_factory),
            )
            .expect("failure node");
        let manager = DefaultAiManager::with_behavior_node_catalog(catalog);
        let kind = match implementation {
            "sequence" => AiBehaviorNodeKind::Sequence,
            "selector" => AiBehaviorNodeKind::Selector,
            _ => AiBehaviorNodeKind::Parallel,
        };
        let tree = manager
            .register_behavior_tree(
                AiBehaviorTreeDescriptor::new(implementation, implementation, "root")
                    .with_node(
                        AiBehaviorNodeDescriptor::new("root", kind, "Root")
                            .with_implementation(implementation)
                            .with_child("terminal")
                            .with_child("running"),
                    )
                    .with_node(
                        AiBehaviorNodeDescriptor::new(
                            "terminal",
                            AiBehaviorNodeKind::Task,
                            "Terminal",
                        )
                        .with_implementation(first_implementation),
                    )
                    .with_node(result_task("running", AiDecisionStatus::Running)),
            )
            .expect("composite resume tree");

        assert_eq!(
            tick(&manager, tree, 20, 0.1).status,
            AiDecisionStatus::Running
        );
        assert_eq!(
            tick(&manager, tree, 20, 0.1).status,
            AiDecisionStatus::Running
        );
        assert_eq!(counter.load(Ordering::SeqCst), 1, "{implementation}");
    }
}

#[test]
fn quarter_lod_accumulates_elapsed_time_for_timed_nodes() {
    let manager = DefaultAiManager::default();
    let tree = manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("lod_wait", "LOD Wait", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Wait")
                    .with_implementation("wait")
                    .with_parameter("duration_seconds", 1.0_f32),
            ),
        )
        .expect("LOD wait tree");
    assert_eq!(
        tick(&manager, tree, 0, 0.0).status,
        AiDecisionStatus::Running
    );

    for frame in 0..4 {
        let reports = manager
            .tick_active_agents_with_lod(WorldHandle::new(100), 0.25, frame, |_| {
                crate::AiBehaviorTickLod::Quarter
            })
            .expect("LOD tick");
        if frame == 0 {
            assert_eq!(reports[0].status, AiDecisionStatus::Running);
        } else {
            assert!(reports.is_empty());
        }
    }
    let reports = manager
        .tick_active_agents_with_lod(WorldHandle::new(100), 0.25, 4, |_| {
            crate::AiBehaviorTickLod::Quarter
        })
        .expect("quarter-rate elapsed tick");
    assert_eq!(reports[0].status, AiDecisionStatus::Succeeded);
}

#[test]
fn run_subtree_maps_success_failure_and_running_statuses() {
    for status in [
        AiDecisionStatus::Succeeded,
        AiDecisionStatus::Failed,
        AiDecisionStatus::Running,
    ] {
        let manager = DefaultAiManager::default();
        manager
            .register_behavior_tree(
                AiBehaviorTreeDescriptor::new("child", "Child", "root")
                    .with_node(result_task("root", status.clone())),
            )
            .expect("subtree target");
        let parent = manager
            .register_behavior_tree(
                AiBehaviorTreeDescriptor::new("parent", "Parent", "root").with_node(
                    AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Subtree, "Subtree")
                        .with_implementation("run_subtree")
                        .with_parameter("behavior_tree", "child"),
                ),
            )
            .expect("subtree parent");
        assert_eq!(tick(&manager, parent, 21, 0.1).status, status);
    }
}

fn matrix_tree(implementation: &str, status: &AiDecisionStatus) -> AiBehaviorTreeDescriptor {
    let tree_id = format!("matrix_{implementation}_{}", status_name(status));
    let kind = match implementation {
        "selector" => AiBehaviorNodeKind::Selector,
        "sequence" => AiBehaviorNodeKind::Sequence,
        "parallel" | "random_selector" => AiBehaviorNodeKind::Parallel,
        "blackboard_condition"
        | "cooldown"
        | "time_limit"
        | "loop"
        | "inverter"
        | "force_result" => AiBehaviorNodeKind::Decorator,
        "update_blackboard_distance" => AiBehaviorNodeKind::Service,
        _ => AiBehaviorNodeKind::Task,
    };
    let mut root = AiBehaviorNodeDescriptor::new("root", kind, implementation)
        .with_implementation(implementation);
    if implementation == "force_result" {
        root = root.with_parameter("forced_result", status_name(status));
    }
    if matches!(kind, AiBehaviorNodeKind::Service) {
        root = root.with_parameter("service_result", status_name(status));
    } else if !matches!(kind, AiBehaviorNodeKind::Task) {
        root = root.with_child("child");
    } else {
        root = root.with_parameter("result", status_name(status));
    }
    let mut tree = AiBehaviorTreeDescriptor::new(tree_id, implementation, "root").with_node(root);
    if !matches!(kind, AiBehaviorNodeKind::Task | AiBehaviorNodeKind::Service) {
        tree = tree.with_node(result_task("child", status.clone()));
    }
    tree
}

fn result_task(id: &str, status: AiDecisionStatus) -> AiBehaviorNodeDescriptor {
    AiBehaviorNodeDescriptor::new(id, AiBehaviorNodeKind::Task, id)
        .with_parameter("result", status_name(&status))
}

fn expected_status(implementation: &str, input: &AiDecisionStatus) -> AiDecisionStatus {
    if implementation == "inverter" {
        return match input {
            AiDecisionStatus::Succeeded => AiDecisionStatus::Failed,
            AiDecisionStatus::Failed => AiDecisionStatus::Succeeded,
            status => status.clone(),
        };
    }
    input.clone()
}

fn status_name(status: &AiDecisionStatus) -> &'static str {
    match status {
        AiDecisionStatus::Idle => "idle",
        AiDecisionStatus::Running => "running",
        AiDecisionStatus::Succeeded => "succeeded",
        AiDecisionStatus::Failed => "failed",
        AiDecisionStatus::Blocked => "blocked",
    }
}

fn tick(
    manager: &DefaultAiManager,
    behavior_tree: zircon_runtime::core::framework::ai::AiBehaviorTreeId,
    entity: u64,
    delta_seconds: f32,
) -> AiAgentTickReport {
    manager
        .tick_agent(AiAgentTickRequest {
            world: WorldHandle::new(100),
            entity,
            behavior_tree: Some(behavior_tree),
            blackboard_schema: None,
            delta_seconds,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("behavior-tree tick")
}
