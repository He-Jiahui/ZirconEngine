#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderArtifactIoPriority(u8);

impl RenderArtifactIoPriority {
    pub const LOW: Self = Self(32);
    pub const NORMAL: Self = Self(128);
    pub const HIGH: Self = Self(192);
    pub const CRITICAL: Self = Self(u8::MAX);

    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }
}
