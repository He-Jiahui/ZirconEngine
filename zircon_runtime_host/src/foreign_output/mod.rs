//! Bounded decoding, ownership, metrics, and protocol state for runtime-owned outputs.

mod budget;
mod decode;
mod error;
mod item_count;
mod kind;
mod metrics;
mod owned_buffer;
mod policy;
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
    release_owned_buffer, release_owned_buffer_after_error, release_owned_buffer_after_result,
    validate_owned_buffer, validate_owned_buffer_releasing_on_error,
};
pub use policy::{
    HOST_REQUEST_OUTPUT_BUDGET, OPERATION_RESULT_OUTPUT_BUDGET, PLUGIN_EVENT_OUTPUT_BUDGET,
    PROFILE_RESPONSE_OUTPUT_BUDGET, WORLD_INVALIDATION_OUTPUT_BUDGET, WORLD_QUERY_OUTPUT_BUDGET,
};
pub use state::RuntimeForeignOutputState;

#[cfg(test)]
mod tests;
