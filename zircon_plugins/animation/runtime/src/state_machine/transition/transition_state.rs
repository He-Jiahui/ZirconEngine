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
