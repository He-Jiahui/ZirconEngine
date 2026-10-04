use std::collections::BTreeMap;
use std::path::PathBuf;

/// A watcher notification action applied to one canonical path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::plugin::native_plugin_loader) enum NativePluginDiscoveryManifestAction {
    Refresh,
    Remove,
}

/// Collector work kept on one active or pending ticket, never in the refresh selection key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::plugin::native_plugin_loader) enum NativePluginDiscoveryRefreshWork {
    FullRootScan,
    ManifestBatch {
        actions: BTreeMap<PathBuf, NativePluginDiscoveryManifestAction>,
        ordered_paths: Vec<PathBuf>,
    },
}

impl NativePluginDiscoveryRefreshWork {
    pub(in crate::plugin::native_plugin_loader) fn root_scan() -> Self {
        Self::FullRootScan
    }

    pub(in crate::plugin::native_plugin_loader) fn refresh_manifest(path: PathBuf) -> Self {
        Self::single(path, NativePluginDiscoveryManifestAction::Refresh)
    }

    pub(in crate::plugin::native_plugin_loader) fn remove_path(path: PathBuf) -> Self {
        Self::single(path, NativePluginDiscoveryManifestAction::Remove)
    }

    pub(in crate::plugin::native_plugin_loader) fn manifest_actions(
        &self,
    ) -> Option<&BTreeMap<PathBuf, NativePluginDiscoveryManifestAction>> {
        match self {
            Self::FullRootScan => None,
            Self::ManifestBatch { actions, .. } => Some(actions),
        }
    }

    pub(super) fn manifest_paths_in_notification_order(&self) -> Option<&[PathBuf]> {
        match self {
            Self::FullRootScan => None,
            Self::ManifestBatch { ordered_paths, .. } => Some(ordered_paths),
        }
    }

    /// Latest event wins for each canonical path; overflow/full-root invalidation dominates all
    /// incremental notifications because it is the only sound recovery from lost watcher state.
    pub(super) fn merge(&mut self, later: Self) {
        match later {
            Self::FullRootScan => *self = Self::FullRootScan,
            Self::ManifestBatch {
                actions: later_actions,
                ordered_paths: later_order,
            } => {
                if let Self::ManifestBatch {
                    actions: current_actions,
                    ordered_paths: current_order,
                } = self
                {
                    for path in later_order {
                        let action = *later_actions
                            .get(&path)
                            .expect("notification order must reference an action");
                        match action {
                            NativePluginDiscoveryManifestAction::Refresh => {
                                if current_actions.remove(&path).is_some() {
                                    current_order.retain(|current| current != &path);
                                }
                            }
                            NativePluginDiscoveryManifestAction::Remove => {
                                // A later directory removal makes every earlier manifest refresh
                                // inside that directory obsolete. Dropping it here prevents the
                                // collector from reading a path that the final batch removes.
                                current_order.retain(|current| !is_path_within(current, &path));
                                current_actions
                                    .retain(|current, _| !is_path_within(current, &path));
                            }
                        }
                        current_order.push(path.clone());
                        current_actions.insert(path, action);
                    }
                }
            }
        }
    }

    fn single(path: PathBuf, action: NativePluginDiscoveryManifestAction) -> Self {
        Self::ManifestBatch {
            actions: BTreeMap::from([(path.clone(), action)]),
            ordered_paths: vec![path],
        }
    }
}

fn is_path_within(path: &std::path::Path, ancestor: &std::path::Path) -> bool {
    path == ancestor || path.starts_with(ancestor)
}

#[cfg(test)]
#[path = "tests/work.rs"]
mod tests;
