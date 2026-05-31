use zircon_runtime::plugin::PluginPackageManifest;

use crate::components::sound_component_descriptors;

use super::dependencies::sound_dependencies;
use super::events::sound_event_catalogs;
use super::options::sound_options;

pub fn attach_sound_manifest_contributions(
    manifest: PluginPackageManifest,
) -> PluginPackageManifest {
    sound_component_descriptors().into_iter().fold(
        sound_event_catalogs().into_iter().fold(
            sound_options().into_iter().fold(
                sound_dependencies()
                    .into_iter()
                    .fold(manifest, |manifest, dependency| {
                        manifest.with_dependency(dependency)
                    }),
                |manifest, option| manifest.with_option(option),
            ),
            |manifest, event_catalog| manifest.with_event_catalog(event_catalog),
        ),
        |manifest, component| manifest.with_component(component),
    )
}
