//! 状态转换描述、进度和交叉淡入权重的公共导出层。
mod interruption_policy;
mod transition_desc;
mod transition_request;
mod transition_runtime;
mod transition_state;
mod transition_weights;

pub use interruption_policy::InterruptionPolicy;
pub use transition_desc::TransitionDesc;
pub use transition_request::TransitionRequest;
pub use transition_runtime::TransitionRuntime;
pub use transition_state::TransitionState;
pub use transition_weights::TransitionWeights;
