use std::collections::HashSet;

mod material;
mod model;
mod scene;

use crate::asset::{AssetImportOutcome, AssetUri, ImportedAsset};

#[cfg(test)]
#[path = "tests/dedup_index_tests.rs"]
mod dedup_index_tests;

/// First-wave typed extraction stays explicit until the generic reflection plan lands.
pub(crate) fn append_handwritten_dependencies(outcome: &mut AssetImportOutcome) {
    for entry in &mut outcome.entries {
        let dependencies = handwritten_dependencies(&entry.asset);
        append_unique_dependencies(&mut entry.dependencies, dependencies);
    }
}

fn append_unique_dependencies(dependencies: &mut Vec<AssetUri>, candidates: Vec<AssetUri>) {
    let mut known = HashSet::with_capacity(dependencies.len().saturating_add(candidates.len()));
    known.extend(dependencies.iter());
    let mut accepted = Vec::with_capacity(candidates.len());
    let mut accepted_count = 0;
    for dependency in &candidates {
        let is_new = known.insert(dependency);
        accepted.push(is_new);
        accepted_count += usize::from(is_new);
    }
    drop(known);
    dependencies.reserve(accepted_count);
    dependencies.extend(
        candidates
            .into_iter()
            .zip(accepted)
            .filter_map(|(dependency, is_accepted)| is_accepted.then_some(dependency)),
    );
}

pub(crate) fn handwritten_dependencies(asset: &ImportedAsset) -> Vec<AssetUri> {
    // Keep registry metadata aligned with every typed asset that exposes direct references.
    // The importer outcome is the authority used to resolve and persist dependency ids; the
    // catalog's later projection must not be the only place where UI/animation references live.
    let references = match asset {
        ImportedAsset::Scene(asset) => scene::extract(asset),
        ImportedAsset::Material(asset) => material::extract(asset),
        ImportedAsset::Model(asset) => model::extract(asset),
        ImportedAsset::AnimationClip(asset) => asset.direct_references(),
        ImportedAsset::AnimationGraph(asset) => asset.direct_references(),
        ImportedAsset::AnimationStateMachine(asset) => asset.direct_references(),
        ImportedAsset::MaterialGraph(asset) => asset.direct_references(),
        ImportedAsset::Terrain(asset) => asset.direct_references(),
        ImportedAsset::TerrainLayerStack(asset) => asset.direct_references(),
        ImportedAsset::TileSet(asset) => asset.direct_references(),
        ImportedAsset::TileMap(asset) => asset.direct_references(),
        ImportedAsset::Prefab(asset) => asset.direct_references(),
        ImportedAsset::UiIcon(asset) => asset.direct_references(),
        ImportedAsset::UiV2View(asset) => asset.direct_references(),
        ImportedAsset::UiV2Component(asset) => asset.direct_references(),
        ImportedAsset::UiV2Style(asset) => asset.direct_references(),
        _ => Vec::new(),
    };
    references
        .into_iter()
        .map(|reference| reference.locator)
        .collect()
}
