use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crossbeam_channel::unbounded;

use super::{ProjectAssetChangeSubscriber, ProjectAssetManager};
use crate::asset::watch::{AssetChange, AssetChangeKind};
use crate::asset::AssetUri;

#[test]
fn delivered_asset_change_invokes_the_subscription_wake() {
    let (sender, receiver) = unbounded();
    let wake_count = Arc::new(AtomicUsize::new(0));
    let wake_count_for_callback = Arc::clone(&wake_count);
    let subscriber = ProjectAssetChangeSubscriber::new(
        sender,
        Some(Arc::new(move || {
            wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
        })),
    );
    let change = AssetChange::new(
        AssetChangeKind::Modified,
        AssetUri::parse("res://textures/albedo.png").unwrap(),
        None,
    );

    assert!(subscriber.send(change));
    subscriber.wake();

    assert_eq!(receiver.len(), 1);
    assert_eq!(wake_count.load(Ordering::Relaxed), 1);
}

#[test]
fn project_generation_wake_tokens_coalesce_at_capacity_one() {
    let manager = ProjectAssetManager::default();
    let wake_count = Arc::new(AtomicUsize::new(0));
    let wake_count_for_callback = Arc::clone(&wake_count);
    let receiver = manager.subscribe_project_generation_wake(Arc::new(move || {
        wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
    }));
    let change = AssetChange::new(
        AssetChangeKind::Modified,
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        None,
    );

    let generation = manager.project_generation_write();
    manager.publish_project_generation(generation, vec![change.clone()]);
    let generation = manager.project_generation_write();
    manager.publish_project_generation(generation, vec![change.clone()]);
    assert_eq!(receiver.len(), 1);
    assert_eq!(wake_count.load(Ordering::Relaxed), 1);

    receiver.try_recv().unwrap();
    let generation = manager.project_generation_write();
    manager.publish_project_generation(generation, vec![change]);
    assert_eq!(receiver.len(), 1);
    assert_eq!(wake_count.load(Ordering::Relaxed), 2);
}

#[test]
fn empty_committed_project_generation_still_wakes_reactive_consumers() {
    let manager = ProjectAssetManager::default();
    let wake_count = Arc::new(AtomicUsize::new(0));
    let wake_count_for_callback = Arc::clone(&wake_count);
    let receiver = manager.subscribe_project_generation_wake(Arc::new(move || {
        wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
    }));
    let generation = manager.project_generation_write();

    manager.publish_project_generation(generation, Vec::new());

    assert_eq!(receiver.len(), 1);
    assert_eq!(wake_count.load(Ordering::Relaxed), 1);
}
