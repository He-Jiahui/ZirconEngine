//! 连接构建时统一预设、启动装配所需的描述符和可用性状态投影。
//! 具体模块装载由 builtin 组合器执行；这里提供的状态供启动、导出和诊断消费。

mod assembly_presets;
mod availability;
mod availability_projection;
mod availability_report;
mod descriptor;
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
