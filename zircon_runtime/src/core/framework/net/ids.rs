use serde::{Deserialize, Serialize};

macro_rules! define_net_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(u64);

        impl $name {
            pub fn new(raw: u64) -> Self {
                Self(raw)
            }

            pub fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

define_net_id!(NetListenerId);
define_net_id!(NetConnectionId);
define_net_id!(NetSessionId);
define_net_id!(NetRequestId);
define_net_id!(NetRouteId);
define_net_id!(NetDownloadId);
define_net_id!(NetObjectId);
