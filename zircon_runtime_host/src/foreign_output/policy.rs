//! Shared host limits for each runtime output family.

use std::time::Duration;

use super::RuntimeForeignOutputBudget;

pub const HOST_REQUEST_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::new(256 * 1024, 256, Duration::from_millis(10)).allow_empty();
pub const PROFILE_RESPONSE_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::new(16 * 1024 * 1024, 65_536, Duration::from_millis(250))
        .allow_empty();
pub const OPERATION_RESULT_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::new(1024 * 1024, 16_384, Duration::from_millis(25));
pub const PLUGIN_EVENT_OUTPUT_BUDGET: RuntimeForeignOutputBudget = RuntimeForeignOutputBudget::new(
    zircon_runtime_interface::ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
    zircon_runtime_interface::ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    Duration::from_millis(10),
)
.allow_empty();
pub const WORLD_QUERY_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::new(1024 * 1024, 16_384, Duration::from_millis(25));
pub const WORLD_INVALIDATION_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::new(1024 * 1024, 16_384, Duration::from_millis(25)).allow_empty();
