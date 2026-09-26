//! 应用级生命周期事实，供平台驱动发布、管理端观察。
//! 激活状态和表面可用性是独立维度；挂起完成仍须等待表面租约退役。

mod activation_state;
mod generation;
mod operation;
mod operation_id;
mod snapshot;
mod state;
mod surface_availability;
mod terminal_result;

pub use activation_state::ApplicationActivationState;
pub use generation::ApplicationLifecycleGeneration;
pub use operation::ApplicationLifecycleOperation;
pub use operation_id::ApplicationLifecycleOperationId;
pub use snapshot::ApplicationLifecycleSnapshot;
pub use state::ApplicationLifecycleState;
pub use surface_availability::ApplicationSurfaceAvailability;
pub use terminal_result::ApplicationLifecycleTerminalResult;
