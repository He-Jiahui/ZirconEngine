//! 性能热点的结构守卫核对拥有者、文件预算和证据文档。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "../diagnostics/artifact_cache_payload.rs"]
mod artifact_cache_payload;
#[path = "../diagnostics/render_product_diagnostics.rs"]
mod render_product_diagnostics;
#[path = "diagnostics/split_layout.rs"]
mod split_layout;
