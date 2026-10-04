//! 高级产品特性的声明与资格报告；真实 pass 执行由图形运行时和 provider 拥有。

mod feature;
mod provider_report;
mod runtime_plan;

pub use feature::AdvancedRenderFeature;
pub use provider_report::{
    AdvancedProviderAvailability, AdvancedProviderReport, AdvancedProviderStatus,
    AdvancedRenderDegradation, AdvancedRenderDegradationReason,
};
pub use runtime_plan::AdvancedProfileRuntimePlan;
