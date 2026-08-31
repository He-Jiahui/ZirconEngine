/// A generational storage handle that is meaningful only to its owning `World`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct InternalEntity {
    index: u32,
    generation: u32,
}

impl InternalEntity {
    pub(crate) const INVALID_INDEX: u32 = u32::MAX;
    pub(crate) const PLACEHOLDER: Self = Self::new(Self::INVALID_INDEX, 0);

    pub(crate) const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub(crate) const fn index(self) -> u32 {
        self.index
    }

    pub(crate) const fn generation(self) -> u32 {
        self.generation
    }
}
