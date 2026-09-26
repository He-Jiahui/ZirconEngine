use std::num::NonZeroU64;

/// 每次应用事实发布递增的版本；观察者据此识别旧快照，不以操作 ID 代替版本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApplicationLifecycleGeneration(NonZeroU64);

impl ApplicationLifecycleGeneration {
    pub const fn initial() -> Self {
        Self(NonZeroU64::MIN)
    }

    pub(crate) const fn next(self) -> Option<Self> {
        match self.0.get().checked_add(1) {
            Some(raw) => match NonZeroU64::new(raw) {
                Some(raw) => Some(Self(raw)),
                None => None,
            },
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}
