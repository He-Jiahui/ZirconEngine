#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeScenePrepareResources {
    atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
}

pub(crate) trait HybridGiScenePrepareResourceSamples {
    fn atlas_slot_rgba_sample(&self, atlas_slot_id: u32) -> Option<[u8; 4]>;

    fn capture_slot_rgba_sample(&self, capture_slot_id: u32) -> Option<[u8; 4]>;
}

impl HybridGiRuntimeScenePrepareResources {
    pub(crate) fn new(
        atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
        capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    ) -> Self {
        Self {
            atlas_slot_rgba_samples,
            capture_slot_rgba_samples,
        }
    }
}

impl HybridGiScenePrepareResourceSamples for HybridGiRuntimeScenePrepareResources {
    fn atlas_slot_rgba_sample(&self, atlas_slot_id: u32) -> Option<[u8; 4]> {
        self.atlas_slot_rgba_samples
            .iter()
            .find_map(|(slot_id, rgba)| (*slot_id == atlas_slot_id).then_some(*rgba))
    }

    fn capture_slot_rgba_sample(&self, capture_slot_id: u32) -> Option<[u8; 4]> {
        self.capture_slot_rgba_samples
            .iter()
            .find_map(|(slot_id, rgba)| (*slot_id == capture_slot_id).then_some(*rgba))
    }
}
