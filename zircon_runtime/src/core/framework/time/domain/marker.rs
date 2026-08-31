use super::ClockDomainId;
use crate::core::framework::time::{Fixed, MonotonicReal, Virtual};

/// Associates a `Time<T>` context with one canonical clock domain.
pub trait ClockDomainMarker {
    const CLOCK_DOMAIN: ClockDomainId;
}

impl ClockDomainMarker for MonotonicReal {
    const CLOCK_DOMAIN: ClockDomainId = ClockDomainId::MonotonicReal;
}

impl ClockDomainMarker for Virtual {
    const CLOCK_DOMAIN: ClockDomainId = ClockDomainId::WorldVirtual;
}

impl ClockDomainMarker for Fixed {
    const CLOCK_DOMAIN: ClockDomainId = ClockDomainId::WorldFixed;
}
