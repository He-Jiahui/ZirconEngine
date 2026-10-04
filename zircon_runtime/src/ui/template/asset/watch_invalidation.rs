use std::collections::HashSet;

use crate::asset::watch::{AssetChange, AssetChangeKind};

use super::dependency_index::UiAssetDependencyIndex;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetWatchInvalidationReport {
    pub changed_assets: Vec<String>,
    pub rebuild_targets: Vec<String>,
    pub removed_assets: Vec<String>,
}

impl UiAssetDependencyIndex {
    pub fn apply_watch_changes(
        &mut self,
        changes: &[AssetChange],
    ) -> UiAssetWatchInvalidationReport {
        let mut report = UiAssetWatchInvalidationReport::default();
        let mut changed_seen = HashSet::new();
        let mut rebuild_seen = HashSet::new();
        let mut removed_seen = HashSet::new();

        for change in changes {
            if let Some(previous_uri) = change.previous_uri.as_ref() {
                let previous = previous_uri.to_string();
                if !removed_seen.contains(previous.as_str()) {
                    removed_seen.insert(previous.clone());
                    report.removed_assets.push(previous.clone());
                }
                for target in self.cascade_invalidation_targets(&previous) {
                    if !rebuild_seen.contains(target.as_str()) {
                        rebuild_seen.insert(target.clone());
                        report.rebuild_targets.push(target);
                    }
                }
                self.remove(&previous);
            }

            let changed = change.uri.to_string();
            if !changed_seen.contains(changed.as_str()) {
                changed_seen.insert(changed.clone());
                report.changed_assets.push(changed.clone());
            }

            for target in self.cascade_invalidation_targets(&changed) {
                if !rebuild_seen.contains(target.as_str()) {
                    rebuild_seen.insert(target.clone());
                    report.rebuild_targets.push(target);
                }
            }

            match &change.kind {
                AssetChangeKind::Removed => {
                    if !removed_seen.contains(changed.as_str()) {
                        removed_seen.insert(changed.clone());
                        report.removed_assets.push(changed.clone());
                    }
                    self.remove(&changed);
                }
                AssetChangeKind::Added | AssetChangeKind::Modified | AssetChangeKind::Renamed => {}
            }
        }

        report
    }
}

#[cfg(test)]
#[path = "tests/watch_invalidation_optimization_tests.rs"]
mod optimization_tests;
