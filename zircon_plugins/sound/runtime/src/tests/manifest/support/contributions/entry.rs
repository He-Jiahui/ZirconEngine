mod collect;
mod ordering;

use super::StaticSoundContributions;

pub(in crate::tests::manifest::support) fn static_sound_contributions(
    manifest: &str,
) -> StaticSoundContributions {
    let mut contributions = collect::static_sound_contributions_from_plugin_toml(manifest);
    ordering::sort_static_sound_contributions(&mut contributions);
    contributions
}
