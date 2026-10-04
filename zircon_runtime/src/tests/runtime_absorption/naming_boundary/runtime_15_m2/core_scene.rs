//! 命名策略扫描需区分生产源码、测试夹具和已分类的历史名称。集中挂载下级测试；所有行为断言留在被挂载模块。
use std::path::Path;

use super::super::support::{assert_contains_all, read_repo_text, read_text};

#[path = "core_scene/core_runtime_state.rs"]
mod core_runtime_state;
#[path = "core_scene/render_contracts.rs"]
mod render_contracts;
#[path = "core_scene/render_layer_schema_v1.rs"]
mod render_layer_schema_v1;
#[path = "core_scene/scene_ecs_owners.rs"]
mod scene_ecs_owners;
