use super::BoundedKeyedIoTicket;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalAdmissionEpoch(pub(crate) u64);

impl GlobalAdmissionEpoch {
    pub const fn initial() -> Self {
        Self(0)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct BoundedKeyedIoFence {
    epoch: GlobalAdmissionEpoch,
    ticket: BoundedKeyedIoTicket,
}

impl BoundedKeyedIoFence {
    pub(crate) const fn new(epoch: GlobalAdmissionEpoch, ticket: BoundedKeyedIoTicket) -> Self {
        Self { epoch, ticket }
    }

    pub const fn epoch(&self) -> GlobalAdmissionEpoch {
        self.epoch
    }

    pub fn ticket(&self) -> BoundedKeyedIoTicket {
        self.ticket.clone()
    }
}
