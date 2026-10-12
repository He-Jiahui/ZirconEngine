//! 静态清单的能力命名空间、归属、状态与目标范围回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
#[path = "capabilities/bevy_references.rs"]
mod bevy_references;
#[path = "capabilities/namespaces.rs"]
mod namespaces;
#[path = "capabilities/owners.rs"]
mod owners;
#[path = "capabilities/status_rows.rs"]
mod status_rows;
#[path = "capabilities/targets.rs"]
mod targets;
#[path = "capabilities/traversal.rs"]
mod traversal;
