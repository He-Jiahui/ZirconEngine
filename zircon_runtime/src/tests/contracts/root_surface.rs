//! 运行时根公开面和图形别名的收敛结果需与架构文档一致。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "root_surface/docs.rs"]
mod docs;
#[path = "root_surface/graphics_alias.rs"]
mod graphics_alias;
#[path = "root_surface/inventory.rs"]
mod inventory;
#[path = "root_surface/public_surface.rs"]
mod public_surface;
#[path = "root_surface/split_layout.rs"]
mod split_layout;
