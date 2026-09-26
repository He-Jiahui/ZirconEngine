use super::BoundedKeyedIoTicket;

/// 键控通道的全局接纳分段；栅栏使后续同键写入不能跨段合并旧义务。
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

/// 栅栏的顺序边界与完成凭证；调用方通过 ticket() 等待此前接纳的写入义务。
/// 栅栏成功还取决于被固定的前置条目或其有效后继，不能仅观察后续新写入。
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
