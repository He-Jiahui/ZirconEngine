use crate::view::ViewDescriptor;

use super::assets_view_descriptor::assets_view_descriptor;
use super::console_view_descriptor::console_view_descriptor;
use super::game_view_descriptor::game_view_descriptor;
use super::hierarchy_view_descriptor::hierarchy_view_descriptor;
use super::inspector_view_descriptor::inspector_view_descriptor;
use super::project_view_descriptor::project_view_descriptor;
use super::scene_view_descriptor::scene_view_descriptor;

pub(in crate::host::manager::builtin_views) fn activity_view_descriptors() -> Vec<ViewDescriptor> {
    vec![
        project_view_descriptor(),
        hierarchy_view_descriptor(),
        inspector_view_descriptor(),
        scene_view_descriptor(),
        game_view_descriptor(),
        assets_view_descriptor(),
        console_view_descriptor(),
    ]
}
