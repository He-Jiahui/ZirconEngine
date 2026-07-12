use crate::behavior_tree::{
    standard_node_catalog, BehaviorNodeCatalog, BehaviorNodeCatalogError, BehaviorNodeCategory,
    BehaviorNodeDescriptor, BehaviorNodeSemantics,
};
use zircon_runtime::plugin::PluginModuleId;

#[test]
fn standard_node_catalog_snapshot() {
    let catalog = standard_node_catalog().expect("standard node catalog is valid");
    let snapshot = catalog
        .descriptors()
        .iter()
        .map(|descriptor| {
            (
                descriptor.id(),
                descriptor.category(),
                descriptor.semantics(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        snapshot,
        [
            (
                "blackboard_condition",
                BehaviorNodeCategory::Decorator,
                BehaviorNodeSemantics::BlackboardCondition
            ),
            (
                "cooldown",
                BehaviorNodeCategory::Decorator,
                BehaviorNodeSemantics::Cooldown
            ),
            (
                "emit_event",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::EmitEvent
            ),
            (
                "force_result",
                BehaviorNodeCategory::Decorator,
                BehaviorNodeSemantics::ForceResult
            ),
            (
                "inverter",
                BehaviorNodeCategory::Decorator,
                BehaviorNodeSemantics::Inverter
            ),
            (
                "loop",
                BehaviorNodeCategory::Decorator,
                BehaviorNodeSemantics::Loop
            ),
            (
                "move_to",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::MoveTo
            ),
            (
                "parallel",
                BehaviorNodeCategory::Composite,
                BehaviorNodeSemantics::Parallel
            ),
            (
                "play_animation",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::PlayAnimation
            ),
            (
                "random_selector",
                BehaviorNodeCategory::Composite,
                BehaviorNodeSemantics::RandomSelector
            ),
            (
                "run_subtree",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::RunSubtree
            ),
            (
                "script_task",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::ScriptTask
            ),
            (
                "selector",
                BehaviorNodeCategory::Composite,
                BehaviorNodeSemantics::Selector
            ),
            (
                "sequence",
                BehaviorNodeCategory::Composite,
                BehaviorNodeSemantics::Sequence
            ),
            (
                "set_blackboard",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::SetBlackboard
            ),
            (
                "time_limit",
                BehaviorNodeCategory::Decorator,
                BehaviorNodeSemantics::TimeLimit
            ),
            (
                "update_blackboard_distance",
                BehaviorNodeCategory::Service,
                BehaviorNodeSemantics::UpdateBlackboardDistance
            ),
            (
                "wait",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::Wait
            ),
        ]
    );
}

#[test]
fn catalog_rejects_duplicate_ids_and_freezes_to_dense_stable_slots() {
    let mut catalog = BehaviorNodeCatalog::default();
    let first = catalog
        .add_node(
            PluginModuleId::from_raw(42),
            BehaviorNodeDescriptor::new(
                "custom_task",
                "Custom Task",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            ),
        )
        .expect("first registration succeeds");
    assert_eq!(first.raw(), 0);
    assert_eq!(
        catalog.add_node(
            PluginModuleId::from_raw(43),
            BehaviorNodeDescriptor::new(
                "custom_task",
                "Duplicate",
                BehaviorNodeCategory::Task,
                BehaviorNodeSemantics::External,
            )
        ),
        Err(BehaviorNodeCatalogError::DuplicateId {
            id: "custom_task".to_string(),
        })
    );

    let frozen = catalog.freeze();
    assert_eq!(frozen.resolve("custom_task"), Some(first));
    assert_eq!(
        frozen.get(first).map(BehaviorNodeDescriptor::id),
        Some("custom_task")
    );
}
