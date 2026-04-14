use serde::{Deserialize, Serialize};

macro_rules! define_handle {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

define_handle!(WorldHandle);
define_handle!(AssetHandle);
define_handle!(PluginSlotId);
define_handle!(HostHandle);
