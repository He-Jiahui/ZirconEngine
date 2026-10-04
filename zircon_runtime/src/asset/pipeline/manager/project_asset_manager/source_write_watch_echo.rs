use std::collections::{HashMap, VecDeque};
use std::fs;

use crate::asset::project::ImportSourceWatchEcho;
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::AssetUri;

const MAX_TRANSACTION_WATCH_ECHOES: usize = 1_024;

/// 记住本事务刚写出的源文件哈希，避免 watcher 回声造成重复导入；真实外部修改仍需传播。
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
#[path = "tests/source_write_watch_echo.rs"]
mod tests;
