use std::sync::{Arc, Mutex, MutexGuard, RwLock};

mod behavior_tree;
mod blackboard;
mod execution_gate;
pub(crate) mod parameters;
mod perception;
mod service;
mod snapshot;
mod state;
mod tick;
pub(crate) mod validation;

use execution_gate::BehaviorNodeExecutionGate;
use state::AiRuntimeState;

#[derive(Clone, Debug)]
pub struct DefaultAiManager {
    state: Arc<Mutex<AiRuntimeState>>,
    behavior_node_catalog: Arc<RwLock<crate::behavior_tree::BehaviorNodeCatalog>>,
    behavior_node_execution_gate: BehaviorNodeExecutionGate,
}

impl DefaultAiManager {
    pub fn with_behavior_node_catalog(
        behavior_node_catalog: crate::behavior_tree::BehaviorNodeCatalog,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(AiRuntimeState::default())),
            behavior_node_catalog: Arc::new(RwLock::new(behavior_node_catalog)),
            behavior_node_execution_gate: BehaviorNodeExecutionGate::default(),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, AiRuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn behavior_node_catalog(
        &self,
    ) -> Arc<RwLock<crate::behavior_tree::BehaviorNodeCatalog>> {
        self.behavior_node_catalog.clone()
    }

    pub(crate) fn revoke_behavior_node_owner(
        &self,
        owner: zircon_runtime::plugin::PluginModuleId,
    ) -> Vec<crate::behavior_tree::BehaviorNodeSlot> {
        behavior_tree::revoke_node_owner(self, owner)
    }

    pub(crate) fn add_behavior_node(
        &self,
        owner: zircon_runtime::plugin::PluginModuleId,
        descriptor: crate::behavior_tree::BehaviorNodeDescriptor,
    ) -> Result<
        crate::behavior_tree::BehaviorNodeSlot,
        crate::behavior_tree::BehaviorNodeCatalogError,
    > {
        let registration_lease = self
            .behavior_node_execution_gate
            .acquire_registration(owner);
        let slot = self
            .behavior_node_catalog
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .add_node(owner, descriptor)?;
        drop(registration_lease);
        Ok(slot)
    }

    pub(crate) fn bind_standard_behavior_nodes_to_owner(
        &self,
        owner: zircon_runtime::plugin::PluginModuleId,
    ) -> Result<(), crate::behavior_tree::BehaviorNodeCatalogError> {
        let registration_lease = self
            .behavior_node_execution_gate
            .acquire_registration(owner);
        let result = self
            .behavior_node_catalog
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .bind_bootstrap_standard_nodes_to(owner);
        drop(registration_lease);
        result
    }

    pub fn tick_active_agents(
        &self,
        world: zircon_runtime::core::framework::scene::WorldHandle,
        delta_seconds: f32,
    ) -> Result<
        Vec<zircon_runtime::core::framework::ai::AiAgentTickReport>,
        zircon_runtime::core::framework::ai::AiManagerError,
    > {
        tick::tick_active_agents(self, world, delta_seconds)
    }

    pub fn active_agent_entities(
        &self,
        world: zircon_runtime::core::framework::scene::WorldHandle,
    ) -> Vec<u64> {
        tick::active_agent_entities(self, world)
    }

    pub(crate) fn replace_world_perception_snapshots(
        &self,
        world: zircon_runtime::core::framework::scene::WorldHandle,
        snapshots: Vec<zircon_runtime::core::framework::ai::AiPerceptionSnapshot>,
    ) -> Result<(), zircon_runtime::core::framework::ai::AiManagerError> {
        perception::replace_world_snapshots(self, world, snapshots)
    }

    pub(crate) fn runtime_snapshots_for_agents(
        &self,
        world: zircon_runtime::core::framework::scene::WorldHandle,
        entities: impl IntoIterator<Item = zircon_runtime::core::framework::scene::EntityId>,
    ) -> Vec<zircon_runtime::core::framework::ai::AiAgentRuntimeSnapshot> {
        snapshot::runtime_snapshots_for_agents(self, world, entities)
    }

    pub fn tick_active_agents_with_lod(
        &self,
        world: zircon_runtime::core::framework::scene::WorldHandle,
        delta_seconds: f32,
        frame: u64,
        lod_for_entity: impl FnMut(u64) -> crate::AiBehaviorTickLod,
    ) -> Result<
        Vec<zircon_runtime::core::framework::ai::AiAgentTickReport>,
        zircon_runtime::core::framework::ai::AiManagerError,
    > {
        tick::tick_active_agents_with_lod(self, world, delta_seconds, frame, lod_for_entity)
    }

    #[cfg(test)]
    pub(crate) fn tick_agent_with_integration_host(
        &self,
        request: zircon_runtime::core::framework::ai::AiAgentTickRequest,
        integration_host: &mut dyn crate::behavior_tree::BehaviorIntegrationHost,
    ) -> Result<
        zircon_runtime::core::framework::ai::AiAgentTickReport,
        zircon_runtime::core::framework::ai::AiManagerError,
    > {
        tick::tick_agent_with_integration_host(self, request, integration_host)
    }

    pub(crate) fn tick_active_agents_with_lod_and_integration_host(
        &self,
        world: zircon_runtime::core::framework::scene::WorldHandle,
        delta_seconds: f32,
        frame: u64,
        lod_for_entity: impl FnMut(u64) -> crate::AiBehaviorTickLod,
        integration_host: &mut dyn crate::behavior_tree::BehaviorIntegrationHost,
    ) -> Result<
        Vec<zircon_runtime::core::framework::ai::AiAgentTickReport>,
        zircon_runtime::core::framework::ai::AiManagerError,
    > {
        tick::tick_active_agents_with_lod_and_integration_host(
            self,
            world,
            delta_seconds,
            frame,
            lod_for_entity,
            integration_host,
        )
    }
}

impl Default for DefaultAiManager {
    fn default() -> Self {
        let catalog = crate::behavior_tree::BehaviorNodeCatalog::with_standard_nodes()
            .unwrap_or_else(|_| crate::behavior_tree::BehaviorNodeCatalog::default());
        Self::with_behavior_node_catalog(catalog)
    }
}
