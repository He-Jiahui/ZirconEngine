//! Solari 实验路径的资格与退化报告；provider 实例和 pass 执行由 graphics 层管理。

#[path = "capability.rs"]
mod capability;
#[path = "settings.rs"]
mod settings;
#[path = "status.rs"]
mod status;

pub use capability::SolariCapabilityRequirement;
pub use settings::SolariSettings;
pub use status::{
    SolariDegradationReason, SolariProviderAvailability, SolariRuntimeDegradation,
    SolariRuntimeReport, SolariRuntimeStatus,
};
