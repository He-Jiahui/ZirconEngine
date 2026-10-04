use serde::{Deserialize, Serialize};

// PluginSlotId 只承载协调器分配的数字身份；代际由 slot record 保存，接口注册表据此筛选活动贡献。
macro_rules! define_vm_handle {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
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
