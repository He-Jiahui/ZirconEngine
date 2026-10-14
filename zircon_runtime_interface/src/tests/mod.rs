//! Runtime Interface crate 的内部测试根，仅由 crate 根的 `#[cfg(test)]` 模块加载。
//!
//! 各 `#[path]` 声明把测试按领域拆分；模板选择器用例从 `contracts` 子模块继续加载。

#[path = "abi_safety_contracts.rs"]
mod abi_safety_contracts;
#[path = "accessibility_contracts.rs"]
mod accessibility_contracts;
#[path = "binding_conversion_contracts.rs"]
mod binding_conversion_contracts;
#[path = "binding_value_contracts.rs"]
mod binding_value_contracts;
#[path = "boundary.rs"]
mod boundary;
#[path = "contracts.rs"]
mod contracts;
#[path = "dispatch_reply_contracts.rs"]
mod dispatch_reply_contracts;
#[path = "editor_design_tokens.rs"]
mod editor_design_tokens;
#[path = "input_response_contracts.rs"]
mod input_response_contracts;
#[path = "layout_engine_contracts.rs"]
mod layout_engine_contracts;
#[path = "model_context_contracts.rs"]
mod model_context_contracts;
#[path = "model_schema_contracts.rs"]
mod model_schema_contracts;
#[path = "pipeline_contracts.rs"]
mod pipeline_contracts;
#[path = "plugin_api_contracts.rs"]
mod plugin_api_contracts;
#[path = "reflect_contracts.rs"]
mod reflect_contracts;
#[path = "reflect_value_budget_contracts.rs"]
mod reflect_value_budget_contracts;
#[path = "render_contracts.rs"]
mod render_contracts;
#[path = "render_parity_key_reuse_perf_contracts.rs"]
mod render_parity_key_reuse_performance_contracts;
#[path = "resource_contracts.rs"]
mod resource_contracts;
#[path = "runtime_operation.rs"]
mod runtime_operation;
#[path = "runtime_owned_result.rs"]
mod runtime_owned_result;
#[path = "surface_frame_contracts.rs"]
mod surface_frame_contracts;
#[path = "ui_contract_spine.rs"]
mod ui_contract_spine;
#[path = "ui_dispatch_error_contracts.rs"]
mod ui_dispatch_error_contracts;
#[path = "ui_ecs_node_lookup_contracts.rs"]
mod ui_ecs_node_lookup_contracts;
#[path = "ui_ecs_projection_contracts.rs"]
mod ui_ecs_projection_contracts;
#[path = "ui_geometry_metrics.rs"]
mod ui_geometry_metrics;
#[path = "ui_layout.rs"]
mod ui_layout;
#[path = "ui_painter_style_contracts.rs"]
mod ui_painter_style_contracts;
#[path = "ui_skin_contracts.rs"]
mod ui_skin_contracts;
#[path = "ui_theme_contracts.rs"]
mod ui_theme_contracts;
#[path = "ui_v2_contracts.rs"]
mod ui_v2_contracts;
#[path = "window_input_contracts.rs"]
mod window_input_contracts;
#[path = "window_runtime_event_adapter_contracts.rs"]
mod window_runtime_event_adapter_contracts;
#[path = "window_transient_contracts.rs"]
mod window_transient_contracts;
#[path = "world_sync_contracts.rs"]
mod world_sync_contracts;

#[path = "window_runtime_event_borrowing_contracts.rs"]
mod window_runtime_event_borrowing_contracts;
