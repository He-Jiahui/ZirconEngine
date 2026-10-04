//! 一个已编译骨架目标表内的稠密行号；不能把裸行号解释为全局骨骼身份。
/// Dense runtime row assigned to one resolved animation target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TargetSlot(u32);

impl TargetSlot {
    pub(super) fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> u32 {
        self.0
    }
}
