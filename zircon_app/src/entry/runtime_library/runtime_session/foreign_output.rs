pub(super) use zircon_runtime_host::foreign_output::{
    RuntimeForeignOutputBudget as ForeignOutputBudget,
    RuntimeForeignOutputKind as ForeignOutputKind, RuntimeForeignOutputState as ForeignOutputState,
    HOST_REQUEST_OUTPUT_BUDGET, OPERATION_RESULT_OUTPUT_BUDGET, PLUGIN_EVENT_OUTPUT_BUDGET,
    PROFILE_RESPONSE_OUTPUT_BUDGET,
    RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH as FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH,
};

#[cfg(test)]
mod performance_tests;
#[cfg(test)]
mod tests;
