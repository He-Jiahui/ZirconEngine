use crate::{AssetChange, AssetChangeKind, AssetUri, AssetWatchEvent, AssetWatcher};

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
