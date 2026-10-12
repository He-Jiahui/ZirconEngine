//! 运行时界面结构和历史命名迁移需遵守架构与文档边界。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "ui_architecture/architecture_boundaries.rs"]
mod architecture_boundaries;
#[path = "ui_architecture/legacy_renames.rs"]
mod legacy_renames;
#[path = "ui_architecture/mirror_docs.rs"]
mod mirror_docs;
#[path = "ui_architecture/split_layout.rs"]
mod split_layout;
#[path = "ui_architecture/support.rs"]
mod support;
