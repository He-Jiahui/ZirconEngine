use super::{ClockDomainDescriptor, ClockDomainId, ClockDomainUnit};

/// Versioned static taxonomy for engine clock domains.
///
/// The registry deliberately contains no mutable state or allocation. Runtime clock instances
/// carry their own epoch and source-generation stamp instead of consulting a global service.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClockDomainRegistry;

impl ClockDomainRegistry {
    pub const VERSION: u16 = 1;

    pub const fn builtin() -> Self {
        Self
    }

    pub const fn version(self) -> u16 {
        Self::VERSION
    }

    pub const fn descriptor(self, id: ClockDomainId) -> ClockDomainDescriptor {
        match id {
            ClockDomainId::MonotonicReal => {
                ClockDomainDescriptor::new(id, ClockDomainUnit::Duration)
            }
            ClockDomainId::WallUtc => {
                ClockDomainDescriptor::new(id, ClockDomainUnit::UnixTimestamp)
            }
            ClockDomainId::WorldVirtual
            | ClockDomainId::WorldFixed
            | ClockDomainId::Input
            | ClockDomainId::Render
            | ClockDomainId::Audio
            | ClockDomainId::Network
            | ClockDomainId::Media
            | ClockDomainId::EditorPreview => {
                ClockDomainDescriptor::new(id, ClockDomainUnit::Duration)
            }
        }
    }
}
