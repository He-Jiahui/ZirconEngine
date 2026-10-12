#[path = "panes/animation.rs"]
mod animation;
#[path = "panes/basic.rs"]
mod basic;
#[path = "panes/build_export.rs"]
mod build_export;
#[path = "panes/hierarchy.rs"]
mod hierarchy;
#[path = "panes/inspector.rs"]
mod inspector;
#[path = "panes/module_plugins.rs"]
mod module_plugins;
#[path = "panes/pane.rs"]
mod pane;
#[path = "panes/performance_timeline.rs"]
mod performance_timeline;
#[path = "panes/runtime_diagnostics.rs"]
mod runtime_diagnostics;
#[path = "panes/viewport.rs"]
mod viewport;

pub(crate) use animation::*;
pub(crate) use basic::*;
pub(crate) use build_export::*;
pub(crate) use hierarchy::*;
pub(crate) use inspector::*;
pub(crate) use module_plugins::*;
pub(crate) use pane::*;
pub(crate) use performance_timeline::*;
pub(crate) use runtime_diagnostics::*;
pub(crate) use viewport::*;
