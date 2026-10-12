#[path = "../types/dependency.rs"]
mod dependency;
#[path = "../types/event_catalog.rs"]
mod event_catalog;
#[path = "../types/module.rs"]
mod module;
#[path = "../../../support/types/static_contributions.rs"]
mod static_contributions;

pub(super) use dependency::StaticDependency;
pub(super) use event_catalog::StaticEventCatalog;
pub(super) use module::StaticModule;
pub(in crate::tests::manifest) use static_contributions::StaticSoundContributions;
