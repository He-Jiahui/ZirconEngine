use serde::{Deserialize, Serialize};

macro_rules! define_vm_handle {
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

define_vm_handle!(PluginSlotId);
define_vm_handle!(HostHandle);
