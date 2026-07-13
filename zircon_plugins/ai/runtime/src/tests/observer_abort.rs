use std::sync::atomic::{AtomicUsize, Ordering};

use zircon_runtime::core::framework::ai::{
    AiAgentTickRequest, AiBehaviorAbortPolicy, AiBehaviorNodeDescriptor, AiBehaviorNodeKind,
    AiBehaviorTreeDescriptor, AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue,
    AiDecisionStatus, AiManager,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::plugin::PluginModuleId;

use crate::behavior_tree::{
    BehaviorNodeCatalog, BehaviorNodeCategory, BehaviorNodeDescriptor, BehaviorNodeRuntime,
    BehaviorNodeSemantics, BehaviorNodeTickContext,
};
use crate::DefaultAiManager;

static TIMING_ABORTS: AtomicUsize = AtomicUsize::new(0);
static COOLDOWN_ABORTS: AtomicUsize = AtomicUsize::new(0);
static TERMINAL_ABORTS: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_ABORTS: AtomicUsize = AtomicUsize::new(0);
static SELF_POLICY_ABORTS: AtomicUsize = AtomicUsize::new(0);
static SUBTREE_ABORTS: AtomicUsize = AtomicUsize::new(0);
static PENDING_ABORTS: AtomicUsize = AtomicUsize::new(0);
static PARALLEL_SUCCESS_TICKS: AtomicUsize = AtomicUsize::new(0);
static PARALLEL_ACTIVE_ABORTS: AtomicUsize = AtomicUsize::new(0);
static POLICY_ABORTS: AtomicUsize = AtomicUsize::new(0);
pub(super) static SWITCH_ABORTS: AtomicUsize = AtomicUsize::new(0);
pub(super) static DISABLE_ABORTS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct TimingAbortAwareRunning;

impl BehaviorNodeRuntime for TimingAbortAwareRunning {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        TIMING_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn abort_aware_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(TimingAbortAwareRunning)
}

#[derive(Debug)]
struct CooldownAbortAwareRunning;

impl BehaviorNodeRuntime for CooldownAbortAwareRunning {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        COOLDOWN_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn cooldown_abort_aware_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(CooldownAbortAwareRunning)
}

#[derive(Debug)]
struct TerminalAbortProbe;

impl BehaviorNodeRuntime for TerminalAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Succeeded
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        TERMINAL_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn terminal_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(TerminalAbortProbe)
}

#[derive(Debug)]
struct ActiveAbortProbe;

impl BehaviorNodeRuntime for ActiveAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        ACTIVE_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn active_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(ActiveAbortProbe)
}

#[derive(Debug)]
struct SelfPolicyAbortProbe;

impl BehaviorNodeRuntime for SelfPolicyAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        SELF_POLICY_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn self_policy_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(SelfPolicyAbortProbe)
}

#[derive(Debug)]
struct SubtreeAbortProbe;

impl BehaviorNodeRuntime for SubtreeAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        SUBTREE_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn subtree_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(SubtreeAbortProbe)
}

#[derive(Debug)]
struct PendingAbortProbe;

impl BehaviorNodeRuntime for PendingAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        PENDING_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn pending_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(PendingAbortProbe)
}

#[derive(Debug)]
struct ParallelTerminalProbe;

impl BehaviorNodeRuntime for ParallelTerminalProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        PARALLEL_SUCCESS_TICKS.fetch_add(1, Ordering::SeqCst);
        AiDecisionStatus::Succeeded
    }
}

fn parallel_terminal_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(ParallelTerminalProbe)
}

#[derive(Debug)]
struct ParallelActiveProbe;

impl BehaviorNodeRuntime for ParallelActiveProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        PARALLEL_ACTIVE_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn parallel_active_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(ParallelActiveProbe)
}

#[derive(Debug)]
struct PolicyAbortProbe;

impl BehaviorNodeRuntime for PolicyAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        POLICY_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn policy_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(PolicyAbortProbe)
}

#[derive(Debug)]
struct SwitchAbortProbe;

impl BehaviorNodeRuntime for SwitchAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        SWITCH_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn switch_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(SwitchAbortProbe)
}

#[derive(Debug)]
struct DisableAbortProbe;

impl BehaviorNodeRuntime for DisableAbortProbe {
    fn tick(&mut self, _context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus {
        AiDecisionStatus::Running
    }

    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {
        DISABLE_ABORTS.fetch_add(1, Ordering::SeqCst);
    }
}

fn disable_abort_probe_factory() -> Box<dyn BehaviorNodeRuntime> {
    Box::new(DisableAbortProbe)
}

#[test]
fn lower_priority_abort_timing_contract() {
    TIMING_ABORTS.store(0, Ordering::SeqCst);
    let (manager, schema) = manager_with_schema();
    let tree = manager
        .register_behavior_tree(guarded_fallback_tree(false))
        .expect("guarded fallback tree");

    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Running
    );
    assert_eq!(TIMING_ABORTS.load(Ordering::SeqCst), 0);

    assert_eq!(
        tick(&manager, tree, schema, true).status,
        AiDecisionStatus::Succeeded
    );
    assert_eq!(
        TIMING_ABORTS.load(Ordering::SeqCst),
        1,
        "observer abort must run cleanup before the higher-priority branch takes over"
    );
}

#[test]
fn abort_preserves_cooldown_state() {
    COOLDOWN_ABORTS.store(0, Ordering::SeqCst);
    let (manager, schema) = manager_with_schema();
    let tree = manager
        .register_behavior_tree(guarded_fallback_tree(true))
        .expect("guarded cooldown tree");

    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Succeeded,
        "the fallback cooldown branch primes its cooldown"
    );
    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Running,
        "the primed cooldown falls through to the latent fallback"
    );
    assert_eq!(
        tick(&manager, tree, schema, true).status,
        AiDecisionStatus::Succeeded
    );
    assert_eq!(COOLDOWN_ABORTS.load(Ordering::SeqCst), 1);
    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Running,
        "aborting the branch must not reset decorator cooldown memory"
    );
}

#[test]
fn abort_policy_controls_preemption_and_self_cleanup() {
    for policy in [AiBehaviorAbortPolicy::None, AiBehaviorAbortPolicy::Self_] {
        POLICY_ABORTS.store(0, Ordering::SeqCst);
        let (manager, schema) = manager_with_schema();
        let tree = manager
            .register_behavior_tree(guarded_fallback_tree_for_implementation(
                policy,
                "test.policy_abort_probe",
            ))
            .expect("non-preempting policy tree");
        assert_eq!(
            tick(&manager, tree, schema, false).status,
            AiDecisionStatus::Running
        );
        assert_eq!(
            tick(&manager, tree, schema, true).status,
            AiDecisionStatus::Running,
            "{policy:?} must not preempt a lower-priority running branch"
        );
        assert_eq!(POLICY_ABORTS.load(Ordering::SeqCst), 0);
    }

    SELF_POLICY_ABORTS.store(0, Ordering::SeqCst);
    let (manager, schema) = manager_with_schema();
    let tree = manager
        .register_behavior_tree(self_abort_tree(AiBehaviorAbortPolicy::Both))
        .expect("both-policy self abort tree");
    assert_eq!(
        tick(&manager, tree, schema, true).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Running
    );
    assert_eq!(SELF_POLICY_ABORTS.load(Ordering::SeqCst), 1);
}

#[test]
fn abort_only_notifies_active_external_runtime() {
    TERMINAL_ABORTS.store(0, Ordering::SeqCst);
    ACTIVE_ABORTS.store(0, Ordering::SeqCst);
    let (manager, schema) = manager_with_schema();
    let tree = manager
        .register_behavior_tree(inactive_sibling_tree())
        .expect("inactive sibling tree");
    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, schema, true).status,
        AiDecisionStatus::Succeeded
    );
    assert_eq!(ACTIVE_ABORTS.load(Ordering::SeqCst), 1);
    assert_eq!(
        TERMINAL_ABORTS.load(Ordering::SeqCst),
        0,
        "a terminal sibling is reset without receiving on_abort"
    );
}

#[test]
fn abort_recurses_into_active_run_subtree() {
    SUBTREE_ABORTS.store(0, Ordering::SeqCst);
    let (manager, schema) = manager_with_schema();
    manager
        .register_behavior_tree(
            AiBehaviorTreeDescriptor::new("abort_child", "Abort Child", "root").with_node(
                AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
                    .with_implementation("test.subtree_abort_probe"),
            ),
        )
        .expect("abort child tree");
    let tree = manager
        .register_behavior_tree(subtree_fallback_tree())
        .expect("subtree fallback tree");
    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, schema, true).status,
        AiDecisionStatus::Succeeded
    );
    assert_eq!(SUBTREE_ABORTS.load(Ordering::SeqCst), 1);
}

#[test]
fn manager_writes_preserve_pending_slot_notifications() {
    PENDING_ABORTS.store(0, Ordering::SeqCst);
    let (manager, schema) = manager_with_schema();
    let tree = manager
        .register_behavior_tree(guarded_tree_with_fallback(
            "pending_manager_write",
            AiBehaviorNodeDescriptor::new("fallback", AiBehaviorNodeKind::Task, "Fallback")
                .with_implementation("test.pending_abort_probe"),
        ))
        .expect("pending manager write tree");
    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Running
    );
    manager
        .set_blackboard_entries(
            WorldHandle::new(901),
            17,
            vec![AiBlackboardEntry::new(
                "alert",
                AiBlackboardValue::Bool(false),
            )],
        )
        .expect("same value write");
    assert_eq!(
        manager
            .tick_active_agents(WorldHandle::new(901), 0.1)
            .expect("same value active tick")[0]
            .status,
        AiDecisionStatus::Running
    );
    assert_eq!(PENDING_ABORTS.load(Ordering::SeqCst), 0);

    manager
        .set_blackboard_entries(
            WorldHandle::new(901),
            17,
            vec![AiBlackboardEntry::new(
                "alert",
                AiBlackboardValue::Bool(true),
            )],
        )
        .expect("changed value write");
    assert_eq!(
        manager
            .tick_active_agents(WorldHandle::new(901), 0.1)
            .expect("changed value active tick")[0]
            .status,
        AiDecisionStatus::Succeeded
    );
    assert_eq!(PENDING_ABORTS.load(Ordering::SeqCst), 1);
}

#[test]
fn self_abort_preserves_parallel_terminal_siblings() {
    PARALLEL_SUCCESS_TICKS.store(0, Ordering::SeqCst);
    PARALLEL_ACTIVE_ABORTS.store(0, Ordering::SeqCst);
    let (manager, schema) = manager_with_schema();
    let tree = manager
        .register_behavior_tree(parallel_self_abort_tree())
        .expect("parallel self-abort tree");
    assert_eq!(
        tick(&manager, tree, schema, true).status,
        AiDecisionStatus::Running
    );
    assert_eq!(
        tick(&manager, tree, schema, false).status,
        AiDecisionStatus::Failed
    );
    assert_eq!(PARALLEL_ACTIVE_ABORTS.load(Ordering::SeqCst), 1);
    assert_eq!(
        PARALLEL_SUCCESS_TICKS.load(Ordering::SeqCst),
        1,
        "Self_ abort must not clear a parallel sibling's terminal cache"
    );
}

pub(super) fn manager_with_schema() -> (
    DefaultAiManager,
    zircon_runtime::core::framework::ai::AiBlackboardSchemaId,
) {
    let mut catalog = BehaviorNodeCatalog::with_standard_nodes().expect("standard node catalog");
    catalog
        .add_node(
            PluginModuleId::from_raw(201),
            BehaviorNodeDescriptor::new(
                "test.abort_aware_running",
                "Abort-aware Running",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
            .with_factory(abort_aware_factory),
        )
        .expect("abort-aware node");
    catalog
        .add_node(
            PluginModuleId::from_raw(202),
            BehaviorNodeDescriptor::new(
                "test.cooldown_abort_aware_running",
                "Cooldown Abort-aware Running",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
            .with_factory(cooldown_abort_aware_factory),
        )
        .expect("cooldown abort-aware node");
    for (owner, id, display_name, factory) in [
        (
            203,
            "test.terminal_abort_probe",
            "Terminal Abort Probe",
            terminal_abort_probe_factory as fn() -> Box<dyn BehaviorNodeRuntime>,
        ),
        (
            204,
            "test.active_abort_probe",
            "Active Abort Probe",
            active_abort_probe_factory,
        ),
        (
            205,
            "test.subtree_abort_probe",
            "Subtree Abort Probe",
            subtree_abort_probe_factory,
        ),
        (
            206,
            "test.pending_abort_probe",
            "Pending Abort Probe",
            pending_abort_probe_factory,
        ),
        (
            207,
            "test.parallel_terminal_probe",
            "Parallel Terminal Probe",
            parallel_terminal_probe_factory,
        ),
        (
            208,
            "test.parallel_active_probe",
            "Parallel Active Probe",
            parallel_active_probe_factory,
        ),
        (
            209,
            "test.self_policy_abort_probe",
            "Self Policy Abort Probe",
            self_policy_abort_probe_factory,
        ),
        (
            210,
            "test.policy_abort_probe",
            "Policy Abort Probe",
            policy_abort_probe_factory,
        ),
        (
            211,
            "test.switch_abort_probe",
            "Switch Abort Probe",
            switch_abort_probe_factory,
        ),
        (
            212,
            "test.disable_abort_probe",
            "Disable Abort Probe",
            disable_abort_probe_factory,
        ),
    ] {
        catalog
            .add_node(
                PluginModuleId::from_raw(owner),
                BehaviorNodeDescriptor::new(
                    id,
                    display_name,
                    BehaviorNodeCategory::Task,
                    BehaviorNodeSemantics::External,
                )
                .with_factory(factory),
            )
            .expect("abort probe node");
    }
    let manager = DefaultAiManager::with_behavior_node_catalog(catalog);
    let schema = manager
        .register_blackboard_schema(
            AiBlackboardSchemaDescriptor::new("observer", "Observer")
                .with_key("alert", "bool", true),
        )
        .expect("observer schema");
    (manager, schema)
}

pub(super) fn guarded_fallback_tree(with_cooldown: bool) -> AiBehaviorTreeDescriptor {
    guarded_fallback_tree_with_policy(AiBehaviorAbortPolicy::LowerPriority, with_cooldown)
}

fn guarded_fallback_tree_with_policy(
    policy: AiBehaviorAbortPolicy,
    with_cooldown: bool,
) -> AiBehaviorTreeDescriptor {
    if !with_cooldown {
        return guarded_fallback_tree_for_implementation(policy, "test.abort_aware_running");
    }
    let mut tree = AiBehaviorTreeDescriptor::new("observer_abort", "Observer Abort", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("guard")
                .with_child("fallback"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("guard", AiBehaviorNodeKind::Decorator, "Alert")
                .with_parameter("blackboard_key", "alert")
                .with_parameter("equals_bool", true)
                .with_abort_policy(policy)
                .with_child("engage"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("engage", AiBehaviorNodeKind::Task, "Engage")
                .with_parameter("result", "succeeded"),
        );
    let fallback = if with_cooldown {
        AiBehaviorNodeDescriptor::new("fallback", AiBehaviorNodeKind::Selector, "Fallback")
            .with_child("cooldown")
            .with_child("latent")
    } else {
        AiBehaviorNodeDescriptor::new("fallback", AiBehaviorNodeKind::Task, "Fallback")
            .with_implementation("test.abort_aware_running")
    };
    tree = tree.with_node(fallback);
    if with_cooldown {
        tree = tree
            .with_node(
                AiBehaviorNodeDescriptor::new(
                    "cooldown",
                    AiBehaviorNodeKind::Decorator,
                    "Cooldown",
                )
                .with_implementation("cooldown")
                .with_parameter("cooldown_seconds", 10.0_f32)
                .with_child("prime"),
            )
            .with_node(
                AiBehaviorNodeDescriptor::new("prime", AiBehaviorNodeKind::Task, "Prime")
                    .with_parameter("result", "succeeded"),
            )
            .with_node(
                AiBehaviorNodeDescriptor::new("latent", AiBehaviorNodeKind::Task, "Latent")
                    .with_implementation("test.cooldown_abort_aware_running"),
            );
    }
    tree
}

fn guarded_fallback_tree_for_implementation(
    policy: AiBehaviorAbortPolicy,
    implementation: &str,
) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("observer_abort", "Observer Abort", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("guard")
                .with_child("fallback"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("guard", AiBehaviorNodeKind::Decorator, "Alert")
                .with_parameter("blackboard_key", "alert")
                .with_parameter("equals_bool", true)
                .with_abort_policy(policy)
                .with_child("engage"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("engage", AiBehaviorNodeKind::Task, "Engage")
                .with_parameter("result", "succeeded"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("fallback", AiBehaviorNodeKind::Task, "Fallback")
                .with_implementation(implementation),
        )
}

fn self_abort_tree(policy: AiBehaviorAbortPolicy) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("self_abort", "Self Abort", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("guard")
                .with_child("fallback"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("guard", AiBehaviorNodeKind::Decorator, "Guard")
                .with_parameter("blackboard_key", "alert")
                .with_parameter("equals_bool", true)
                .with_abort_policy(policy)
                .with_child("active"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("active", AiBehaviorNodeKind::Task, "Active")
                .with_implementation("test.self_policy_abort_probe"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("fallback", AiBehaviorNodeKind::Task, "Fallback")
                .with_parameter("result", "running"),
        )
}

fn inactive_sibling_tree() -> AiBehaviorTreeDescriptor {
    guarded_tree_with_fallback(
        "inactive_sibling",
        AiBehaviorNodeDescriptor::new("fallback", AiBehaviorNodeKind::Sequence, "Fallback")
            .with_child("terminal")
            .with_child("active"),
    )
    .with_node(
        AiBehaviorNodeDescriptor::new("terminal", AiBehaviorNodeKind::Task, "Terminal")
            .with_implementation("test.terminal_abort_probe"),
    )
    .with_node(
        AiBehaviorNodeDescriptor::new("active", AiBehaviorNodeKind::Task, "Active")
            .with_implementation("test.active_abort_probe"),
    )
}

fn subtree_fallback_tree() -> AiBehaviorTreeDescriptor {
    guarded_tree_with_fallback(
        "subtree_fallback",
        AiBehaviorNodeDescriptor::new("fallback", AiBehaviorNodeKind::Subtree, "Fallback")
            .with_implementation("run_subtree")
            .with_parameter("behavior_tree", "abort_child"),
    )
}

fn guarded_tree_with_fallback(
    id: &str,
    fallback: AiBehaviorNodeDescriptor,
) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new(id, id, "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root")
                .with_child("guard")
                .with_child("fallback"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("guard", AiBehaviorNodeKind::Decorator, "Guard")
                .with_parameter("blackboard_key", "alert")
                .with_parameter("equals_bool", true)
                .with_abort_policy(AiBehaviorAbortPolicy::LowerPriority)
                .with_child("engage"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("engage", AiBehaviorNodeKind::Task, "Engage")
                .with_parameter("result", "succeeded"),
        )
        .with_node(fallback)
}

fn parallel_self_abort_tree() -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new("parallel_self_abort", "Parallel Self Abort", "root")
        .with_node(
            AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Parallel, "Root")
                .with_child("terminal")
                .with_child("guard"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("terminal", AiBehaviorNodeKind::Task, "Terminal")
                .with_implementation("test.parallel_terminal_probe"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("guard", AiBehaviorNodeKind::Decorator, "Guard")
                .with_parameter("blackboard_key", "alert")
                .with_parameter("equals_bool", true)
                .with_abort_policy(AiBehaviorAbortPolicy::Self_)
                .with_child("active"),
        )
        .with_node(
            AiBehaviorNodeDescriptor::new("active", AiBehaviorNodeKind::Task, "Active")
                .with_implementation("test.parallel_active_probe"),
        )
}

pub(super) fn single_external_tree(id: &str, implementation: &str) -> AiBehaviorTreeDescriptor {
    AiBehaviorTreeDescriptor::new(id, id, "root").with_node(
        AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Task, "Root")
            .with_implementation(implementation),
    )
}

fn tick(
    manager: &DefaultAiManager,
    behavior_tree: zircon_runtime::core::framework::ai::AiBehaviorTreeId,
    schema: zircon_runtime::core::framework::ai::AiBlackboardSchemaId,
    alert: bool,
) -> zircon_runtime::core::framework::ai::AiAgentTickReport {
    manager
        .tick_agent(AiAgentTickRequest {
            world: WorldHandle::new(901),
            entity: 17,
            behavior_tree: Some(behavior_tree),
            blackboard_schema: Some(schema),
            delta_seconds: 0.1,
            blackboard: vec![AiBlackboardEntry::new(
                "alert",
                AiBlackboardValue::Bool(alert),
            )],
            perception: None,
        })
        .expect("observer tick")
}

pub(super) fn tick_without_schema(
    manager: &DefaultAiManager,
    behavior_tree: Option<zircon_runtime::core::framework::ai::AiBehaviorTreeId>,
    entity: u64,
) -> zircon_runtime::core::framework::ai::AiAgentTickReport {
    manager
        .tick_agent(AiAgentTickRequest {
            world: WorldHandle::new(904),
            entity,
            behavior_tree,
            blackboard_schema: None,
            delta_seconds: 0.1,
            blackboard: Vec::new(),
            perception: None,
        })
        .expect("schema-free tick")
}
