//! Core runtime, lifecycle management, service registry, and shared runtime primitives.

mod channel_util;
mod config_store;
mod error;
mod event_bus;
mod frame_clock;
mod job_scheduler;
mod lifecycle;
pub mod runtime;
pub mod tasks;
mod time;
mod types;

pub mod diagnostics;
pub mod framework;
pub mod manager;
pub mod math;
pub mod modules;
pub mod resource;
pub mod state;

pub use channel_util::{recv_latest, spawn_named_thread, wait_for};
pub use config_store::ConfigStore;
pub use error::{CoreError, ZirconError};
pub use event_bus::{EngineEvent, EventBus};
pub use frame_clock::FrameClock;
pub use job_scheduler::JobScheduler;
pub use lifecycle::{LifecycleState, ServiceKind, StartupMode};
pub use runtime::{
    CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, ManagerDescriptor,
    ModuleContext, ModuleDescriptor, PluginContext, PluginDescriptor, PluginFactory, RegistryName,
    ServiceFactory,
};
pub use state::{NextState, OnEnter, OnExit, OnTransition, State, StateSpec, StateTransitionEvent};
pub use tasks::{
    TaskPool, TaskPoolDescriptor, TaskPoolKind, TaskPoolOptions, TaskPoolReport,
    TaskPoolReportEntry, TaskPoolThreadAssignmentPolicy, TaskPoolThreadCounts, TaskPools,
};
pub use time::{
    RuntimeTimeAdvance, RuntimeTimeClocks, TIME_FIXED_STEPS_DIAGNOSTIC, TIME_FPS_DIAGNOSTIC,
    TIME_FRAME_COUNT_DIAGNOSTIC, TIME_FRAME_TIME_DIAGNOSTIC,
};
pub use types::{ChannelReceiver, ChannelSender, ServiceObject};
