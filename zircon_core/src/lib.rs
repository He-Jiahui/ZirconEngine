//! Core runtime, lifecycle management, service registry, and shared runtime primitives.

mod channel_util;
mod config_store;
mod error;
mod event_bus;
mod frame_clock;
mod job_scheduler;
mod lifecycle;
mod runtime;
mod types;

pub use channel_util::{recv_latest, spawn_named_thread, wait_for};
pub use config_store::ConfigStore;
pub use error::{CoreError, ZirconError};
pub use event_bus::{EngineEvent, EventBus};
pub use frame_clock::FrameClock;
pub use job_scheduler::JobScheduler;
pub use lifecycle::{LifecycleState, ServiceKind, StartupMode};
pub use runtime::{
    CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, ManagerDescriptor,
    ModuleContext, ModuleDescriptor, PluginContext, PluginDescriptor, RegistryName, ServiceFactory,
};
pub use types::{ChannelReceiver, ChannelSender, ServiceObject};
