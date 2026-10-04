mod shutdown;

pub use shutdown::ModuleShutdownReport;

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde_json::Value;
use zr_contracts::random::{RandomServiceCheckpoint, RandomServiceState};

use crate::core::diagnostics::{
    DiagnosticPath, DiagnosticStore, DiagnosticStoreSnapshot, RuntimeDevtoolsPluginCatalogEntry,
};
use crate::core::framework::events::{
    EngineEventDeliveryPolicy, EngineEventSubscription, EventBusDiagnosticsSnapshot,
};
use crate::core::framework::time::{
    MonotonicReal, Time, TimePolicy, TimePolicyError, TimePolicyTransaction,
};
use crate::core::{CoreError, RuntimeModuleLifecycleObserver};

use super::handle::{CoreHandle, ServiceHandle};
use super::random::{RandomService, RandomServiceError};
use super::state::CoreRuntimeInner;
use super::state_machine::{
    NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent,
};
use super::tasks::{
    EngineTaskGraph, EngineTaskGraphInitError, EngineTaskGraphOptions, JobScheduler,
    TaskGraphAdmissionError, TaskGraphScope, TaskGraphScopeDescriptor, TaskGraphShutdownError,
    TaskGraphShutdownReport, TaskGraphWorkerInventory,
};
use super::time::{FrameTimeSnapshot, TimePolicyReceipt};
use super::weak::CoreWeak;
use super::{
    ClockDiscontinuity, ClockSource, FrameClock, FrameClockRebaseReceipt, ModuleDescriptor,
};

/// CoreHandle 的宿主外观；克隆值及 `handle()` 共享同一运行时状态与时钟。
#[derive(Clone)]
pub struct CoreRuntime {
    handle: CoreHandle,
}

impl CoreRuntime {
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|error| panic!("{error}"))
    }

    /// Creates a runtime with one worker set owned by this runtime instance.
    pub fn try_new() -> Result<Self, EngineTaskGraphInitError> {
        Self::try_with_task_graph_options(EngineTaskGraphOptions::default())
    }

    /// Creates a runtime with an explicit worker budget owned by this runtime
    /// instance. Hosts that may run more than one runtime must resolve and
    /// pass their intended budget instead of relying on process-global pools.
    pub fn try_with_task_graph_options(
        task_graph_options: EngineTaskGraphOptions,
    ) -> Result<Self, EngineTaskGraphInitError> {
        Self::try_with_frame_clock_and_random_service_and_task_graph_options(
            FrameClock::default(),
            RandomService::default(),
            task_graph_options,
        )
    }

    /// Creates a runtime with an explicit deterministic master seed.
    pub fn with_random_seed(master_seed: u64) -> Self {
        Self::with_frame_clock_and_random_service(
            FrameClock::default(),
            RandomService::new(master_seed),
        )
    }

    /// Creates a runtime from seed authority with an empty random-stream registry.
    pub fn with_random_service_state(state: RandomServiceState) -> Self {
        Self::with_frame_clock_and_random_service(
            FrameClock::default(),
            RandomService::from_state(state),
        )
    }

    /// Creates a runtime from seed authority and registered random-stream progress.
    pub fn with_random_service_checkpoint(
        checkpoint: RandomServiceCheckpoint,
    ) -> Result<Self, RandomServiceError> {
        Ok(Self::with_frame_clock_and_random_service(
            FrameClock::default(),
            RandomService::from_checkpoint(checkpoint)?,
        ))
    }

    /// Creates a runtime whose outer-frame clock receives samples from `clock_source`.
    ///
    /// The source affects only the authoritative frame delta. It does not alter
    /// task deadlines, file watchers, telemetry, or profiling clocks.
    pub fn with_clock_source(clock_source: Arc<dyn ClockSource>) -> Self {
        Self::with_frame_clock(FrameClock::with_clock_source(clock_source))
    }

    /// Creates a runtime with explicit outer-frame and deterministic random sources.
    pub fn with_clock_source_and_random_seed(
        clock_source: Arc<dyn ClockSource>,
        master_seed: u64,
    ) -> Self {
        Self::with_frame_clock_and_random_service(
            FrameClock::with_clock_source(clock_source),
            RandomService::new(master_seed),
        )
    }

    /// Creates a runtime from explicit outer-frame and seed-only random authority.
    pub fn with_clock_source_and_random_service_state(
        clock_source: Arc<dyn ClockSource>,
        state: RandomServiceState,
    ) -> Self {
        Self::with_frame_clock_and_random_service(
            FrameClock::with_clock_source(clock_source),
            RandomService::from_state(state),
        )
    }

    /// Creates a runtime from explicit outer-frame and complete random-stream progress.
    pub fn with_clock_source_and_random_service_checkpoint(
        clock_source: Arc<dyn ClockSource>,
        checkpoint: RandomServiceCheckpoint,
    ) -> Result<Self, RandomServiceError> {
        Ok(Self::with_frame_clock_and_random_service(
            FrameClock::with_clock_source(clock_source),
            RandomService::from_checkpoint(checkpoint)?,
        ))
    }

    fn with_frame_clock(frame_clock: FrameClock) -> Self {
        Self::with_frame_clock_and_random_service(frame_clock, RandomService::default())
    }

    fn with_frame_clock_and_random_service(
        frame_clock: FrameClock,
        random_service: RandomService,
    ) -> Self {
        Self::try_with_frame_clock_and_random_service_and_task_graph_options(
            frame_clock,
            random_service,
            EngineTaskGraphOptions::default(),
        )
        .unwrap_or_else(|error| panic!("{error}"))
    }

    fn try_with_frame_clock_and_random_service_and_task_graph_options(
        frame_clock: FrameClock,
        random_service: RandomService,
        task_graph_options: EngineTaskGraphOptions,
    ) -> Result<Self, EngineTaskGraphInitError> {
        let task_graph = EngineTaskGraph::try_new(task_graph_options)?;
        let inner = Arc::new(CoreRuntimeInner::new(
            frame_clock,
            random_service,
            task_graph,
        ));
        Ok(Self {
            handle: CoreHandle { inner },
        })
    }

    pub fn handle(&self) -> CoreHandle {
        self.handle.clone()
    }

    pub fn weak(&self) -> CoreWeak {
        self.handle.downgrade()
    }

    pub fn scheduler(&self) -> &JobScheduler {
        self.handle.scheduler()
    }

    /// Returns the runtime-owned execution service for explicit scoped work.
    pub fn task_graph(&self) -> &EngineTaskGraph {
        self.handle.task_graph()
    }

    /// Returns the worker budget owned by this runtime's execution service.
    pub fn task_graph_worker_inventory(&self) -> TaskGraphWorkerInventory {
        self.handle.task_graph_worker_inventory()
    }

    pub fn create_task_graph_scope(
        &self,
        descriptor: TaskGraphScopeDescriptor,
    ) -> Result<TaskGraphScope, TaskGraphAdmissionError> {
        self.task_graph().create_scope(descriptor)
    }

    /// Closes scoped task admission and waits for the scoped task census to
    /// quiesce. A failed result leaves the task graph closing.
    ///
    /// This is intentionally narrower than complete runtime teardown: process
    /// timers and remaining private workers are not yet graph-owned.
    pub fn shutdown_task_graph(
        &self,
        deadline: Duration,
    ) -> Result<TaskGraphShutdownReport, TaskGraphShutdownError> {
        self.task_graph().shutdown(deadline)
    }

    /// Continues task-graph teardown against an enclosing owner's absolute
    /// deadline instead of restarting a relative timeout at this boundary.
    pub(crate) fn shutdown_task_graph_until(
        &self,
        deadline: Instant,
    ) -> Result<TaskGraphShutdownReport, TaskGraphShutdownError> {
        self.task_graph().shutdown_until(deadline)
    }

    /// Returns this runtime instance's seed authority and unique stream registry.
    pub fn random_service(&self) -> &RandomService {
        self.handle.random_service()
    }

    pub fn real_time(&self) -> Time<MonotonicReal> {
        self.handle.real_time()
    }

    pub fn advance_time_by(&self, real_delta: Duration, max_fixed_steps: u32) -> FrameTimeSnapshot {
        self.handle.advance_time_by(real_delta, max_fixed_steps)
    }

    /// 从本 Runtime 的单调帧时钟取样；传入宿主增量时应调用 `advance_time_by`。
    pub fn tick_time(&self, max_fixed_steps: u32) -> FrameTimeSnapshot {
        self.handle.tick_time(max_fixed_steps)
    }

    pub(crate) fn rebase_frame_clock(&self) -> FrameClockRebaseReceipt {
        self.handle.rebase_frame_clock()
    }

    pub fn submit_clock_discontinuity(
        &self,
        discontinuity: ClockDiscontinuity,
    ) -> FrameClockRebaseReceipt {
        self.handle.submit_clock_discontinuity(discontinuity)
    }

    pub fn time_policy(&self) -> TimePolicy {
        self.handle.time_policy()
    }

    pub fn time_policy_generation(&self) -> u64 {
        self.handle.time_policy_generation()
    }

    pub fn apply_time_policy(
        &self,
        transaction: TimePolicyTransaction,
    ) -> Result<TimePolicyReceipt, TimePolicyError> {
        self.handle.apply_time_policy(transaction)
    }

    pub fn diagnostic_store(&self) -> DiagnosticStore {
        self.handle.diagnostic_store()
    }

    pub fn diagnostic_store_snapshot(&self) -> DiagnosticStoreSnapshot {
        self.handle.diagnostic_store_snapshot()
    }

    pub fn replace_devtools_plugin_catalog_entries(
        &self,
        entries: Vec<RuntimeDevtoolsPluginCatalogEntry>,
    ) {
        self.handle.replace_devtools_plugin_catalog_entries(entries);
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
        self.handle
            .record_diagnostic(path, frame_index, value, unit, subsystem_tags);
    }

    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        self.handle.register_module(descriptor)
    }

    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle.activate_module(module_name)
    }

    pub fn activate_module_with_ready_timeout(
        &self,
        module_name: &str,
        ready_timeout: Duration,
    ) -> Result<(), CoreError> {
        self.handle
            .activate_module_with_ready_timeout(module_name, ready_timeout)
    }

    pub fn activate_registered_modules(&self) -> Result<(), CoreError> {
        self.handle.activate_registered_modules()
    }

    pub fn activate_registered_modules_with_ready_timeout(
        &self,
        ready_timeout: Duration,
    ) -> Result<(), CoreError> {
        self.handle
            .activate_registered_modules_with_ready_timeout(ready_timeout)
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle.deactivate_module(module_name)
    }

    pub fn deactivate_module_with_drain_timeout(
        &self,
        module_name: &str,
        drain_timeout: Duration,
    ) -> Result<(), CoreError> {
        self.handle
            .deactivate_module_with_drain_timeout(module_name, drain_timeout)
    }

    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle.resolve_driver(name)
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle.resolve_manager(name)
    }

    pub fn resolve_driver_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.handle.resolve_driver_handle(name)
    }

    pub fn resolve_manager_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.handle.resolve_manager_handle(name)
    }

    pub fn resolve_plugin_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.handle.resolve_plugin_handle(name)
    }

    pub fn try_publish_event(
        &self,
        topic: impl Into<String>,
        payload: Value,
    ) -> Result<
        crate::core::framework::events::EngineEventPublishReceipt,
        crate::core::framework::events::EngineEventPublishRejected,
    > {
        self.handle.try_publish_event(topic, payload)
    }

    pub fn subscribe_events(
        &self,
        topic: impl Into<String>,
        policy: EngineEventDeliveryPolicy,
    ) -> Result<
        Box<dyn EngineEventSubscription>,
        crate::core::framework::events::EngineEventSubscribeError,
    > {
        self.handle.subscribe_events(topic, policy)
    }

    pub fn close_event_admission(&self) -> crate::core::framework::events::EventBusCloseReceipt {
        self.handle.close_event_admission()
    }
    pub fn event_bus_retention(&self) -> crate::core::framework::events::EventBusRetentionSnapshot {
        self.handle.event_bus_retention()
    }

    pub fn event_bus_diagnostics(&self) -> EventBusDiagnosticsSnapshot {
        self.handle.event_bus_diagnostics()
    }

    pub fn store_config_value(&self, key: impl Into<String>, value: Value) {
        self.handle.store_config_value(key, value)
    }

    pub fn load_config_value(&self, key: &str) -> Option<Value> {
        self.handle.load_config_value(key)
    }

    pub fn snapshot_config_values(&self) -> HashMap<String, Value> {
        self.handle.snapshot_config_values()
    }

    pub fn load_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        self.handle.load_config(key)
    }

    pub fn install_runtime_module_lifecycle_observer(
        &self,
        observer: Arc<dyn RuntimeModuleLifecycleObserver>,
    ) {
        self.handle
            .install_runtime_module_lifecycle_observer(observer);
    }

    pub fn init_state<T>(&self) -> StateTransitionEvent<T>
    where
        T: StateSpec + Default,
    {
        self.handle.init_state::<T>()
    }

    pub fn insert_state<T: StateSpec>(&self, state: T) -> StateTransitionEvent<T> {
        self.handle.insert_state(state)
    }

    pub fn state<T: StateSpec>(&self) -> Option<State<T>> {
        self.handle.state::<T>()
    }

    pub fn next_state<T: StateSpec>(&self) -> NextState<T> {
        self.handle.next_state::<T>()
    }

    pub fn set_next_state<T: StateSpec>(&self, state: T) {
        self.handle.set_next_state(state);
    }

    pub fn set_next_state_if_neq<T: StateSpec>(&self, state: T) {
        self.handle.set_next_state_if_neq(state);
    }

    pub fn reset_next_state<T: StateSpec>(&self) {
        self.handle.reset_next_state::<T>();
    }

    pub fn apply_state_transition<T: StateSpec>(&self) -> Option<StateTransitionEvent<T>> {
        self.handle.apply_state_transition::<T>()
    }

    pub fn latest_state_transition<T: StateSpec>(&self) -> Option<StateTransitionEvent<T>> {
        self.handle.latest_state_transition::<T>()
    }

    pub fn register_on_enter<T, F>(&self, label: OnEnter<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.handle.register_on_enter(label, hook);
    }

    pub fn register_on_exit<T, F>(&self, label: OnExit<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.handle.register_on_exit(label, hook);
    }

    pub fn register_on_transition<T, F>(&self, label: OnTransition<T>, hook: F)
    where
        T: StateSpec,
        F: Fn(&StateTransitionEvent<T>) + Send + Sync + 'static,
    {
        self.handle.register_on_transition(label, hook);
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

#[cfg(test)]
#[path = "tests/runtime_performance_tests.rs"]
mod performance_tests;
