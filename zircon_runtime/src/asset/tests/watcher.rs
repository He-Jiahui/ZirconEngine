use crossbeam_channel::unbounded;
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::asset::watch::{watch_loop_for_test, watched_asset_uri_for_path};
use crate::asset::watch::{
    AssetChange, AssetChangeKind, AssetWatchError, AssetWatchEvent, AssetWatcher,
};
use crate::asset::AssetUri;

#[test]
fn watcher_folds_redundant_events_into_latest_change_set() {
    let material = AssetUri::parse("res://materials/grid.zmaterial").unwrap();
    let renamed = AssetUri::parse("res://materials/grid_pbr.zmaterial").unwrap();

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
    let meta_path = Path::new("sandbox/assets/materials/grid.zmaterial.zmeta");
    let old_meta_path = Path::new("sandbox/assets/materials/grid.zmaterial.meta.toml");

    assert!(watched_asset_uri_for_path(assets_root, meta_path).is_err());
    assert!(watched_asset_uri_for_path(assets_root, old_meta_path).is_err());
}

#[test]
fn rapid_successive_writes_within_debounce_window_emit_single_reload() {
    let assets_root = PathBuf::from("sandbox/assets");
    let material_path = assets_root.join("materials").join("grid.zmaterial");
    let (stop_tx, stop_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let (change_tx, change_rx) = unbounded::<Vec<AssetChange>>();
    let (error_tx, error_rx) = unbounded::<AssetWatchError>();

    let join = std::thread::spawn(move || {
        watch_loop_for_test(
            assets_root,
            Duration::from_millis(10),
            stop_rx,
            event_rx,
            Arc::new(move |changes| {
                let _ = change_tx.send(changes);
            }),
            Arc::new(move |error| {
                let _ = error_tx.send(error);
            }),
        );
    });

    event_tx.send(Ok(modified_event(&material_path))).unwrap();
    event_tx.send(Ok(modified_event(&material_path))).unwrap();

    let changes = change_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(
        changes,
        vec![AssetChange::new(
            AssetChangeKind::Modified,
            AssetUri::parse("res://materials/grid.zmaterial").unwrap(),
            None,
        )]
    );
    assert!(change_rx.recv_timeout(Duration::from_millis(50)).is_err());
    assert!(error_rx.try_recv().is_err());

    stop_tx.send(()).unwrap();
    join.join().unwrap();
}

#[test]
fn watcher_failure_on_removed_directory_surfaces_observable_error() {
    let assets_root = PathBuf::from("sandbox/assets");
    let (stop_tx, stop_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let (change_tx, change_rx) = unbounded::<Vec<AssetChange>>();
    let (error_tx, error_rx) = unbounded::<AssetWatchError>();

    let loop_root = assets_root.clone();
    let join = std::thread::spawn(move || {
        watch_loop_for_test(
            loop_root,
            Duration::from_millis(1),
            stop_rx,
            event_rx,
            Arc::new(move |changes| {
                let _ = change_tx.send(changes);
            }),
            Arc::new(move |error| {
                let _ = error_tx.send(error);
            }),
        );
    });

    event_tx
        .send(Err(
            notify::Error::path_not_found().add_path(assets_root.clone())
        ))
        .unwrap();

    let error = error_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(error.assets_root, assets_root);
    assert!(
        error.message.contains("path") || error.message.contains("not found"),
        "unexpected watcher error: {error:?}"
    );
    assert!(change_rx.try_recv().is_err());

    stop_tx.send(()).unwrap();
    join.join().unwrap();
}

fn modified_event(path: &Path) -> Event {
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content)))
        .add_path(path.to_path_buf())
}
