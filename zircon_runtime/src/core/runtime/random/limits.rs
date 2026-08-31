/// Retained-state limits for one Runtime random authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RandomServiceLimits {
    max_registered_streams: usize,
}

impl RandomServiceLimits {
    /// MVP bound for retained deterministic stream owners in one Runtime.
    pub const MVP: Self = Self::new(65_536);

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
