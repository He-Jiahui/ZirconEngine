use std::path::Path;

use super::super::support::{
    assert_contains_all, read_repo_text, read_runtime_15_naming_date_map,
    read_runtime_15_naming_status_map, read_runtime_15_naming_status_rows, read_text,
};

#[path = "core_scene/core_runtime_state.rs"]
mod core_runtime_state;
#[path = "core_scene/render_contracts.rs"]
mod render_contracts;
#[path = "core_scene/render_layer_schema_v1.rs"]
mod render_layer_schema_v1;
#[path = "core_scene/scene_ecs_owners.rs"]
mod scene_ecs_owners;
