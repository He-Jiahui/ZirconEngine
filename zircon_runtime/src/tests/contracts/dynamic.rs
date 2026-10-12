//! 动态场景会话、补丁预览和资产重载路径需保持现有只读与所有权约束。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "dynamic/asset_reload_selection_status.rs"]
mod asset_reload_selection_status;
#[path = "dynamic/patch_preview_api.rs"]
mod patch_preview_api;
#[path = "dynamic/patch_preview_behavior.rs"]
mod patch_preview_behavior;
#[path = "dynamic/patch_preview_status_docs.rs"]
mod patch_preview_status_docs;
#[path = "dynamic/session_capture_persistence.rs"]
mod session_capture_persistence;
#[path = "dynamic/session_load_query_path.rs"]
mod session_load_query_path;
#[path = "dynamic/session_retention_mutation_merge.rs"]
mod session_retention_mutation_merge;
#[path = "dynamic/sources.rs"]
mod sources;
#[path = "dynamic/split_layout.rs"]
mod split_layout;
