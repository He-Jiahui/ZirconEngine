use super::ClockDomainId;
use crate::core::framework::time::{Fixed, MonotonicReal, Virtual};

/// 为 [Time](crate::core::framework::time::Time) 的上下文声明时钟域类别。
///
/// # 实现者责任
///
/// 构造 Time 时，CLOCK_DOMAIN 决定初始 stamp 的域和单位；应选择与上下文语义一致的类别。
/// 内建实现分别映射 MonotonicReal、Virtual 和 Fixed，推进权仍由 Runtime 或 World 持有。
/// 自定义实现不会改变 Time 的 Duration 数值表示，也不会自动校验输入值的来源。
///
/// # 使用边界
///
/// 泛型上下文和 stamp 便于调用方区分时钟；将 delta 提取为 Duration 后，仍须自行维护域的一致性。
/// 此 trait 不包含时钟采样、服务注册或运行时代际更新。
///
/// Associates a `Time<T>` context with one canonical clock domain.
pub trait ClockDomainMarker {
    const CLOCK_DOMAIN: ClockDomainId;
}

impl ClockDomainMarker for MonotonicReal {
    const CLOCK_DOMAIN: ClockDomainId = ClockDomainId::MonotonicReal;
}

impl ClockDomainMarker for Virtual {
    const CLOCK_DOMAIN: ClockDomainId = ClockDomainId::WorldVirtual;
}

impl ClockDomainMarker for Fixed {
    const CLOCK_DOMAIN: ClockDomainId = ClockDomainId::WorldFixed;
}
