use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::core::resource::AssetReference;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetDependencyIndex {
    // Forward and reverse maps stay in sync so watch invalidation can jump from a
    // changed leaf asset to every compiled UI asset that must be rebuilt.
    references_by_asset: BTreeMap<String, Vec<AssetReference>>,
    dependents_by_asset: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetDependencyQueryReport {
    pub asset_id: String,
    pub direct_references: Vec<AssetReference>,
    pub direct_dependents: Vec<String>,
    pub cascade_dependents: Vec<String>,
}

impl UiAssetDependencyIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_compiled(&mut self, asset_id: &str, references: &[AssetReference]) {
        self.remove(asset_id);

        let references = dedupe_references(references);
        for reference in &references {
            self.dependents_by_asset
                .entry(reference_asset_id(reference))
                .or_default()
                .insert(asset_id.to_string());
        }
        self.references_by_asset
            .insert(asset_id.to_string(), references);
    }

    pub fn remove(&mut self, asset_id: &str) -> Option<Vec<AssetReference>> {
        let references = self.references_by_asset.remove(asset_id)?;
        for reference in &references {
            let dependency_asset_id = reference_asset_id(reference);
            let remove_dependency =
                if let Some(dependents) = self.dependents_by_asset.get_mut(&dependency_asset_id) {
                    dependents.remove(asset_id);
                    dependents.is_empty()
                } else {
                    false
                };
            if remove_dependency {
                self.dependents_by_asset.remove(&dependency_asset_id);
            }
        }
        Some(references)
    }

    pub fn references_of(&self, asset_id: &str) -> &[AssetReference] {
        self.references_by_asset
            .get(asset_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn dependents_of<'a>(&'a self, asset_id: &str) -> impl Iterator<Item = &'a str> {
        self.dependents_by_asset
            .get(asset_id)
            .into_iter()
            .flat_map(|dependents| dependents.iter().map(String::as_str))
    }

    pub fn cascade_invalidation_targets(&self, changed: &str) -> Vec<String> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        let mut targets = Vec::new();

        // The changed asset seeds traversal but is not returned as its own
        // dependent, which keeps self-cycles from scheduling duplicate rebuilds.
        seen.insert(changed);
        queue.push_back(changed);

        while let Some(asset_id) = queue.pop_front() {
            let Some(dependents) = self.dependents_by_asset.get(asset_id) else {
                continue;
            };
            for dependent in dependents {
                let dependent = dependent.as_str();
                if seen.insert(dependent) {
                    targets.push(dependent.to_string());
                    queue.push_back(dependent);
                }
            }
        }

        targets
    }

    pub fn query_asset(&self, asset_id: &str) -> UiAssetDependencyQueryReport {
        UiAssetDependencyQueryReport {
            asset_id: asset_id.to_string(),
            direct_references: self.references_of(asset_id).to_vec(),
            direct_dependents: self.dependents_of(asset_id).map(str::to_string).collect(),
            cascade_dependents: self.cascade_invalidation_targets(asset_id),
        }
    }

    pub fn asset_count(&self) -> usize {
        self.references_by_asset.len()
    }

    pub fn is_empty(&self) -> bool {
        self.references_by_asset.is_empty()
    }
}

fn dedupe_references(references: &[AssetReference]) -> Vec<AssetReference> {
    let mut unique = BTreeMap::new();
    for reference in references {
        unique
            .entry(reference_asset_id(reference))
            .or_insert_with(|| reference.clone());
    }
    unique.into_iter().map(|(_, reference)| reference).collect()
}

fn reference_asset_id(reference: &AssetReference) -> String {
    reference.locator.to_string()
}
