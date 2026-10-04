//! 核心脊柱、根公开面和生成模板的结构清单需与审计证据同步。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "core_spine_root_generated/generated_behavior.rs"]
mod generated_behavior;
#[path = "core_spine_root_generated/inventory.rs"]
mod inventory;
#[path = "core_spine_root_generated/mirror_docs.rs"]
mod mirror_docs;
#[path = "core_spine_root_generated/source_helpers.rs"]
mod source_helpers;
#[path = "core_spine_root_generated/split_layout.rs"]
mod split_layout;
