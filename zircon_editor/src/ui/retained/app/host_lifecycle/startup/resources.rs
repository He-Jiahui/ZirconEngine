#[path = "resources/bundle.rs"]
mod bundle;
#[path = "resources/events.rs"]
mod events;
#[path = "resources/managers.rs"]
mod managers;

pub(super) use bundle::StartupManagers;
pub(super) use managers::resolve_startup_managers;
