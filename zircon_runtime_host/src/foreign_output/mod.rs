//! Bounded decoding, ownership, metrics, and protocol state for runtime-owned outputs.
//!
//! 本模块负责 Runtime 拥有输出的限额解码、释放、指标与协议状态。
//!
//! App 的 `entry::runtime_library::runtime_session` 与 Editor 的
//! `core::gateway::session` 在调用 Runtime ABI 后，将结果交给本模块验证；非空分配始终由
//! 创建它的会话释放。解码预算源自 `zircon_runtime_interface`，协议违规会熔断共享会话状态。

#[path = "budget.rs"]
mod budget;
#[path = "decode.rs"]
mod decode;
#[path = "error.rs"]
mod error;
#[path = "item_count.rs"]
mod item_count;
#[path = "kind.rs"]
mod kind;
#[path = "metrics.rs"]
mod metrics;
#[path = "owned_buffer.rs"]
mod owned_buffer;
#[path = "policy.rs"]
mod policy;
#[path = "state.rs"]
mod state;

pub use budget::{RuntimeForeignOutputBudget, RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH};
pub use error::{RuntimeForeignOutputError, RuntimeForeignOutputErrorKind};
pub use item_count::{
    json_value_item_count, operation_result_item_count, plugin_event_batch_item_count,
    profile_control_response_item_count, world_invalidation_item_count, world_query_item_count,
};
pub use kind::RuntimeForeignOutputKind;
pub use metrics::{RuntimeForeignOutputMetrics, RuntimeForeignOutputMetricsSnapshot};
pub use owned_buffer::{
    release_owned_result, release_owned_result_after_error, release_owned_result_after_result,
    validate_owned_result, validate_owned_result_releasing_on_error, RuntimeOwnedOutputReleaser,
};
pub use policy::{
    HOST_REQUEST_OUTPUT_BUDGET, OPERATION_RESULT_OUTPUT_BUDGET, PLUGIN_EVENT_OUTPUT_BUDGET,
    PROFILE_RESPONSE_OUTPUT_BUDGET, WORLD_INVALIDATION_OUTPUT_BUDGET, WORLD_QUERY_OUTPUT_BUDGET,
};
pub use state::RuntimeForeignOutputState;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
