//! 资产管线吸收后，门禁与镜像文档共同约束管理器、工作池和产物路径。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "asset_pipeline/cargo_gate.rs"]
mod cargo_gate;
#[path = "asset_pipeline/inventory.rs"]
mod inventory;
#[path = "asset_pipeline/mirror_docs.rs"]
mod mirror_docs;
#[path = "asset_pipeline/split_layout.rs"]
mod split_layout;
#[path = "asset_pipeline/support.rs"]
mod support;
