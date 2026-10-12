//! 连接构建时统一预设、启动装配所需的描述符和可用性状态投影。
//! 具体模块装载由 builtin 组合器执行；这里提供的状态供启动、导出和诊断消费。

#[path = "runtime_profile/assembly_presets.rs"]
mod assembly_presets;
#[path = "runtime_profile/availability.rs"]
mod availability;
#[path = "runtime_profile/availability_projection.rs"]
mod availability_projection;
#[path = "runtime_profile/availability_report.rs"]
mod availability_report;
#[path = "runtime_profile/descriptor.rs"]
mod descriptor;
#[path = "runtime_profile/feature_presets.rs"]
mod feature_presets;

pub use availability_projection::{
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityRow,
    RuntimePluginAvailabilitySummary,
};
pub use availability_report::{
    RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityEntry,
    RuntimePluginAvailabilityReport,
};
pub use descriptor::{RuntimeProfileDescriptor, RuntimeProfilePluginSelection};
pub use feature_presets::{RuntimeProfileFeaturePreset, RUNTIME_PROFILE_FEATURE_PRESETS};
