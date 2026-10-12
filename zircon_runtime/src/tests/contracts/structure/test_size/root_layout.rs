//! 为测试目录根布局挂载专属检查与共享清单；此入口只划分职责，实际断言由子模块及测试框架承担。
use super::*;

#[path = "../../../structure/test_size/root_layout/folder_backed.rs"]
mod folder_backed;
#[path = "../../../structure/test_size/root_layout/module_layout.rs"]
mod module_layout;
#[path = "../../../structure/test_size/root_layout/ui_children.rs"]
mod ui_children;
