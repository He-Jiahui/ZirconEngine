mod material;
mod model;
mod scene;

use crate::asset::{AssetImportOutcome, ImportedAsset};

/// First-wave typed extraction stays explicit until the generic reflection plan lands.
pub(crate) fn append_handwritten_dependencies(outcome: &mut AssetImportOutcome) {
    for entry in &mut outcome.entries {
        for dependency in handwritten_dependencies(&entry.asset) {
            if !entry.dependencies.contains(&dependency) {
                entry.dependencies.push(dependency);
            }
        }
    }
}

pub(crate) fn handwritten_dependencies(asset: &ImportedAsset) -> Vec<crate::asset::AssetUri> {
    let references = match asset {
        ImportedAsset::Scene(asset) => scene::extract(asset),
        ImportedAsset::Material(asset) => material::extract(asset),
        ImportedAsset::Model(asset) => model::extract(asset),
        _ => Vec::new(),
    };
    references
        .into_iter()
        .map(|reference| reference.locator)
        .collect()
}
