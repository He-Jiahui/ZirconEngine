//! 转换中的稠密状态标识，只在创建它的状态机布局内有意义。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionState(u32);

impl TransitionState {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u32 {
        self.0
    }
}
