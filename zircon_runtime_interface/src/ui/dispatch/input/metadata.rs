use serde::{Deserialize, Serialize};

macro_rules! define_u64_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

macro_rules! define_string_id {
    ($name:ident) => {
        #[derive(
            Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }
        }
    };
}

define_u64_id!(UiUserId);
define_u64_id!(UiDeviceId);
define_u64_id!(UiPointerId);
define_u64_id!(UiDragSessionId);
define_u64_id!(UiInputSequence);

define_string_id!(UiWindowId);
define_string_id!(UiSurfaceId);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiInputTimestamp {
    pub monotonic_micros: u64,
}

impl UiInputTimestamp {
    pub const fn from_micros(monotonic_micros: u64) -> Self {
        Self { monotonic_micros }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputEventMetadata {
    pub timestamp: UiInputTimestamp,
    pub sequence: UiInputSequence,
    pub user_id: Option<UiUserId>,
    pub device_id: Option<UiDeviceId>,
    pub window_id: Option<UiWindowId>,
    pub surface_id: Option<UiSurfaceId>,
    pub pointer_id: Option<UiPointerId>,
    pub modifiers: UiInputModifiers,
    pub synthetic: bool,
}

impl UiInputEventMetadata {
    pub const fn new(timestamp: UiInputTimestamp, sequence: UiInputSequence) -> Self {
        Self {
            timestamp,
            sequence,
            user_id: None,
            device_id: None,
            window_id: None,
            surface_id: None,
            pointer_id: None,
            modifiers: UiInputModifiers {
                shift: false,
                control: false,
                alt: false,
                super_key: false,
                caps_lock: false,
                num_lock: false,
            },
            synthetic: false,
        }
    }
}
