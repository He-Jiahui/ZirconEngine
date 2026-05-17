use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::config_store::ConfigStore;
use crate::core::diagnostics::{DiagnosticPath, DiagnosticStore, DiagnosticStoreSnapshot};
use crate::core::error::CoreError;
use crate::core::event_bus::{EngineEvent, EventBus};
use crate::core::framework::time::{Fixed, Real, Time, Virtual};
use crate::core::job_scheduler::JobScheduler;
use crate::core::state::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
use crate::core::tasks::{TaskPool, TaskPoolKind, TaskPoolReport, TaskPools};
use crate::core::time::{RuntimeTimeAdvance, RuntimeTimeClocks};
use crate::core::types::ChannelReceiver;
use crate::plugin::{RuntimeExtensionRegistry, RuntimeExtensionRegistryError};

use super::handle::CoreHandle;
use super::state::CoreRuntimeInner;
use super::weak::CoreWeak;
use super::ModuleDescriptor;

#[derive(Clone)]
pub struct CoreRuntime {
    inner: Arc<CoreRuntimeInner>,
}

impl CoreRuntime {
    pub fn new() -> Self {
        let task_pools = TaskPools::default();
        Self {
            inner: Arc::new(CoreRuntimeInner {
                modules: Default::default(),
                services: Default::default(),
                event_bus: EventBus::default(),
                config_store: ConfigStore::default(),
                scheduler: JobScheduler::from_pool(task_pools.compute().clone()),
                task_pools,
                frame_clock: Default::default(),
                time: Default::default(),
                diagnostics: Default::default(),
                states: Default::default(),
                scene_hooks: Default::default(),
            }),
        }
    }

    pub fn handle(&self) -> CoreHandle {
        CoreHandle {
            inner: self.inner.clone(),
        }
    }

    pub fn weak(&self) -> CoreWeak {
        self.handle().downgrade()
    }

    pub fn scheduler(&self) -> &JobScheduler {
        &self.inner.scheduler
    }

    pub fn task_pools(&self) -> &TaskPools {
        &self.inner.task_pools
    }

    pub fn task_pool(&self, kind: TaskPoolKind) -> &TaskPool {
        self.inner.task_pools.get(kind)
    }

    pub fn task_pool_report(&self) -> TaskPoolReport {
        self.handle().task_pool_report()
    }

    pub fn time_clocks(&self) -> RuntimeTimeClocks {
        self.handle().time_clocks()
    }

    pub fn real_time(&self) -> Time<Real> {
        self.handle().real_time()
    }

    pub fn virtual_time(&self) -> Time<Virtual> {
        self.handle().virtual_time()
    }

    pub fn fixed_time(&self) -> Time<Fixed> {
        self.handle().fixed_time()
    }

    pub fn advance_time_by(
        &self,
        real_delta: Duration,
        max_fixed_steps: u32,
    ) -> RuntimeTimeAdvance {
        self.handle().advance_time_by(real_delta, max_fixed_steps)
    }

    pub fn tick_time(&self, max_fixed_steps: u32) -> RuntimeTimeAdvance {
        self.handle().tick_time(max_fixed_steps)
    }

    pub fn pause_virtual_time(&self) {
        self.handle().pause_virtual_time();
    }

    pub fn unpause_virtual_time(&self) {
        self.handle().unpause_virtual_time();
    }

    pub fn set_virtual_time_max_delta(&self, max_delta: Duration) {
        self.handle().set_virtual_time_max_delta(max_delta);
    }

    pub fn set_virtual_time_relative_speed_f64(&self, speed: f64) {
        self.handle().set_virtual_time_relative_speed_f64(speed);
    }

    pub fn set_fixed_timestep(&self, timestep: Duration) {
        self.handle().set_fixed_timestep(timestep);
    }

    pub fn diagnostic_store(&self) -> DiagnosticStore {
        self.handle().diagnostic_store()
    }

    pub fn diagnostic_store_snapshot(&self) -> DiagnosticStoreSnapshot {
        self.handle().diagnostic_store_snapshot()
    }

    pub fn record_diagnostic<U, T>(
        &self,
        path: impl Into<DiagnosticPath>,
        frame_index: u64,
        value: f64,
        unit: Option<U>,
        subsystem_tags: impl IntoIterator<Item = T>,
    ) where
        U: Into<String>,
        T: Into<String>,
    {
        self.handle()
            .record_diagnostic(path, frame_index, value, unit, subsystem_tags);
    }

    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        self.handle().register_module(descriptor)
    }

    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle().activate_module(module_name)
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle().deactivate_module(module_name)
    }

    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle().resolve_driver(name)
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle().resolve_manager(name)
    }

    pub fn publish_event(&self, topic: impl Into<String>, payload: Value) {
        self.handle().publish_event(topic, payload)
    }

    pub fn subscribe_events(&self, topic: impl Into<String>) -> ChannelReceiver<EngineEvent> {
        self.handle().subscribe_events(topic)
    }

    pub fn store_config_value(&self, key: impl Into<String>, value: Value) {
        self.handle().store_config_value(key, value)
    }

    pub fn load_config_value(&self, key: &str) -> Option<Value> {
        self.handle().load_config_value(key)
    }

    pub fn snapshot_config_values(&self) -> HashMap<String, Value> {
        self.handle().snapshot_config_values()
    }

    pub fn load_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        self.handle().load_config(key)
    }

    pub fn install_scene_runtime_hooks(
        &self,
        extensions: &RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.handle().install_scene_runtime_hooks(extensions)
    }

    pub fn init_state<T>(&self) -> StateTransitionEvent<T>
    where
        T: StateSpec + Default,
    {
        self.handle().init_state::<T>()
    }

    pub fn insert_state<T: StateSpec>(&self, state: T) -> StateTransitionEvent<T> {
        self.handle().insert_state(state)
    }

    pub fn state<T: StateSpec>(&self) -> Option<State<T>> {
        self.handle().state::<T>()
    }

    pub fn next_state<T: StateSpec>(&self) -> NextState<T> {
        self.handle().next_state::<T>()
    }

    pub fn set_next_state<T: StateSpec>(&self, state: T) {
        self.handle().set_next_state(state);
    }

    pub fn set_next_state_if_neq<T: StateSpec>(&self, state: T) {
        self.handle().set_next_state_if_neq(state);
    }

    pub fn reset_next_state<T: StateSpec>(&self) {
        self.handle().reset_next_state::<T>();
    }

    pub fn apply_state_transition<T: StateSpec>(&self) -> Option<StateTransitionEvent<T>> {
        self.handle().apply_state_transition::<T>()
    }

    pub fn state_transition_events<T: StateSpec>(&self) -> Vec<StateTransitionEvent<T>> {
        self.handle().state_transition_events::<T>()
    }

    pub fn register_on_enter<T, F>(&self, label: OnEnter<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.handle().register_on_enter(label, hook);
    }

    pub fn register_on_exit<T, F>(&self, label: OnExit<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.handle().register_on_exit(label, hook);
    }

    pub fn register_on_transition<T, F>(&self, label: OnTransition<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.handle().register_on_transition(label, hook);
    }
}

impl Default for CoreRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CoreRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreRuntime").finish()
    }
}
