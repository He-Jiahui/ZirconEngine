//! 将静态 PlatformCapabilityReport 与当前平台宿主快照合并为 admission 可用的
//! runtime 状态；这里不创建宿主，也不绕过 PlatformManager/PlatformDriver。
mod report;
mod status;

pub use report::PlatformRuntimeCapabilityReport;
pub use status::{PlatformRuntimeCapabilityStatus, PlatformRuntimeHostRequirement};
