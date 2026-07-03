use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

#[path = "core_framework/camera_controller.rs"]
mod camera_controller;
#[path = "core_framework/render_fixtures.rs"]
mod render_fixtures;
#[path = "core_framework/render_layer_schema_v1.rs"]
mod render_layer_schema_v1;
