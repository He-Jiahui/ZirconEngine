use super::{RuntimePluginDescriptor, RuntimePluginDescriptorBuilder};

mod advanced_rendering;
mod asset_rows;
mod augmentation;
mod capability_status;
mod classification;
mod core_classification;
mod core_rows;
mod importer_classification;
mod language;
mod net_features;
mod optional_features;
mod particles_features;
mod render_classification;
mod render_rows;
mod rendering_features;
mod rows;
mod sound_features;

use augmentation::augment_descriptor;
use classification::classify_descriptor;
use optional_features::attach_optional_features;
use rows::builtin_catalog_rows;

type BuiltinCatalogDescriptorBuilder = RuntimePluginDescriptorBuilder;

impl RuntimePluginDescriptor {
    pub fn builtin_catalog() -> Vec<Self> {
        builtin_catalog_rows()
            .map(|row| {
                Self::builder(
                    row.package_id,
                    row.display_name,
                    row.runtime_id.clone(),
                    row.runtime_crate,
                )
                .with_target_modes(row.target_modes.iter().copied())
                .with_capability(row.capability)
            })
            .map(augment_descriptor)
            .map(attach_optional_features)
            .map(classify_descriptor)
            .map(RuntimePluginDescriptorBuilder::build)
            .collect()
    }
}
