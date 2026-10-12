//! 运行时内置模块组合保持核心脊柱顺序，外部插件按必需性报告缺失。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "builtin_modules/core_spine.rs"]
#[cfg(all(feature = "graphics", feature = "script", feature = "ui"))]
mod core_spine;
#[path = "builtin_modules/plugin_selection.rs"]
mod plugin_selection;
#[path = "builtin_modules/split_layout.rs"]
mod split_layout;
