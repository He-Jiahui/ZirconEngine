use std::collections::HashSet;

mod material;
mod model;
mod scene;

use crate::asset::{AssetImportOutcome, AssetUri, ImportedAsset};

#[cfg(test)]
#[path = "dedup_index_tests.rs"]
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
    let mut additions = Vec::with_capacity(candidates.len());
    for dependency in &candidates {
        if known.insert(dependency) {
            additions.push(dependency.clone());
        }
    }
    drop(known);
    dependencies.extend(additions);
}

pub(crate) fn handwritten_dependencies(asset: &ImportedAsset) -> Vec<AssetUri> {
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
