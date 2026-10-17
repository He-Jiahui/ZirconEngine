//! 窗口输入的身份、时钟和修饰键元数据；类型化 ID 的构造器只包装输入值，不查重或确认实体存在。
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
#[serde(rename_all = "snake_case")]
/// 输入指针来源；is_touch_like 将 Touch 与 Pen 归为触控类来源。
pub enum UiPointerSource {
    #[default]
    Mouse,
    Touch,
    Pen,
    Unknown,
}

impl UiPointerSource {
    /// 判断来源是否为 Touch 或 Pen。
    pub const fn is_touch_like(self) -> bool {
        matches!(self, Self::Touch | Self::Pen)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 单调时钟时间戳，单位为微秒；不是墙上时钟时间。
pub struct UiInputTimestamp {
    pub monotonic_micros: u64,
}

impl UiInputTimestamp {
    /// 用单调时钟微秒值构造时间戳。
    pub const fn from_micros(monotonic_micros: u64) -> Self {
        Self { monotonic_micros }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
/// 单个输入事件采样到的修饰键与锁定键状态。
pub struct UiInputModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

/// 平台输入进入共享分发器时携带的时序与来源上下文；缺省身份留空，由窗口适配层按实际来源补齐。
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
    pub pointer_source: UiPointerSource,
    pub modifiers: UiInputModifiers,
    pub synthetic: bool,
}

impl UiInputEventMetadata {
    /// 初始化时序与序列，其他身份字段留空，修饰键清零，pointer source 默认 Mouse。
    pub const fn new(timestamp: UiInputTimestamp, sequence: UiInputSequence) -> Self {
        Self {
            timestamp,
            sequence,
            user_id: None,
            device_id: None,
            window_id: None,
            surface_id: None,
            pointer_id: None,
            pointer_source: UiPointerSource::Mouse,
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
