#[path = "bridges/bundle.rs"]
mod bundle;
#[path = "bridges/factory.rs"]
mod factory;

pub(super) use bundle::StartupTemplateBridges;
pub(super) use factory::create_startup_template_bridges;
