#[path = "action_factory.rs"]
mod action_factory;
#[path = "console.rs"]
mod console;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "placeholder.rs"]
mod placeholder;
#[path = "project.rs"]
mod project;
#[path = "resolve.rs"]
mod resolve;
#[path = "scene.rs"]
mod scene;

pub(crate) use resolve::empty_state_for_tab;
