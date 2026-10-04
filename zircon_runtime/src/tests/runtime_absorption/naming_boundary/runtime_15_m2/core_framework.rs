//! 命名策略扫描需区分生产源码、测试夹具和已分类的历史名称。集中挂载下级测试；所有行为断言留在被挂载模块。
use std::path::Path;

use super::super::support::{assert_contains_all, read_repo_text, read_text};

#[path = "core_framework/camera_controller.rs"]
mod camera_controller;
#[path = "core_framework/render_fixtures.rs"]
mod render_fixtures;
#[path = "core_framework/render_layer_schema_v1.rs"]
mod render_layer_schema_v1;
