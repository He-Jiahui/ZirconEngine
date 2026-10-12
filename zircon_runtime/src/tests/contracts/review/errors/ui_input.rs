//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "ui_input/surface_effects.rs"]
mod surface_effects;
#[path = "ui_input/surrounding_text.rs"]
mod surrounding_text;
