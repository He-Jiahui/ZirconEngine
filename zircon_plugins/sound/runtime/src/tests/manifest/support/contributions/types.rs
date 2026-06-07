mod dependency;
mod event_catalog;
mod module;
mod static_contributions;

pub(super) use dependency::StaticDependency;
pub(super) use event_catalog::StaticEventCatalog;
pub(super) use module::StaticModule;
pub(in crate::tests::manifest) use static_contributions::StaticSoundContributions;
