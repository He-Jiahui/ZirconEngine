#[path = "../../../support/entry/collect/runtime_modules.rs"]
mod runtime_modules;
#[path = "../../../entry/collect/static_contributions.rs"]
mod static_contributions;

pub(super) use static_contributions::static_sound_contributions_from_plugin_toml;
