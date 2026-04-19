use std::path::Path;

use crate::watch::watched_asset_uri_for_path;
use crate::watch::{AssetChange, AssetChangeKind, AssetWatchEvent, AssetWatcher};
use crate::AssetUri;

#[test]
fn watcher_folds_redundant_events_into_latest_change_set() {
    let material = AssetUri::parse("res://materials/grid.material.toml").unwrap();
    let renamed = AssetUri::parse("res://materials/grid_pbr.material.toml").unwrap();

    let changes = AssetWatcher::fold_events(&[
        AssetWatchEvent::Added(material.clone()),
        AssetWatchEvent::Modified(material.clone()),
        AssetWatchEvent::Renamed {
            from: material.clone(),
            to: renamed.clone(),
        },
    ]);

    assert_eq!(
        changes,
        vec![AssetChange::new(
            AssetChangeKind::Renamed,
            renamed,
            Some(material),
        )]
    );
}

#[test]
fn watcher_ignores_meta_sidecar_paths() {
    let assets_root = Path::new("sandbox/assets");
    let meta_path = Path::new("sandbox/assets/materials/grid.material.toml.meta.toml");

    assert!(watched_asset_uri_for_path(assets_root, meta_path).is_err());
}
