use super::{ClockDomainId, ClockDomainRegistry, ClockDomainUnit};

/// Runtime identity attached to a time value.
///
/// `epoch` changes when that domain's semantics are reconfigured. `source_generation` changes
/// when the underlying source is rebased, such as after activation or a lifecycle discontinuity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClockDomainStamp {
    id: ClockDomainId,
    unit: ClockDomainUnit,
    epoch: u64,
    source_generation: u64,
}

impl ClockDomainStamp {
    pub(crate) const fn initial(id: ClockDomainId) -> Self {
        let descriptor = ClockDomainRegistry::builtin().descriptor(id);
        Self {
            id: descriptor.id(),
            unit: descriptor.unit(),
            epoch: 0,
            source_generation: 0,
        }
    }

    pub const fn id(self) -> ClockDomainId {
        self.id
    }

    pub const fn unit(self) -> ClockDomainUnit {
        self.unit
    }

    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    pub const fn source_generation(self) -> u64 {
        self.source_generation
    }

    pub(crate) fn bump_epoch(&mut self) {
        self.epoch = self.epoch.saturating_add(1);
    }

    pub(crate) fn set_source_generation(&mut self, source_generation: u64) {
        self.source_generation = source_generation;
    }
}
