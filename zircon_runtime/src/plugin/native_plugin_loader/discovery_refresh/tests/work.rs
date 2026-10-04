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
