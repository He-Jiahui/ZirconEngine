#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InterfaceSlot(u32);

impl InterfaceSlot {
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}
