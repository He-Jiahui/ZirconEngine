//! 场景吸收、命名空间与内部反射边界的结构回归入口；源码读取用于约束当前公开面。
#[path = "module_registration.rs"]
mod module_registration;
#[path = "namespace_surface.rs"]
mod namespace_surface;
#[path = "property_reflection.rs"]
mod property_reflection;
#[path = "runtime_orchestration.rs"]
mod runtime_orchestration;
