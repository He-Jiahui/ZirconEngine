use super::{RuntimePluginDescriptor, RuntimePluginDescriptorBuilder};

#[path = "builtin_catalog/advanced_rendering.rs"]
mod advanced_rendering;
#[path = "builtin_catalog/asset_rows.rs"]
mod asset_rows;
#[path = "builtin_catalog/augmentation.rs"]
mod augmentation;
#[path = "builtin_catalog/capability_status.rs"]
mod capability_status;
#[path = "builtin_catalog/classification.rs"]
mod classification;
#[path = "builtin_catalog/core_classification.rs"]
mod core_classification;
#[path = "builtin_catalog/core_rows.rs"]
mod core_rows;
#[path = "builtin_catalog/importer_classification.rs"]
mod importer_classification;
#[path = "builtin_catalog/language.rs"]
mod language;
#[path = "builtin_catalog/net_features.rs"]
mod net_features;
#[path = "builtin_catalog/optional_features.rs"]
mod optional_features;
#[path = "builtin_catalog/particles_features.rs"]
mod particles_features;
#[path = "builtin_catalog/render_classification.rs"]
mod render_classification;
#[path = "builtin_catalog/render_rows.rs"]
mod render_rows;
#[path = "builtin_catalog/rendering_features.rs"]
mod rendering_features;
#[path = "builtin_catalog/rows.rs"]
mod rows;
#[path = "builtin_catalog/sound_features.rs"]
mod sound_features;

use augmentation::augment_descriptor;
use classification::classify_descriptor;
use optional_features::attach_optional_features;
use rows::builtin_catalog_rows;

type BuiltinCatalogDescriptorBuilder = RuntimePluginDescriptorBuilder;
type IdentifiedBuiltinCatalogDescriptorBuilder = (&'static str, BuiltinCatalogDescriptorBuilder);

impl RuntimePluginDescriptor {
    /// 构造内建包的静态目录：先补充能力与可选特性，再分类状态，供目录构造器一次性生成初始代。
    pub fn builtin_catalog() -> Vec<Self> {
        builtin_catalog_rows()
            .map(|row| {
                (
                    row.package_id,
                    Self::builder(
                        row.package_id,
                        row.display_name,
                        row.runtime_id.clone(),
                        row.runtime_crate,
                    )
                    .with_target_modes(row.target_modes.iter().copied())
                    .with_capability(row.capability),
                )
            })
            .map(augment_descriptor)
            .map(attach_optional_features)
            .map(classify_descriptor)
            .map(|(_, descriptor)| descriptor)
            .map(RuntimePluginDescriptorBuilder::build)
            .collect()
    }
}
