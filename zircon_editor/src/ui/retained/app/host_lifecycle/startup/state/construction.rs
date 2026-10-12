#[path = "../../../startup/state/construction/assembly.rs"]
mod assembly;
#[path = "../../../startup/state/construction/input.rs"]
mod input;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) use assembly::construct_startup_host;
pub(in crate::ui::retained_host::app::host_lifecycle::startup) use input::StartupHostConstruction;
