//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "shader_prewarm_cli/args_boundary.rs"]
mod args_boundary;
#[path = "shader_prewarm_cli/run_boundary.rs"]
mod run_boundary;
