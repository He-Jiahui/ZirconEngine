pub type VmPluginManagementPolicyResult<T> = std::result::Result<T, VmPluginManagementPolicyError>;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum VmPluginManagementPolicyError {
    #[error("disabled garbage collection cannot set interval_frames")]
    GarbageCollectionDisabledWithInterval,
    #[error("garbage collection interval_frames must be greater than zero")]
    GarbageCollectionIntervalFramesZero,
    #[error("memory soft_limit_bytes must be greater than zero")]
    MemorySoftLimitBytesZero,
    #[error("memory hard_limit_bytes must be greater than zero")]
    MemoryHardLimitBytesZero,
    #[error(
        "memory soft_limit_bytes {soft_limit_bytes} exceeds hard_limit_bytes {hard_limit_bytes}"
    )]
    MemorySoftLimitExceedsHardLimit {
        soft_limit_bytes: u64,
        hard_limit_bytes: u64,
    },
}
