//! 时钟域的类别、静态单位映射和随快照传递的代际信息。
//!
//! 框架时间模块统一导出这些契约：ClockDomainRegistry 提供静态分类，
//! ClockDomainMarker 将 Time 的上下文与域类别关联，ClockDomainStamp 记录类别及两级代际。
//! 本模块不采样或推进时间；外层 Runtime 持有真实时钟，World 持有虚拟与固定步时钟。
//! ECS 消费已选择的快照，不在系统调用期间重新决定来源。
mod descriptor;
mod id;
mod marker;
mod registry;
mod stamp;
mod unit;

pub use descriptor::ClockDomainDescriptor;
pub use id::ClockDomainId;
pub use marker::ClockDomainMarker;
pub use registry::ClockDomainRegistry;
pub use stamp::ClockDomainStamp;
pub use unit::ClockDomainUnit;
