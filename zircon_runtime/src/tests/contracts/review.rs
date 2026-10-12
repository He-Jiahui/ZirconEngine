//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "review/f12_dead_code.rs"]
mod f12_dead_code;
#[path = "review/f8_api_convergence.rs"]
mod f8_api_convergence;
#[path = "review/late_api_cleanup.rs"]
mod late_api_cleanup;
#[path = "review/p0_robustness.rs"]
mod p0_robustness;
#[path = "review/plugin_importer_dx.rs"]
mod plugin_importer_dx;
#[path = "review/render_structure.rs"]
mod render_structure;
#[path = "review/status_placeholder_guard.rs"]
mod status_placeholder_guard;
#[path = "review/errors/mod.rs"]
mod typed_error_convergence;
