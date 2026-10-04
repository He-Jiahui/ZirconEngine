use crate::core::framework::platform::{
    PlatformHostGeneration, PlatformHostInstanceId, PlatformHostLifecycleState,
    PlatformHostObservedCapabilities,
};

/// The host observations required before a static platform catalog entry can
/// become a runtime-ready capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeHostRequirement {
    EventLoop,
    Windowing,
    DisplayTopology,
    WindowingAndEventLoop,
}

impl PlatformRuntimeHostRequirement {
    // requirement 与宿主证据逐项对应；WindowingAndEventLoop 必须同时观察到两者。
    pub(crate) const fn is_observed_by(self, observed: PlatformHostObservedCapabilities) -> bool {
        match self {
            Self::EventLoop => observed.event_loop(),
            Self::Windowing => observed.windowing(),
            Self::DisplayTopology => observed.display_topology(),
            Self::WindowingAndEventLoop => observed.windowing() && observed.event_loop(),
        }
    }
}

/// Runtime truth for one catalog capability. `Ready` requires both an active
/// provider identity and the required observation; a compiled feature alone
/// can never produce this variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeCapabilityStatus<T> {
    Disabled,
    FeatureDisabled {
        feature: &'static str,
    },
    Unavailable {
        reason: &'static str,
    },
    HostUnavailable {
        lifecycle: PlatformHostLifecycleState,
        provider: Option<PlatformHostInstanceId>,
        generation: PlatformHostGeneration,
    },
    NotObserved {
        value: T,
        requirement: PlatformRuntimeHostRequirement,
        provider: PlatformHostInstanceId,
        generation: PlatformHostGeneration,
    },
    Degraded {
        value: T,
        provider: PlatformHostInstanceId,
        generation: PlatformHostGeneration,
    },
    Ready {
        value: T,
        provider: PlatformHostInstanceId,
        generation: PlatformHostGeneration,
    },
}

impl<T> PlatformRuntimeCapabilityStatus<T> {
    // Ready 同时携带 provider/generation/value；其它状态即使有静态 value 也不能当作已就绪。
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}
