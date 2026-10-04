use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::RenderPassDeviceEpochCache;
use crate::graphics::scene::scene_renderer::graph_execution::RenderPassDeviceEpoch;

#[derive(Debug)]
struct DropProbe(Arc<AtomicUsize>);

impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn stable_identity_reuses_value_and_epoch_or_key_change_releases_before_create() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut cache = RenderPassDeviceEpochCache::<u32, DropProbe>::default();
    let first_epoch = RenderPassDeviceEpoch::new(7, 11);

    cache
        .get_or_try_insert_with(first_epoch, 3, || Ok(DropProbe(Arc::clone(&drops))))
        .unwrap();
    cache
        .get_or_try_insert_with(first_epoch, 3, || {
            panic!("stable cache identity must not recreate its value")
        })
        .unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 0);

    cache
        .get_or_try_insert_with(RenderPassDeviceEpoch::new(7, 12), 3, || {
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            Ok(DropProbe(Arc::clone(&drops)))
        })
        .unwrap();
    cache
        .get_or_try_insert_with(RenderPassDeviceEpoch::new(7, 12), 4, || {
            assert_eq!(drops.load(Ordering::SeqCst), 2);
            Ok(DropProbe(Arc::clone(&drops)))
        })
        .unwrap();
    cache
        .get_or_try_insert_with(RenderPassDeviceEpoch::new(8, 12), 4, || {
            assert_eq!(drops.load(Ordering::SeqCst), 3);
            Ok(DropProbe(Arc::clone(&drops)))
        })
        .unwrap();
}

#[test]
fn failed_recreation_leaves_old_epoch_value_released() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut cache = RenderPassDeviceEpochCache::<(), DropProbe>::default();

    cache
        .get_or_try_insert_with(RenderPassDeviceEpoch::new(5, 1), (), || {
            Ok(DropProbe(Arc::clone(&drops)))
        })
        .unwrap();
    let error = cache
        .get_or_try_insert_with(RenderPassDeviceEpoch::new(5, 2), (), || {
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            Err("replacement failed".to_string())
        })
        .unwrap_err();
    assert_eq!(error, "replacement failed");

    cache
        .get_or_try_insert_with(RenderPassDeviceEpoch::new(5, 2), (), || {
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            Ok(DropProbe(Arc::clone(&drops)))
        })
        .unwrap();
}
