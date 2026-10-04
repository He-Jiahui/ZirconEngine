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
        let matches = self
            .entry
            .as_ref()
            .is_some_and(|entry| entry.device_epoch == device_epoch && entry.key == key);
        if !matches {
            drop(self.entry.take());
            let value = create()?;
            self.entry.insert(RenderPassDeviceEpochCacheEntry {
                device_epoch,
                key,
                value,
            });
        }
        let entry = self
            .entry
            .as_ref()
            .expect("successful cache lookup must retain an entry");
        Ok(&entry.value)
    }
}

#[cfg(test)]
#[path = "tests/render_pass_device_epoch_cache.rs"]
mod tests;
