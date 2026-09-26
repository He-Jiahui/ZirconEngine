use serde::{Deserialize, Serialize};

/// 从标准化指针事件推导的阶段，供组件状态、默认交互及派发结果共用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerActivationPhase {
    #[default]
    None,
    PrimaryPress,
    PrimaryRelease,
    SecondaryPress,
    SecondaryRelease,
    MiddlePress,
    MiddleRelease,
    Hover,
    Scroll,
}
