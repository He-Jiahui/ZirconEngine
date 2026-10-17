//! 虚拟几何插件的提取、逐相机准备与提交后回读契约；内部状态由框架按相机历史键管理。
#[path = "extract_output.rs"]
mod extract_output;
#[path = "gpu_completion.rs"]
mod gpu_completion;
#[path = "prepare_input.rs"]
mod prepare_input;
#[path = "prepare_output.rs"]
mod prepare_output;
#[path = "provider.rs"]
mod provider;
#[path = "../provider_registration.rs"]
mod provider_registration;
#[path = "../runtime_feedback.rs"]
mod runtime_feedback;
#[path = "runtime_state.rs"]
mod runtime_state;
#[path = "runtime_stats.rs"]
mod runtime_stats;
#[path = "runtime_update.rs"]
mod runtime_update;
#[path = "source_admission.rs"]
mod source_admission;

pub use extract_output::VirtualGeometryRuntimeExtractOutput;
pub use gpu_completion::VirtualGeometryGpuCompletion;
pub use prepare_input::VirtualGeometryRuntimePrepareInput;
pub use prepare_output::VirtualGeometryRuntimePrepareOutput;
pub use provider::VirtualGeometryRuntimeProvider;
pub use provider_registration::VirtualGeometryRuntimeProviderRegistration;
pub use runtime_feedback::VirtualGeometryRuntimeFeedback;
pub use runtime_state::VirtualGeometryRuntimeState;
pub use runtime_stats::VirtualGeometryRuntimeStats;
pub use runtime_update::VirtualGeometryRuntimeUpdate;

pub(crate) use source_admission::build_admitted_or_owned_extract;
pub use source_admission::{
    virtual_geometry_indexed_source_rows, VirtualGeometryAdmittedExtract,
    VirtualGeometryIndexedAuthorRun, VirtualGeometryIndexedSourceAdmission,
    VirtualGeometryIndexedSourceRow, VirtualGeometryPrimitiveAdmission,
    VirtualGeometrySourceAdmission, VirtualGeometrySourceDependency, VirtualGeometrySourceLoader,
};
