//! 原生窗口到视口表面的绑定权限：先准备图形资源，再发布可路由租约，停用后完成退役。
//! PlatformDriver 串行化状态转换；租约本身不拥有原生窗口或图形对象。

#[path = "error.rs"]
mod error;
#[path = "generation.rs"]
mod generation;
#[path = "lease.rs"]
mod lease;
#[path = "registry.rs"]
mod registry;
#[path = "request.rs"]
mod request;
#[path = "retirement_plan.rs"]
mod retirement_plan;

pub use error::SurfaceLeaseError;
pub use generation::SurfaceLeaseGeneration;
pub use lease::{PreparedSurfaceLease, SurfaceLease, SurfaceLeasePublication};
pub use registry::SurfaceLeaseRegistry;
pub use request::SurfaceLeaseRequest;
pub(crate) use retirement_plan::SurfaceLeaseRetirementPlan;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
