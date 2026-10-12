#[path = "build.rs"]
mod build;
#[path = "../../../region/tool_region/collapsed_constraints.rs"]
mod collapsed_constraints;
#[path = "presence.rs"]
mod presence;

pub(crate) use build::build_tool_region_state;
