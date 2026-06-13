#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShadowResolutionTier {
    T128,
    T256,
    T512,
    T1024,
    T2048,
}

impl ShadowResolutionTier {
    pub const MIN: Self = Self::T128;
    pub const MAX: Self = Self::T2048;

    pub const fn size_px(self) -> u32 {
        match self {
            Self::T128 => 128,
            Self::T256 => 256,
            Self::T512 => 512,
            Self::T1024 => 1024,
            Self::T2048 => 2048,
        }
    }

    pub const fn next_lower(self) -> Option<Self> {
        match self {
            Self::T128 => None,
            Self::T256 => Some(Self::T128),
            Self::T512 => Some(Self::T256),
            Self::T1024 => Some(Self::T512),
            Self::T2048 => Some(Self::T1024),
        }
    }

    pub const fn downgraded_by_steps(self, steps: u32) -> Self {
        let mut tier = self;
        let mut remaining = steps;
        while remaining > 0 {
            tier = match tier.next_lower() {
                Some(lower) => lower,
                None => return tier,
            };
            remaining -= 1;
        }
        tier
    }
}

impl Default for ShadowResolutionTier {
    fn default() -> Self {
        Self::T1024
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShadowPcfQuality {
    Low,
    Medium,
    High,
}

impl ShadowPcfQuality {
    pub const fn tap_count(self) -> u32 {
        match self {
            Self::Low => 1,
            Self::Medium => 5,
            Self::High => 9,
        }
    }
}

impl Default for ShadowPcfQuality {
    fn default() -> Self {
        Self::Low
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightShadowSettings {
    pub casts_shadow: bool,
    pub depth_bias: f32,
    pub normal_bias: f32,
    pub strength: f32,
    pub resolution_preference: ShadowResolutionTier,
    pub pcf_quality: ShadowPcfQuality,
}

impl Default for LightShadowSettings {
    fn default() -> Self {
        Self {
            casts_shadow: false,
            depth_bias: 0.0,
            normal_bias: 0.0,
            strength: 1.0,
            resolution_preference: ShadowResolutionTier::default(),
            pcf_quality: ShadowPcfQuality::default(),
        }
    }
}
