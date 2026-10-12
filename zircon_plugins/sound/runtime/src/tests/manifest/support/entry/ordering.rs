#[path = "../../../support/entry/ordering/dependencies.rs"]
mod dependencies;
#[path = "../../../support/entry/ordering/event_catalogs.rs"]
mod event_catalogs;
#[path = "../../../support/entry/ordering/modules.rs"]
mod modules;

use super::super::StaticSoundContributions;

pub(super) fn sort_static_sound_contributions(contributions: &mut StaticSoundContributions) {
    dependencies::sort_static_dependencies(&mut contributions.dependencies);
    event_catalogs::sort_static_event_catalogs(&mut contributions.event_catalogs);
    modules::sort_static_modules(&mut contributions.modules);
}
