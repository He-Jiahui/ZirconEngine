//! App 会话对 zircon_runtime_host 外来输出预算与隔离状态的局部命名。
//! 这些类型控制运行时分配物的验证和释放；会话调用方不得绕过共享宿主边界。

pub(super) use zircon_runtime_host::foreign_output::{
    RuntimeForeignOutputBudget as ForeignOutputBudget,
    RuntimeForeignOutputKind as ForeignOutputKind, RuntimeForeignOutputState as ForeignOutputState,
    HOST_REQUEST_OUTPUT_BUDGET, OPERATION_RESULT_OUTPUT_BUDGET, PLUGIN_EVENT_OUTPUT_BUDGET,
    PROFILE_RESPONSE_OUTPUT_BUDGET,
    RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH as FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH,
};

#[cfg(test)]
#[path = "foreign_output/tests/performance_tests.rs"]
mod performance_tests;
#[cfg(test)]
#[path = "foreign_output/tests/cases.rs"]
mod tests;
