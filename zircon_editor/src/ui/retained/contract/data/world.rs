#[path = "world/builder.rs"]
mod builder;
#[path = "world/model.rs"]
mod model;

#[cfg(test)]
use builder::build_world_space_ui_surface_submissions;
pub(crate) use builder::build_world_space_ui_surface_submissions_from_host_scene;
pub(crate) use model::WorldSpaceUiSurfaceSubmission;

#[cfg(test)]
#[path = "world/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/world_space_submission_perf.rs"]
mod performance_tests;
