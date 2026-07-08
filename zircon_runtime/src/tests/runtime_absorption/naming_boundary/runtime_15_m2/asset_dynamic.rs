use std::path::Path;

use super::super::support::{assert_contains_all, read_repo_text, read_text};

#[path = "asset_dynamic/asset_watch.rs"]
mod asset_watch;
#[path = "asset_dynamic/dynamic_api_vampire.rs"]
mod dynamic_api_vampire;
#[path = "asset_dynamic/scene_ecs_queries.rs"]
mod scene_ecs_queries;
#[path = "asset_dynamic/texture_containers.rs"]
mod texture_containers;
