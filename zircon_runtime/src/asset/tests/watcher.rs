use crossbeam_channel::unbounded;
use notify::event::{DataChange, ModifyKind};
use notify::{Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::asset::watch::{watch_ingress, watch_loop_for_test, watched_asset_uri_for_path};
use crate::asset::watch::{
    AssetChange, AssetChangeKind, AssetWatchBatch, AssetWatchError, AssetWatchEvent, AssetWatcher,
    AssetWatcherOptions,
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
fn watcher_ignores_atomic_write_transaction_siblings() {
    let assets_root = Path::new("sandbox/assets");
    let staging = Path::new("sandbox/assets/shaders/.pbr_shader.zmeta.zr-staging-37484-1180");
    let backup = Path::new("sandbox/assets/shaders/.pbr_shader.zmeta.zr-backup-37484-1181");
    let ordinary_hidden = Path::new("sandbox/assets/.zr-staging-guide.txt");

    assert!(watched_asset_uri_for_path(assets_root, staging).is_err());
    assert!(watched_asset_uri_for_path(assets_root, backup).is_err());
    assert_eq!(
        watched_asset_uri_for_path(assets_root, ordinary_hidden).unwrap(),
        AssetUri::parse("res://.zr-staging-guide.txt").unwrap()
    );
}

#[test]
fn rapid_successive_writes_within_debounce_window_emit_single_reload() {
    let assets_root = PathBuf::from("sandbox/assets");
    let material_path = assets_root.join("materials").join("grid.zmaterial");
    let (stop_tx, stop_rx) = unbounded();
    let options = AssetWatcherOptions {
        debounce: Duration::from_millis(10),
        max_batch_latency: Duration::from_millis(100),
        ..AssetWatcherOptions::default()
    };
    let (event_tx, event_rx) = watch_ingress(options);
    let (change_tx, change_rx) = unbounded::<Vec<AssetChange>>();
    let (error_tx, error_rx) = unbounded::<AssetWatchError>();

    let join = std::thread::spawn(move || {
        watch_loop_for_test(
            assets_root,
            options,
            stop_rx,
            event_rx,
            Arc::new(move |batch| {
                let _ = change_tx.send(batch.changes);
            }),
            Arc::new(move |error| {
                let _ = error_tx.send(error);
            }),
        );
    });

    event_tx.try_send(Ok(modified_event(&material_path)));
    event_tx.try_send(Ok(modified_event(&material_path)));

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
fn watcher_loop_folds_pending_events_incrementally() {
    let source = include_str!("../watch/watch_loop.rs");

    assert!(source.contains("fold_event(&mut pending"));
    assert!(!source.contains("pending.extend("));
}

#[test]
fn watcher_failure_on_removed_directory_surfaces_observable_error() {
    let assets_root = PathBuf::from("sandbox/assets");
    let (stop_tx, stop_rx) = unbounded();
    let options = AssetWatcherOptions {
        debounce: Duration::from_millis(1),
        max_batch_latency: Duration::from_millis(100),
        ..AssetWatcherOptions::default()
    };
    let (event_tx, event_rx) = watch_ingress(options);
    let (change_tx, change_rx) = unbounded::<Vec<AssetChange>>();
    let (error_tx, error_rx) = unbounded::<AssetWatchError>();

    let loop_root = assets_root.clone();
    let join = std::thread::spawn(move || {
        watch_loop_for_test(
            loop_root,
            options,
            stop_rx,
            event_rx,
            Arc::new(move |batch| {
                let _ = change_tx.send(batch.changes);
            }),
            Arc::new(move |error| {
                let _ = error_tx.send(error);
            }),
        );
    });

    event_tx.try_send(Err(
        notify::Error::path_not_found().add_path(assets_root.clone())
    ));

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

#[test]
fn continuous_watcher_storm_flushes_at_the_max_batch_latency() {
    let assets_root = PathBuf::from("sandbox/assets");
    let material_path = assets_root.join("materials/grid.zmaterial");
    let options = AssetWatcherOptions {
        debounce: Duration::from_secs(1),
        max_batch_latency: Duration::from_millis(20),
        ingress_entry_capacity: 256,
        ..AssetWatcherOptions::default()
    };
    let (stop_tx, stop_rx) = unbounded();
    let (event_tx, event_rx) = watch_ingress(options);
    let (batch_tx, batch_rx) = unbounded::<AssetWatchBatch>();
    let join = std::thread::spawn(move || {
        watch_loop_for_test(
            assets_root,
            options,
            stop_rx,
            event_rx,
            Arc::new(move |batch| {
                let _ = batch_tx.send(batch);
            }),
            Arc::new(|_| {}),
        );
    });

    let started = std::time::Instant::now();
    while started.elapsed() < Duration::from_millis(75) {
        event_tx.try_send(Ok(modified_event(&material_path)));
        std::thread::yield_now();
    }

    let first = batch_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(first.changes.len(), 1);
    assert!(first.diagnostics.oldest_age >= Duration::from_millis(20));
    assert!(batch_rx.recv_timeout(Duration::from_secs(1)).is_ok());
    stop_tx.send(()).unwrap();
    join.join().unwrap();
}

#[test]
fn watcher_pending_overflow_emits_an_explicit_reconciliation_token() {
    let assets_root = PathBuf::from("sandbox/assets");
    let options = AssetWatcherOptions {
        debounce: Duration::from_millis(1),
        max_batch_latency: Duration::from_millis(20),
        pending_entry_capacity: 1,
        ..AssetWatcherOptions::default()
    };
    let (stop_tx, stop_rx) = unbounded();
    let (event_tx, event_rx) = watch_ingress(options);
    let (batch_tx, batch_rx) = unbounded::<AssetWatchBatch>();
    let join = std::thread::spawn(move || {
        watch_loop_for_test(
            assets_root.clone(),
            options,
            stop_rx,
            event_rx,
            Arc::new(move |batch| {
                let _ = batch_tx.send(batch);
            }),
            Arc::new(|_| {}),
        );
    });

    event_tx.try_send(Ok(modified_event(Path::new(
        "sandbox/assets/materials/a.zmaterial",
    ))));
    event_tx.try_send(Ok(modified_event(Path::new(
        "sandbox/assets/materials/b.zmaterial",
    ))));

    let batch = batch_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(batch.requires_reconciliation);
    assert_eq!(batch.diagnostics.pending_overflow_count, 1);
    assert_eq!(batch.changes.len(), 1);
    stop_tx.send(()).unwrap();
    join.join().unwrap();
}

#[test]
fn watcher_ingress_overflow_wakes_the_loop_and_emits_reconciliation() {
    let assets_root = PathBuf::from("sandbox/assets");
    let options = AssetWatcherOptions {
        debounce: Duration::from_millis(1),
        max_batch_latency: Duration::from_millis(20),
        ingress_entry_capacity: 1,
        ..AssetWatcherOptions::default()
    };
    let (stop_tx, stop_rx) = unbounded();
    let (event_tx, event_rx) = watch_ingress(options);
    let (batch_tx, batch_rx) = unbounded::<AssetWatchBatch>();

    event_tx.try_send(Ok(modified_event(Path::new(
        "sandbox/assets/materials/a.zmaterial",
    ))));
    event_tx.try_send(Ok(modified_event(Path::new(
        "sandbox/assets/materials/b.zmaterial",
    ))));

    let join = std::thread::spawn(move || {
        watch_loop_for_test(
            assets_root,
            options,
            stop_rx,
            event_rx,
            Arc::new(move |batch| {
                let _ = batch_tx.send(batch);
            }),
            Arc::new(|_| {}),
        );
    });

    let batch = batch_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(batch.requires_reconciliation);
    assert_eq!(batch.diagnostics.ingress_overflow_count, 1);
    assert_eq!(batch.changes.len(), 1);
    stop_tx.send(()).unwrap();
    join.join().unwrap();
}

fn modified_event(path: &Path) -> Event {
    Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content)))
        .add_path(path.to_path_buf())
}
