use super::RenderPassDeviceEpoch;

pub(in crate::graphics::scene::scene_renderer) struct RenderPassDeviceEpochCache<K, V> {
    entry: Option<RenderPassDeviceEpochCacheEntry<K, V>>,
}

struct RenderPassDeviceEpochCacheEntry<K, V> {
    device_epoch: RenderPassDeviceEpoch,
    key: K,
    value: V,
}

impl<K, V> Default for RenderPassDeviceEpochCache<K, V> {
    fn default() -> Self {
        Self { entry: None }
    }
}

impl<K: Eq, V> RenderPassDeviceEpochCache<K, V> {
    pub(in crate::graphics::scene::scene_renderer) fn get_or_try_insert_with(
        &mut self,
        device_epoch: RenderPassDeviceEpoch,
        key: K,
        create: impl FnOnce() -> Result<V, String>,
    ) -> Result<&V, String> {
        match self.entry.as_ref() {
            Some(entry) if entry.device_epoch == device_epoch && entry.key == key => {
                return Ok(&entry.value);
            }
            _ => {}
        }

        drop(self.entry.take());
        let value = create()?;
        let entry = self.entry.insert(RenderPassDeviceEpochCacheEntry {
            device_epoch,
            key,
            value,
        });
        Ok(&entry.value)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
}
