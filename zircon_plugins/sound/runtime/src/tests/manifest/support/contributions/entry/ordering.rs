mod dependencies;
mod event_catalogs;
mod modules;

use super::super::StaticSoundContributions;

pub(super) fn sort_static_sound_contributions(contributions: &mut StaticSoundContributions) {
    dependencies::sort_static_dependencies(&mut contributions.dependencies);
    event_catalogs::sort_static_event_catalogs(&mut contributions.event_catalogs);
    modules::sort_static_modules(&mut contributions.modules);
}
