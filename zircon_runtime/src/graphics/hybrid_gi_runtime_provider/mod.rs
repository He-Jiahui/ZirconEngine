//! 混合 GI 插件的运行时边界：注册共享 provider，按相机历史键持有状态，并在帧准备与提交后交换侧带。
#[path = "gpu_completion.rs"]
mod gpu_completion;
#[path = "prepare_input.rs"]
mod prepare_input;
#[path = "prepare_output.rs"]
mod prepare_output;
#[path = "provider.rs"]
mod provider;
#[path = "provider_registration.rs"]
mod provider_registration;
#[path = "runtime_feedback.rs"]
mod runtime_feedback;
#[path = "runtime_state.rs"]
mod runtime_state;
#[path = "runtime_stats.rs"]
mod runtime_stats;
#[path = "runtime_update.rs"]
mod runtime_update;

pub use gpu_completion::HybridGiGpuCompletion;
pub use prepare_input::HybridGiRuntimePrepareInput;
pub use prepare_output::HybridGiRuntimePrepareOutput;
pub use provider::HybridGiRuntimeProvider;
pub use provider_registration::HybridGiRuntimeProviderRegistration;
pub use runtime_feedback::HybridGiRuntimeFeedback;
pub use runtime_state::HybridGiRuntimeState;
pub use runtime_stats::HybridGiRuntimeStats;
pub use runtime_update::HybridGiRuntimeUpdate;
