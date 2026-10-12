//! 任务系统的执行模型、并行来源与文档门禁保持同一口径。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "job_system/inventory.rs"]
mod inventory;
#[path = "job_system/mirror_docs.rs"]
mod mirror_docs;
#[path = "job_system/source_helpers.rs"]
mod source_helpers;
#[path = "job_system/split_layout.rs"]
mod split_layout;
