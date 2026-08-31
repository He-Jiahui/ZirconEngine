use std::collections::{HashMap, VecDeque};
use std::fs;

use crate::asset::project::ImportSourceWatchEcho;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::AssetUri;

const MAX_TRANSACTION_WATCH_ECHOES: usize = 1_024;

#[derive(Default)]
pub(in crate::asset::pipeline::manager) struct TransactionWatchEchoes {
    entries: HashMap<AssetUri, ImportSourceWatchEcho>,
    insertion_order: VecDeque<AssetUri>,
}

impl TransactionWatchEchoes {
    pub(in crate::asset::pipeline::manager) fn register(
        &mut self,
        echoes: impl IntoIterator<Item = ImportSourceWatchEcho>,
    ) {
        for echo in echoes {
            let watched_uri = echo.watched_uri().clone();
            if self.entries.insert(watched_uri.clone(), echo).is_none() {
                self.insertion_order.push_back(watched_uri);
            }
        }
        while self.entries.len() > MAX_TRANSACTION_WATCH_ECHOES {
            let Some(evicted_uri) = self.insertion_order.pop_front() else {
                break;
            };
            self.entries.remove(&evicted_uri);
        }
    }

    pub(in crate::asset::pipeline::manager) fn filter(
        &mut self,
        changes: Vec<AssetChange>,
    ) -> Vec<AssetChange> {
        changes
            .into_iter()
            .filter_map(|change| self.filter_change(change))
            .collect()
    }

    pub(in crate::asset::pipeline::manager) fn clear(&mut self) {
        self.entries.clear();
        self.insertion_order.clear();
    }

    fn filter_change(&mut self, change: AssetChange) -> Option<AssetChange> {
        if !matches!(
            change.kind,
            AssetChangeKind::Added | AssetChangeKind::Modified
        ) {
            self.entries.remove(&change.uri);
            if let Some(previous_uri) = change.previous_uri.as_ref() {
                self.entries.remove(previous_uri);
            }
            return Some(change);
        }
        let Some(echo) = self.entries.get(&change.uri).cloned() else {
            return Some(change);
        };
        let current_hash = fs::read(echo.target_path())
            .ok()
            .map(|bytes| blake3::hash(&bytes));
        if current_hash == Some(echo.content_hash()) {
            return None;
        }
        self.entries.remove(&change.uri);
        if change.uri == *echo.source_uri() {
            return Some(change);
        }
        Some(AssetChange::new(
            change.kind,
            echo.source_uri().clone(),
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_transaction_echo_is_suppressed_and_sidecar_change_retargets_model() {
        let root = std::env::temp_dir().join(format!(
            "zircon_transaction_watch_echo_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let model_path = root.join("mesh.obj");
        let material_path = root.join("mesh.mtl");
        std::fs::write(&model_path, b"model").unwrap();
        std::fs::write(&material_path, b"initial material").unwrap();
        let model_uri = AssetUri::parse("res://models/mesh.obj").unwrap();
        let material_uri = AssetUri::parse("res://models/mesh.mtl").unwrap();
        let mut echoes = TransactionWatchEchoes::default();
        echoes.register([
            ImportSourceWatchEcho::new(model_uri.clone(), model_uri.clone(), model_path, b"model"),
            ImportSourceWatchEcho::new(
                material_uri.clone(),
                model_uri.clone(),
                material_path.clone(),
                b"initial material",
            ),
        ]);

        assert!(echoes
            .filter(vec![AssetChange::new(
                AssetChangeKind::Added,
                model_uri.clone(),
                None,
            )])
            .is_empty());

        std::fs::write(&material_path, b"changed material").unwrap();
        let changes = echoes.filter(vec![AssetChange::new(
            AssetChangeKind::Modified,
            material_uri,
            None,
        )]);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].uri, model_uri);

        let _ = std::fs::remove_dir_all(root);
    }
}
