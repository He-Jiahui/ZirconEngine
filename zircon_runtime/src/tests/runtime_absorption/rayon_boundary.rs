//! 直接并行库调用只由核心任务原语拥有，生产扫描负责报告越界引用。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "rayon_boundary/cutover_status.rs"]
mod cutover_status;
#[path = "rayon_boundary/production_scan.rs"]
mod production_scan;
#[path = "rayon_boundary/split_layout.rs"]
mod split_layout;
#[path = "rayon_boundary/support.rs"]
mod support;
