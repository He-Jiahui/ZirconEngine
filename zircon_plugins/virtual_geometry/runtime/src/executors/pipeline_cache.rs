//! Small variant cache; a device epoch change releases every native handle.
use zircon_runtime::graphics::RenderPassDeviceEpoch;

pub(super) struct PipelineCache<K, V, E = RenderPassDeviceEpoch> {
    epoch: Option<E>,
    entries: Vec<(K, V)>,
}
impl<K: Eq, V: Clone, E: Eq + Copy> PipelineCache<K, V, E> {
    pub(super) const fn new() -> Self {
        Self {
            epoch: None,
            entries: Vec::new(),
        }
    }
    pub(super) fn get(&mut self, epoch: Option<E>, key: K, create: impl FnOnce() -> V) -> V {
        let Some(epoch) = epoch else {
            return create();
        };
        if self.epoch != Some(epoch) {
            self.entries.clear();
            self.epoch = Some(epoch);
        }
        if let Some((_, value)) = self.entries.iter().find(|(candidate, _)| *candidate == key) {
            return value.clone();
        }
        let value = create();
        if self.entries.len() >= 8 {
            self.entries.remove(0);
        }
        self.entries.push((key, value.clone()));
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline_cache_reuses_variants_and_invalidates_on_device_epoch_change() {
        let mut cache = PipelineCache::new();
        let epoch = Some(1_u64);
        assert_eq!(cache.get(epoch, 0, || 10), 10);
        assert_eq!(cache.get(epoch, 1, || 11), 11);
        assert_eq!(
            cache.get(epoch, 0, || panic!("stable variant recreated")),
            10
        );
        assert_eq!(cache.get(Some(2_u64), 0, || 12), 12);
        assert_eq!(cache.entries.len(), 1);
        assert_eq!(cache.get(None, 0, || 13), 13);
        assert_eq!(cache.get(epoch, 0, || 14), 14);
    }
}
