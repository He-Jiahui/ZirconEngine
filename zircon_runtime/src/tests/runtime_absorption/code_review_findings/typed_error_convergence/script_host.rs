//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "script_host/gameplay_scene.rs"]
mod gameplay_scene;
#[path = "script_host/host_reflection_docs.rs"]
mod host_reflection_docs;
#[path = "script_host/plugin_management.rs"]
mod plugin_management;
