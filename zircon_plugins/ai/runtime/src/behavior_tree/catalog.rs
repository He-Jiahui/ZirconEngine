use std::fmt;
use std::sync::Arc;
use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeParameter, AiBlackboardEntry, AiDecisionStatus, AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::bridge::PluginInterface;
use zircon_runtime::plugin::{
    ExtensionSlot, FrozenExtensionTable, PluginModuleId, TypedExtensionPoint,
};

use super::nodes::standard_node_descriptors;

#[cfg(test)]
#[path = "catalog/borrowed_lookup_tests.rs"]
mod borrowed_lookup_tests;

pub(crate) const BOOTSTRAP_BEHAVIOR_NODE_OWNER: PluginModuleId = PluginModuleId::from_raw(u32::MAX);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BehaviorNodeCategory {
    Composite,
    Decorator,
    Service,
    Task,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BehaviorNodeSemantics {
    Selector,
    Sequence,
    Parallel,
    RandomSelector,
    BlackboardCondition,
    Cooldown,
    TimeLimit,
    Loop,
    Inverter,
    ForceResult,
    UpdateBlackboardDistance,
    Wait,
    MoveTo,
    PlayAnimation,
    SetBlackboard,
    EmitEvent,
    RunSubtree,
    ScriptTask,
    External,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectorRecheckPolicy {
    #[default]
    Stable,
    RecheckWhileLowerPriorityRuns,
}

pub struct BehaviorNodeTickContext<'a> {
    parameters: &'a [AiBehaviorNodeParameter],
    blackboard: &'a [AiBlackboardEntry],
    perception: Option<&'a AiPerceptionSnapshot>,
    delta_seconds: f32,
}

impl<'a> BehaviorNodeTickContext<'a> {
    pub(crate) fn new(
        parameters: &'a [AiBehaviorNodeParameter],
        blackboard: &'a [AiBlackboardEntry],
        perception: Option<&'a AiPerceptionSnapshot>,
        delta_seconds: f32,
    ) -> Self {
        Self {
            parameters,
            blackboard,
            perception,
            delta_seconds,
        }
    }

    pub fn parameters(&self) -> &[AiBehaviorNodeParameter] {
        self.parameters
    }

    pub fn blackboard(&self) -> &[AiBlackboardEntry] {
        self.blackboard
    }

    pub fn perception(&self) -> Option<&AiPerceptionSnapshot> {
        self.perception
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }
}

pub trait BehaviorNodeRuntime: fmt::Debug + Send {
    fn tick(&mut self, context: &BehaviorNodeTickContext<'_>) -> AiDecisionStatus;

    /// Cancels a currently active runtime before its branch is reset or deactivated.
    fn on_abort(&mut self, _context: &BehaviorNodeTickContext<'_>) {}
}

#[derive(Clone, Copy)]
pub struct BehaviorNodeFactory {
    callback: fn() -> Box<dyn BehaviorNodeRuntime>,
}

impl BehaviorNodeFactory {
    pub fn new(callback: fn() -> Box<dyn BehaviorNodeRuntime>) -> Self {
        Self { callback }
    }

    pub(crate) fn create(self) -> Box<dyn BehaviorNodeRuntime> {
        (self.callback)()
    }
}

impl fmt::Debug for BehaviorNodeFactory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BehaviorNodeFactory(..)")
    }
}

impl PartialEq for BehaviorNodeFactory {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.callback, other.callback)
    }
}

impl Eq for BehaviorNodeFactory {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BehaviorNodeDescriptor {
    id: String,
    display_name: String,
    category: BehaviorNodeCategory,
    semantics: BehaviorNodeSemantics,
    selector_recheck: SelectorRecheckPolicy,
    factory: Option<BehaviorNodeFactory>,
}

impl BehaviorNodeDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        category: BehaviorNodeCategory,
        semantics: BehaviorNodeSemantics,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            category,
            semantics,
            selector_recheck: match semantics {
                BehaviorNodeSemantics::BlackboardCondition
                | BehaviorNodeSemantics::Cooldown
                | BehaviorNodeSemantics::RunSubtree => {
                    SelectorRecheckPolicy::RecheckWhileLowerPriorityRuns
                }
                _ => SelectorRecheckPolicy::Stable,
            },
            factory: None,
        }
    }

    pub fn with_factory(mut self, factory: fn() -> Box<dyn BehaviorNodeRuntime>) -> Self {
        self.factory = Some(BehaviorNodeFactory::new(factory));
        self
    }

    pub fn with_selector_recheck_policy(mut self, policy: SelectorRecheckPolicy) -> Self {
        self.selector_recheck = policy;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category(&self) -> BehaviorNodeCategory {
        self.category
    }

    pub fn semantics(&self) -> BehaviorNodeSemantics {
        self.semantics
    }

    pub fn selector_recheck_policy(&self) -> SelectorRecheckPolicy {
        self.selector_recheck
    }

    pub fn factory(&self) -> Option<BehaviorNodeFactory> {
        self.factory
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BehaviorNodeSlot(ExtensionSlot);

impl BehaviorNodeSlot {
    pub fn raw(self) -> u32 {
        self.0.raw()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BehaviorNodeCatalogError {
    EmptyId,
    DuplicateId { id: String },
}

impl fmt::Display for BehaviorNodeCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("node implementation id is empty"),
            Self::DuplicateId { id } => {
                write!(formatter, "node implementation id `{id}` is duplicated")
            }
        }
    }
}

impl std::error::Error for BehaviorNodeCatalogError {}

#[derive(Clone, Debug, Default)]
pub struct BehaviorNodeCatalog {
    descriptors: TypedExtensionPoint<String, BehaviorNodeDescriptor>,
}

impl BehaviorNodeCatalog {
    pub fn with_standard_nodes() -> Result<Self, BehaviorNodeCatalogError> {
        Self::with_standard_nodes_owned_by(BOOTSTRAP_BEHAVIOR_NODE_OWNER)
    }

    pub(crate) fn with_standard_nodes_owned_by(
        owner: PluginModuleId,
    ) -> Result<Self, BehaviorNodeCatalogError> {
        let mut catalog = Self::default();
        for &(id, display_name, category, semantics) in standard_node_descriptors() {
            catalog.add_node(
                owner,
                BehaviorNodeDescriptor::new(id, display_name, category, semantics),
            )?;
        }
        Ok(catalog)
    }

    pub(crate) fn bind_bootstrap_standard_nodes_to(
        &mut self,
        owner: PluginModuleId,
    ) -> Result<(), BehaviorNodeCatalogError> {
        if owner == BOOTSTRAP_BEHAVIOR_NODE_OWNER
            || self
                .descriptors
                .entries_owned_by(BOOTSTRAP_BEHAVIOR_NODE_OWNER)
                .next()
                .is_none()
        {
            return Ok(());
        }
        self.descriptors
            .reassign_owned_by(BOOTSTRAP_BEHAVIOR_NODE_OWNER, owner);
        Ok(())
    }

    pub fn add_node(
        &mut self,
        owner: PluginModuleId,
        descriptor: BehaviorNodeDescriptor,
    ) -> Result<BehaviorNodeSlot, BehaviorNodeCatalogError> {
        if descriptor.id.trim().is_empty() {
            return Err(BehaviorNodeCatalogError::EmptyId);
        }
        let id = descriptor.id.clone();
        self.descriptors
            .register(owner, id.clone(), descriptor)
            .map(BehaviorNodeSlot)
            .map_err(|_| BehaviorNodeCatalogError::DuplicateId { id })
    }

    pub fn freeze(mut self) -> FrozenBehaviorNodeCatalog {
        self.descriptors
            .sort_by_values(|left, right| left.id.cmp(&right.id));
        FrozenBehaviorNodeCatalog {
            descriptors: self.descriptors.finalize(),
        }
    }

    pub fn snapshot(&self) -> FrozenBehaviorNodeCatalog {
        self.clone().freeze()
    }

    pub fn remove_owned_by(&mut self, owner: PluginModuleId) -> Vec<BehaviorNodeSlot> {
        self.descriptors
            .remove_owned_by(owner)
            .into_iter()
            .map(BehaviorNodeSlot)
            .collect()
    }

    pub(crate) fn owner_for_slot(&self, slot: BehaviorNodeSlot) -> Option<PluginModuleId> {
        self.descriptors.owner_for_slot(slot.0)
    }
}

pub trait BehaviorNodeRegistry: Send + Sync {
    fn add_node(
        &self,
        owner: PluginModuleId,
        descriptor: BehaviorNodeDescriptor,
    ) -> Result<BehaviorNodeSlot, BehaviorNodeCatalogError>;

    fn descriptors(&self) -> Vec<BehaviorNodeDescriptor>;

    fn revoke_owner(&self, owner: PluginModuleId) -> Vec<BehaviorNodeSlot>;
}

impl PluginInterface for dyn BehaviorNodeRegistry {
    const INTERFACE_ID: &'static str = "ai.behavior_node_registry.v1";
}

#[derive(Clone, Debug)]
pub(crate) struct BehaviorNodeRegistryService {
    manager: Arc<crate::DefaultAiManager>,
}

impl BehaviorNodeRegistryService {
    pub(crate) fn new(manager: Arc<crate::DefaultAiManager>) -> Self {
        Self { manager }
    }
}

impl BehaviorNodeRegistry for BehaviorNodeRegistryService {
    fn add_node(
        &self,
        owner: PluginModuleId,
        descriptor: BehaviorNodeDescriptor,
    ) -> Result<BehaviorNodeSlot, BehaviorNodeCatalogError> {
        self.manager.add_behavior_node(owner, descriptor)
    }

    fn descriptors(&self) -> Vec<BehaviorNodeDescriptor> {
        self.manager
            .behavior_node_catalog()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .descriptors
            .values()
            .to_vec()
    }

    fn revoke_owner(&self, owner: PluginModuleId) -> Vec<BehaviorNodeSlot> {
        self.manager.revoke_behavior_node_owner(owner)
    }
}

#[derive(Clone, Debug)]
pub struct FrozenBehaviorNodeCatalog {
    descriptors: FrozenExtensionTable<String, BehaviorNodeDescriptor>,
}

impl FrozenBehaviorNodeCatalog {
    pub fn descriptors(&self) -> &[BehaviorNodeDescriptor] {
        self.descriptors.values()
    }

    pub fn resolve(&self, id: &str) -> Option<BehaviorNodeSlot> {
        self.descriptors.resolve_borrowed(id).map(BehaviorNodeSlot)
    }

    pub fn get(&self, slot: BehaviorNodeSlot) -> Option<&BehaviorNodeDescriptor> {
        self.descriptors.get(slot.0)
    }

    pub fn owner_for_slot(&self, slot: BehaviorNodeSlot) -> Option<PluginModuleId> {
        self.descriptors.owner_for_slot(slot.0)
    }
}

pub fn standard_node_catalog() -> Result<FrozenBehaviorNodeCatalog, BehaviorNodeCatalogError> {
    BehaviorNodeCatalog::with_standard_nodes().map(BehaviorNodeCatalog::freeze)
}
