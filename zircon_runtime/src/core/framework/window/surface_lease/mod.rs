//! 原生窗口到视口表面的绑定权限：先准备图形资源，再发布可路由租约，停用后完成退役。
//! PlatformDriver 串行化状态转换；租约本身不拥有原生窗口或图形对象。

mod error;
mod generation;
mod lease;
mod registry;
mod request;
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
