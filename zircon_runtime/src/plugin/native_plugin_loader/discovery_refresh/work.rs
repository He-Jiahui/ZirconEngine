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
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::{NativePluginDiscoveryManifestAction, NativePluginDiscoveryRefreshWork};

    #[test]
    fn manifest_batches_keep_the_latest_action_per_path() {
        let weather = PathBuf::from("plugins/weather/plugin.toml");
        let climate = PathBuf::from("plugins/climate/plugin.toml");
        let mut work = NativePluginDiscoveryRefreshWork::refresh_manifest(weather.clone());

        work.merge(NativePluginDiscoveryRefreshWork::remove_path(
            weather.clone(),
        ));
        work.merge(NativePluginDiscoveryRefreshWork::refresh_manifest(
            climate.clone(),
        ));

        let actions = work.manifest_actions().expect("incremental actions");
        assert_eq!(
            actions.get(&weather),
            Some(&NativePluginDiscoveryManifestAction::Remove)
        );
        assert_eq!(
            actions.get(&climate),
            Some(&NativePluginDiscoveryManifestAction::Refresh)
        );
    }

    #[test]
    fn full_root_scan_dominates_incremental_notifications() {
        let mut work = NativePluginDiscoveryRefreshWork::refresh_manifest(PathBuf::from(
            "plugins/weather/plugin.toml",
        ));

        work.merge(NativePluginDiscoveryRefreshWork::root_scan());

        assert!(work.manifest_actions().is_none());
    }

    #[test]
    fn merged_batches_preserve_parent_child_notification_order() {
        let package = PathBuf::from("plugins/weather");
        let manifest = package.join("plugin.toml");
        let mut work = NativePluginDiscoveryRefreshWork::remove_path(package.clone());

        work.merge(NativePluginDiscoveryRefreshWork::refresh_manifest(
            manifest.clone(),
        ));

        let expected = [package, manifest];
        assert_eq!(
            work.manifest_paths_in_notification_order(),
            Some(expected.as_slice())
        );
    }

    #[test]
    fn later_parent_removal_discards_an_earlier_descendant_refresh() {
        let package = PathBuf::from("plugins/weather");
        let manifest = package.join("plugin.toml");
        let mut work = NativePluginDiscoveryRefreshWork::refresh_manifest(manifest);

        work.merge(NativePluginDiscoveryRefreshWork::remove_path(
            package.clone(),
        ));

        let actions = work.manifest_actions().expect("incremental actions");
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions.get(&package),
            Some(&NativePluginDiscoveryManifestAction::Remove)
        );
        assert_eq!(
            work.manifest_paths_in_notification_order(),
            Some([package].as_slice())
        );
    }

    #[test]
    fn unique_refreshes_append_without_rescanning_existing_order() {
        let weather = PathBuf::from("plugins/weather/plugin.toml");
        let climate = PathBuf::from("plugins/climate/plugin.toml");
        let ocean = PathBuf::from("plugins/ocean/plugin.toml");
        let mut work = NativePluginDiscoveryRefreshWork::refresh_manifest(weather.clone());

        work.merge(NativePluginDiscoveryRefreshWork::refresh_manifest(
            climate.clone(),
        ));
        work.merge(NativePluginDiscoveryRefreshWork::refresh_manifest(
            ocean.clone(),
        ));

        assert_eq!(
            work.manifest_paths_in_notification_order(),
            Some([weather, climate, ocean].as_slice())
        );
        assert_eq!(work.manifest_actions().map(BTreeMap::len), Some(3));
    }

    #[test]
    fn duplicate_refresh_moves_path_to_latest_notification_position() {
        let weather = PathBuf::from("plugins/weather/plugin.toml");
        let climate = PathBuf::from("plugins/climate/plugin.toml");
        let mut work = NativePluginDiscoveryRefreshWork::refresh_manifest(weather.clone());

        work.merge(NativePluginDiscoveryRefreshWork::refresh_manifest(
            climate.clone(),
        ));
        work.merge(NativePluginDiscoveryRefreshWork::refresh_manifest(
            weather.clone(),
        ));

        assert_eq!(
            work.manifest_paths_in_notification_order(),
            Some([climate, weather].as_slice())
        );
        assert_eq!(work.manifest_actions().map(BTreeMap::len), Some(2));
    }
}
