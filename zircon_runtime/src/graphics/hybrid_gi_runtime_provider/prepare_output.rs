#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HybridGiRuntimePrepareOutput {
    evictable_probe_ids: Vec<u32>,
}

impl HybridGiRuntimePrepareOutput {
    pub fn new(evictable_probe_ids: Vec<u32>) -> Self {
        Self {
            evictable_probe_ids,
        }
    }

    pub fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probe_ids
    }

    pub fn into_evictable_probe_ids(self) -> Vec<u32> {
        self.evictable_probe_ids
    }
}
