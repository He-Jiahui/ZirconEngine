use std::num::NonZeroU64;

macro_rules! window_state_generation {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(NonZeroU64);

        impl $name {
            pub(crate) const fn initial() -> Self {
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
    };
}

window_state_generation!(WindowCreateGeneration);
window_state_generation!(WindowRequestedGeneration);
window_state_generation!(WindowEffectiveGeneration);
