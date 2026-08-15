use std::path::Path;

use self::authority::discovery_authority;
use super::{NativePluginLoadReport, NativePluginLoader};

pub(super) mod authority;

#[cfg(test)]
mod tests;

impl NativePluginLoader {
    /// Projects the canonical authority's last-good snapshot. A cold root waits for the single
    /// bounded authority refresh; later calls perform no synchronous filesystem traversal.
    pub fn discover(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        discovery_authority().discover(root.as_ref())
    }

    /// Schedules a coalesced, path-scoped manifest refresh after a watcher/editor notification.
    pub fn refresh_discovery_manifest(
        &self,
        root: impl AsRef<Path>,
        manifest_path: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        discovery_authority().refresh_manifest(root.as_ref(), manifest_path.as_ref())
    }

    /// Removes a watcher-reported path from the immutable discovery index without rescanning.
    pub fn remove_discovered_path(
        &self,
        root: impl AsRef<Path>,
        removed_path: impl AsRef<Path>,
    ) -> NativePluginLoadReport {
        discovery_authority().remove_path(root.as_ref(), removed_path.as_ref())
    }

    /// Returns the last published generation without polling or touching the filesystem.
    pub fn discovery_generation(&self, root: impl AsRef<Path>) -> Option<u64> {
        discovery_authority().generation(root.as_ref())
    }
}
