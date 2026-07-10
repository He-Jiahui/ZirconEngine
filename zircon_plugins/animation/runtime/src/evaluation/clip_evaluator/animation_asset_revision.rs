use zircon_runtime::core::resource::ResourceId;

/// Stable asset identity paired with the resource revision used for cache invalidation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnimationAssetRevision {
    id: ResourceId,
    revision: u64,
}

impl AnimationAssetRevision {
    pub const fn new(id: ResourceId, revision: u64) -> Self {
        Self { id, revision }
    }

    pub const fn id(self) -> ResourceId {
        self.id
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }
}
