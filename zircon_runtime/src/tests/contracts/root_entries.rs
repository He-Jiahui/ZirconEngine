//! 运行时根入口只公开选定模块，模块家族和历史别名由各自子模块判定。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "root_entries/core_spine.rs"]
mod core_spine;
#[path = "root_entries/module_families.rs"]
mod module_families;
#[path = "root_entries/runtime_root.rs"]
mod runtime_root;
