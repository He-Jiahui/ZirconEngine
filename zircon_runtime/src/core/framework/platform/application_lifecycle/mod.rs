//! 应用级生命周期事实，供平台驱动发布、管理端观察。
//! 激活状态和表面可用性是独立维度；挂起完成仍须等待表面租约退役。

#[path = "../../../platform/application_lifecycle/activation_state.rs"]
mod activation_state;
#[path = "generation.rs"]
mod generation;
#[path = "operation.rs"]
mod operation;
#[path = "operation_id.rs"]
mod operation_id;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "state.rs"]
mod state;
#[path = "../../../platform/application_lifecycle/surface_availability.rs"]
mod surface_availability;
#[path = "../../../platform/application_lifecycle/terminal_result.rs"]
mod terminal_result;

pub use activation_state::ApplicationActivationState;
pub use generation::ApplicationLifecycleGeneration;
pub use operation::ApplicationLifecycleOperation;
pub use operation_id::ApplicationLifecycleOperationId;
pub use snapshot::ApplicationLifecycleSnapshot;
pub use state::ApplicationLifecycleState;
pub use surface_availability::ApplicationSurfaceAvailability;
pub use terminal_result::ApplicationLifecycleTerminalResult;
