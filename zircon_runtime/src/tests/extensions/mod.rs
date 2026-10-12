//! 外置插件切换的结构回归入口；文件与清单检查约束职责归属，具体管理器实现及运行行为仍由各领域的行为测试验证。
#[path = "absorption_surface.rs"]
mod absorption_surface;
#[path = "animation_physics_absorption.rs"]
mod animation_physics_absorption;
#[path = "manager_handles.rs"]
mod manager_handles;
#[path = "root_entries.rs"]
mod root_entries;
#[path = "tech_stack_dependency_guard.rs"]
mod tech_stack_dependency_guard;
