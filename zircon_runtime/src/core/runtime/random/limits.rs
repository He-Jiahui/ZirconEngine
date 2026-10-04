use zr_contracts::random::RandomServiceCheckpoint;

/// 默认上限与 checkpoint 格式的最大流数一致，限制单个 Runtime 保留的稳定流键数。
/// Retained-state limits for one Runtime random authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomServiceLimits {
    max_registered_streams: usize,
}

impl RandomServiceLimits {
    /// MVP bound for retained deterministic stream owners in one Runtime.
    pub const MVP: Self = Self::new(RandomServiceCheckpoint::MAX_STREAMS);

    pub const fn new(max_registered_streams: usize) -> Self {
        Self {
            max_registered_streams,
        }
    }

    pub const fn max_registered_streams(self) -> usize {
        self.max_registered_streams
    }
}

impl Default for RandomServiceLimits {
    fn default() -> Self {
        Self::MVP
    }
}
