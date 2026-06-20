mod builder;
mod model;

pub(crate) use builder::{
    build_world_space_ui_surface_submissions,
    build_world_space_ui_surface_submissions_from_host_scene,
};
pub(crate) use model::WorldSpaceUiSurfaceSubmission;

#[cfg(test)]
mod tests;
