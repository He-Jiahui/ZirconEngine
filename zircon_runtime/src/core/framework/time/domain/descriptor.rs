use super::{ClockDomainId, ClockDomainUnit};

/// Immutable entry in the canonical clock-domain taxonomy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClockDomainDescriptor {
    id: ClockDomainId,
    unit: ClockDomainUnit,
}

impl ClockDomainDescriptor {
    pub(crate) const fn new(id: ClockDomainId, unit: ClockDomainUnit) -> Self {
        Self { id, unit }
    }

    pub const fn id(self) -> ClockDomainId {
        self.id
    }

    pub const fn unit(self) -> ClockDomainUnit {
        self.unit
    }
}
