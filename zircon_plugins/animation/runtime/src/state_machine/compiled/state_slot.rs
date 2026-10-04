//! 编译状态机内的稠密状态行号；不能跨另一状态机或另一修订复用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StateSlot(u32);

impl StateSlot {
    pub(super) fn new(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    pub(super) fn index(self) -> usize {
        self.0 as usize
    }
}
