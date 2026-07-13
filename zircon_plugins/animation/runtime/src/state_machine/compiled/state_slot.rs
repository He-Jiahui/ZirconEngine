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
