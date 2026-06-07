mod dependencies;
mod entry;
mod event_catalogs;
mod modules;
mod types;

pub(super) use entry::static_sound_contributions;
pub(in crate::tests::manifest) use types::StaticSoundContributions;
use types::{StaticDependency, StaticEventCatalog, StaticModule};
