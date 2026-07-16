mod catalog;
mod compile;
mod executor;
mod nodes;

pub(crate) const SUBTREE_TARGET_PARAMETER_KEY: &str = "behavior_tree";

pub(crate) use catalog::BehaviorNodeRegistryService;
pub use catalog::{
    standard_node_catalog, BehaviorNodeCatalog, BehaviorNodeCatalogError, BehaviorNodeCategory,
    BehaviorNodeDescriptor, BehaviorNodeFactory, BehaviorNodeRegistry, BehaviorNodeRuntime,
    BehaviorNodeSemantics, BehaviorNodeSlot, BehaviorNodeTickContext, FrozenBehaviorNodeCatalog,
    SelectorRecheckPolicy,
};
pub(crate) use compile::reachable_behavior_trees;
pub use compile::{
    compile_behavior_tree, compile_behavior_tree_toml, compile_behavior_tree_with_catalog,
    BehaviorTreeAssetError, BehaviorTreeCompileError, CompiledBehaviorNode, CompiledBehaviorTree,
};
pub(crate) use executor::{
    abort_behavior_tree_instance, evaluate_behavior_tree, BehaviorTreeInstanceState,
};
#[cfg(test)]
pub(crate) use nodes::IntegrationTaskResult;
pub(crate) use nodes::{
    BehaviorIntegrationHost, BehaviorIntegrationTaskContext, RuntimeBehaviorIntegrationHost,
};
