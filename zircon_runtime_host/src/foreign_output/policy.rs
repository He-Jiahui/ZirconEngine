//! Shared host limits for each runtime output family.

use zircon_runtime_interface::{
    ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1, ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1, ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1, ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
};

use super::RuntimeForeignOutputBudget;

pub const HOST_REQUEST_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1);
pub const PROFILE_RESPONSE_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1);
pub const OPERATION_RESULT_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1);
pub const PLUGIN_EVENT_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1);
pub const WORLD_QUERY_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1);
pub const WORLD_INVALIDATION_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1);
