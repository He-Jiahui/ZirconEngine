//! Shared host limits for each runtime output family.
//!
//! 本模块集中定义各 Runtime 输出类别共用的 Host 预算常量。
//!
//! 数值直接由 `zircon_runtime_interface` 对应的 `ZR_RUNTIME_*_OUTPUT_LIMIT_V1` 提供，避免
//! App 与 Editor 分别维护不同的 ABI 上限；Profile 响应额外启用已知集合预检。

use zircon_runtime_interface::{
    ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1, ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1, ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1,
    ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1, ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
};

use super::RuntimeForeignOutputBudget;

/// Host 请求队列输出的 ABI 预算。
pub const HOST_REQUEST_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1);
/// Profile 控制响应预算，并在类型化解码前检查文件和已知集合数量。
pub const PROFILE_RESPONSE_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1)
        .preflight_profile_files();
/// Runtime 操作结果输出的 ABI 预算。
pub const OPERATION_RESULT_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1);
/// 插件事件投递批次输出的 ABI 预算。
pub const PLUGIN_EVENT_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1);
/// World 查询结果输出的 ABI 预算。
pub const WORLD_QUERY_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1);
/// World 失效通知批次输出的 ABI 预算。
pub const WORLD_INVALIDATION_OUTPUT_BUDGET: RuntimeForeignOutputBudget =
    RuntimeForeignOutputBudget::from_interface(ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1);
