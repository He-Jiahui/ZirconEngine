//! 命名策略扫描需区分生产源码、测试夹具和已分类的历史名称。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "naming_boundary/classifiers.rs"]
mod classifiers;
#[path = "naming_boundary/lexical_scan.rs"]
mod lexical_scan;
mod runtime_15_m2;
#[path = "naming_boundary/split_layout.rs"]
mod split_layout;
#[path = "naming_boundary/support.rs"]
mod support;
#[path = "naming_boundary/top_level.rs"]
mod top_level;
