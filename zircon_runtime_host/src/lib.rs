//! Host-side safety adapters for runtime-owned ABI outputs and lifecycle state.
//!
//! Host 侧负责运行时 ABI 输出安全适配与生命周期状态。
//!
//! 本 crate 集中管理两类跨边界状态：[`foreign_output`] 校验、限额解码并释放 Runtime
//! 返回的拥有型输出；[`viewport_surface`] 在 App 与 Editor 调用 ABI 期间登记视口表面绑定。
//! 这些类型由 App 的 `runtime_session` 和 Editor 的 `core::gateway::session` 调用。

#[path = "foreign_output/mod.rs"]
pub mod foreign_output;
#[path = "viewport_surface.rs"]
pub mod viewport_surface;
