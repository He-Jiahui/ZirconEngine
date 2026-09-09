---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/world_sync/mod.rs
  - zircon_runtime_interface/src/project/mod.rs
  - zircon_runtime_interface/src/serialization/mod.rs
  - zircon_runtime_interface/src/reflect/mod.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/abi/host_api_shape.rs
  - zircon_runtime_interface/src/runtime_api/frame/mod.rs
  - zircon_runtime_interface/src/runtime_api/host/mod.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/mod.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/ui/mod.rs
implementation_files:
  - zircon_runtime_interface/src/world_sync
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/serialization
  - zircon_runtime_interface/src/reflect
  - zircon_runtime_interface/src/resource
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/ui
plan_sources:
  - user: 2026-09-09 API 公开接口覆盖审计
tests:
  - zircon_runtime_interface/src/tests/world_sync_contracts.rs
  - zircon_runtime_interface/src/serialization/tests
  - zircon_runtime_interface/src/reflect/schema_catalog/tests.rs
  - zircon_runtime_interface/src/tests/reflect_contracts.rs
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/surface_frame_contracts.rs
  - zircon_runtime_interface/src/tests/window_input_contracts
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/session_entry_points.rs
doc_type: api-reference
---

# Runtime Interface API 清单

本页以 `zircon_runtime_interface` crate 根的 `pub use` 为准，列出当前可被外部 crate 通过 `zircon_runtime_interface::...` 访问的符号。它是 lockstep FFI/DTO 边界，不等同于稳定的第三方 ABI；版本后缀（如 `V1`、`V8`）是布局和语义的一部分。`pub(crate)`、测试 fixture 与未从 crate 根导出的模块不在本表。

## 使用规则

- 先检查 `ZIRCON_RUNTIME_API_VERSION_V8` 与插件/宿主支持的 ABI 版本，再读取函数表。
- 所有 `ZrOwned*`、`ZrByte*` 和 allocation handle 都有明确释放者；跨 DLL 释放必须调用对应函数指针。
- `#[repr(C)]` 结构的新增字段只能通过版本化类型或尾部扩展完成，不能改变现有字段顺序。
- 结果中的 `ZrStatus`、错误码和长度字段必须先校验，再解引用指针。

## 符号总表

| 模块 | 完整路径 | 类型/用途摘要 |
| --- | --- | --- |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_ACCESSIBILITY_ACTION_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_ACCESSIBILITY_TREE_OUTPUT_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_CLIPBOARD_RESULT_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_CLIPBOARD_TEXT_MAX_ENCODED_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_FRAME_MAX_DIMENSION_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_FRAME_MAX_RGBA_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_JSON_MAX_NESTING_DEPTH_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_OPERATION_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_PLUGIN_EVENT_SUBSCRIBE_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_PROFILE_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_SESSION_PROFILE_MAX_ENCODED_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_VIEWPORT_CAMERA_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_WORLD_QUERY_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZR_RUNTIME_WORLD_WATCH_REQUEST_LIMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `buffer` | `zircon_runtime_interface::buffer::ZrByteBufferRef` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `buffer` | `zircon_runtime_interface::buffer::ZrByteSlice` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `buffer` | `zircon_runtime_interface::buffer::ZrByteSliceError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `buffer` | `zircon_runtime_interface::buffer::ZrFreeBytesFn` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `buffer` | `zircon_runtime_interface::buffer::ZrOwnedByteBuffer` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `buffer` | `zircon_runtime_interface::buffer::ZrOwnedResultV2` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `buffer` | `zircon_runtime_interface::buffer::ZrRuntimePayloadLimitV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::EditorCommandExecutionContract` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::EditorCommandResourceBudget` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::EditorCommandResourceBudgetError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::EditorCommandResultCodecId` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::EditorCommandResultCodecIdError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::MAX_EDITOR_COMMAND_EXECUTION_TIME_MS` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::MAX_EDITOR_COMMAND_INPUT_BYTES` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_command_execution` | `zircon_runtime_interface::editor_command_execution::MAX_EDITOR_COMMAND_OUTPUT_BYTES` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_command_id` | `zircon_runtime_interface::editor_command_id::EditorCommandId` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `editor_command_id` | `zircon_runtime_interface::editor_command_id::EditorCommandIdError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `editor_contribution` | `zircon_runtime_interface::editor_contribution::SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_contribution` | `zircon_runtime_interface::editor_contribution::SerializedContributionBatch` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `editor_contribution` | `zircon_runtime_interface::editor_contribution::SerializedContributionBatchError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `editor_contribution` | `zircon_runtime_interface::editor_contribution::SerializedEditorContribution` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `handles` | `zircon_runtime_interface::handles::ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `handles` | `zircon_runtime_interface::handles::ZrRuntimeAllocationId` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `handles` | `zircon_runtime_interface::handles::ZrRuntimePluginHandle` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `handles` | `zircon_runtime_interface::handles::ZrRuntimeSessionHandle` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `handles` | `zircon_runtime_interface::handles::ZrRuntimeViewportHandle` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::hub_editor_focus_ack_path` | 公开辅助函数；执行输入校验、解析、路径计算或合并。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::hub_editor_focus_request_directory` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::hub_editor_focus_signal_path` | 公开辅助函数；执行输入校验、解析、路径计算或合并。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HUB_PROTOCOL_VERSION_V1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HUB_RECENT_PROJECT_LIMIT_V1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::hub_recent_project_path_key` | 公开辅助函数；执行输入校验、解析、路径计算或合并。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::hub_recent_projects_lock_path` | 公开辅助函数；执行输入校验、解析、路径计算或合并。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HUB_RECENT_PROJECTS_MAX_ENCODED_BYTES_V1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::hub_recent_projects_path` | 公开辅助函数；执行输入校验、解析、路径计算或合并。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::hub_recent_projects_path_from_home` | 公开辅助函数；执行输入校验、解析、路径计算或合并。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubEditorFocusAckDispositionV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubEditorFocusAckV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubEditorFocusSignalError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubEditorFocusSignalPathError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubEditorFocusSignalV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubEditorLaunchOutcomeV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubEditorMailboxV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubProtocolVersionV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsLoad` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsLoadDisposition` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsMutation` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsStore` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsStoreError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectsWritePolicy` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectTombstoneV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubRecentProjectV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubSessionToken` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::HubSessionTokenParseError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::merge_hub_recent_projects` | 公开辅助函数；执行输入校验、解析、路径计算或合并。 |
| `hub_protocol` | `zircon_runtime_interface::hub_protocol::windows_hub_recent_projects_mutex_name` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `manifest` | `zircon_runtime_interface::manifest::ZrPluginModuleDescriptorV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `manifest` | `zircon_runtime_interface::manifest::ZrPluginModuleKind` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `manifest` | `zircon_runtime_interface::manifest::ZrRuntimeTargetMode` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_RESOURCE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_NATIVE_SYSTEM_ACCESS_MODE_WRITE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_NATIVE_SYSTEM_THREAD_AFFINITY_MAIN_THREAD_ONLY_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_NATIVE_SYSTEM_THREAD_AFFINITY_WORKER_SAFE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_PLUGIN_ENTRY_SYMBOL_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_PLUGIN_ENTRY_SYMBOL_V3` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZR_PLUGIN_ENTRY_SYMBOL_V4` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrComponentDescV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrEventTypeId` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostApiV3` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostApiV4` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostAssetApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostAssetRequestFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostBridgeApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostBridgeCallFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostDiagnosticsApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostDiagnosticsEmitFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostDiagnosticsMetricFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostEcsApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostEcsApiV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostEventApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostEventDrainFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostEventEmitFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostRegisterComponentFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostRegisterSystemFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostRegisterSystemFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrHostSpawnCommandFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrNativeSystemAccessV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrNativeSystemInvokeFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginEntryFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginEntryFnV3` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginEntryFnV4` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginEntryReportV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginSnapshotRestoreFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginSnapshotSaveFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrPluginStateSnapshotApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrSystemRegistrationV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `plugin_api` | `zircon_runtime_interface::plugin_api::ZrSystemRegistrationV2` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `plugin_diagnostics` | `zircon_runtime_interface::plugin_diagnostics::RegistrationDiagnostic` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `plugin_diagnostics` | `zircon_runtime_interface::plugin_diagnostics::RegistrationDiagnosticSeverity` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `plugin_events` | `zircon_runtime_interface::plugin_events::ZrPluginEventCallbackFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_events` | `zircon_runtime_interface::plugin_events::ZrPluginEventCallbackRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `plugin_events` | `zircon_runtime_interface::plugin_events::ZrPluginEventCallbackResultV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `profiling` | `zircon_runtime_interface::profiling::CounterHotspotEntry` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `profiling` | `zircon_runtime_interface::profiling::CounterHotspotReport` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::HotspotEntry` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `profiling` | `zircon_runtime_interface::profiling::HotspotReport` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_CAPTURE_MAX_COUNTERS` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_CAPTURE_MAX_FRAME_BUDGET_MS` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_CAPTURE_MAX_FRAMES` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_CAPTURE_MAX_SPANS` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_COUNTER_HOTSPOTS_FILE` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_DEFAULT_FRAME_BUDGET_MS` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_DEFAULT_MAX_COUNTERS` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_DEFAULT_MAX_FRAMES` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_DEFAULT_MAX_SPANS` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_DEFAULT_OUTPUT_ROOT` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_DEFAULT_SESSION_ID` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_HOTSPOTS_FILE` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_SUMMARY_FILE` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_TIMELINE_NATIVE_FILE` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_TIMELINE_PERFETTO_FILE` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::PROFILE_UI_HOTSPOTS_FILE` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileCaptureConfig` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileControlCommand` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileControlRequest` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileControlResponse` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileCounterSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileFrameSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileRecorderRetentionSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileSampleRetentionSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ProfileSpanSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::RuntimeDiagnosticMeasurement` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::RuntimeDiagnosticSeriesSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::RuntimeDiagnosticsSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::RuntimeInputDiagnosticsSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::RuntimeRenderDeviceDiagnosticsSnapshot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::RuntimeSceneAssetReloadDiagnostics` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::UiHotspotAlert` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::UiHotspotReport` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::UiScenarioHotspot` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `profiling` | `zircon_runtime_interface::profiling::ZrRuntimeProfileControlFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::GatewaySessionIdentity` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::validate_runtime_api_v8_shape` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::validate_runtime_frame_rgba_shape` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_BUTTON_STATE_PRESSED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_BUTTON_STATE_RELEASED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_CLIPBOARD_RESULT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_CURSOR_ENTERED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_CURSOR_LEFT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_EDITOR_TRANSFORM_WRITE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_FILE_DRAG_DROP_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_IME_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_KEYBOARD_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_MOUSE_BUTTON_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_MOUSE_MOTION_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_MOUSE_WHEEL_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_TOUCH_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_VIEWPORT_CAMERA_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_VIEWPORT_RESIZED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_FETCH_FLAG_STREAMING_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_FILE_DRAG_CANCELLED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_FILE_DRAG_DROPPED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_FILE_DRAG_HOVERED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_FRAME_DEMAND_AFTER_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_FRAME_DEMAND_IDLE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_C_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_START_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_GET_API_SYMBOL_V8` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_CURSOR_HIDDEN_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_COMMIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_CURSOR_AREA_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_DELETE_SURROUNDING_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_DISABLED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_ENABLED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_PREEDIT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_REQUEST_DISABLE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_REQUEST_ENABLE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_IME_STATE_SURROUNDING_TEXT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_KEY_ACTION_PRESSED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_KEY_ACTION_RELEASED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_KEY_ACTION_TEXT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_MOUSE_WHEEL_COORDS_PRESENT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_NATIVE_SURFACE_KIND_NONE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_NATIVE_SURFACE_KIND_WIN32_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_TOUCH_PHASE_ENDED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_TOUCH_PHASE_MOVED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_TOUCH_PHASE_STARTED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_PERSPECTIVE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_BACKFACES_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_BOOL_FALSE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_BOOL_TRUE_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_BACKEND_SCALE_FACTOR_CHANGED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_CLOSE_REQUESTED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_MOVED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_OCCLUDED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_SCALE_FACTOR_CHANGED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_SURFACE_RECREATED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_STATUS_THEME_CHANGED_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_THEME_DARK_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_THEME_LIGHT_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZR_RUNTIME_WINDOW_THEME_UNKNOWN_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrHostApiV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeAccessibilityTreeRequestV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeApiV8` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeApiV8ShapeError` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeBindViewportSurfaceFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeBindViewportSurfaceRequestV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCancelViewportPickFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCaptureAccessibilityTreeFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCaptureFrameFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeClipboardHostRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeClipboardResultV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCreateSessionFnV3` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCursorGrabModeV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCursorHostRequestKindV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCursorHostRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeCursorPositionV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeDrainHostRequestsFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeDrainPluginEventsFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeDrainWorldInvalidationsFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeEditorTransformError` | 该模块的错误类型；用于分类失败原因并决定重试、回滚或终止。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeEditorTransformPhaseV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeEditorTransformWriteV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeEntityIdSliceV1` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeEventV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeFrameDemandV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeFrameRequestV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeFrameV2` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeGamepadRumbleRequestKindV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeGamepadRumbleRequestV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeGetApiFnV8` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeHarvestOperationFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeHighlightRenderAttributesV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeHighlightSetV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeHostFetchFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeHostFetchRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeHostRequestBatchV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeHostRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeImeCoordinateSpaceV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeImeCursorAreaV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeImeHostRequestKindV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeImeHostRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeImeSurroundingTextV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeImeTextRangeV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeNativeSurfaceTargetV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeOperationDetailKindV2` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeOperationHandle` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeOperationOutcomeV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeOperationPhase` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeOperationResultV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeOperationStatusV2` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeOperationSubmitRequestV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimePluginEventDeliveryBatchV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimePluginEventDeliveryV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimePluginEventSubscribeRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimePluginEventSubscriptionHandle` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimePollOperationFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimePollViewportPickFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimePresentViewportFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeQueryWorldFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeReleaseAllocationFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeRequestViewportPickFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeSessionConfigV3` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeSubmitHighlightSetFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeSubmitOperationFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeSubscribePluginEventFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeTickFrameFnV2` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeTransformV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeTranslatedEventV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeUiActionHostRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeUiHostRequestKindV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeUiHostRequestV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeUnbindViewportSurfaceFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeUnsubscribePluginEventFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeUnwatchWorldFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportCameraV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportMetricsV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportPickDispositionV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportPickPurposeV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportPickRequestV1` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportPickResultV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportPickTicket` | 不透明句柄/标识；用于跨调用关联资源，所有权和释放规则见模块文档。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportPixelV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeViewportSizeV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeWakeSinkV1` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `runtime_api` | `zircon_runtime_interface::runtime_api::ZrRuntimeWatchWorldFnV1` | 跨运行时/插件边界的函数指针或 API 表；负责注册、调用和生命周期协作。 |
| `script_diagnostics` | `zircon_runtime_interface::script_diagnostics::ScriptDiagnostic` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `script_diagnostics` | `zircon_runtime_interface::script_diagnostics::ScriptDiagnosticSeverity` | 诊断、性能采样或快照数据模型；用于观测、导出与回归比较。 |
| `script_diagnostics` | `zircon_runtime_interface::script_diagnostics::ScriptSourceLocation` | 公开数据类型；承载该模块的序列化字段、配置或运行时状态。 |
| `status` | `zircon_runtime_interface::status::ZrStatus` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `status` | `zircon_runtime_interface::status::ZrStatusCode` | 事件、请求、响应或状态模型；描述异步操作的阶段和结果。 |
| `version` | `zircon_runtime_interface::version::ZIRCON_RUNTIME_ABI_VERSION_V1` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `version` | `zircon_runtime_interface::version::ZIRCON_RUNTIME_ABI_VERSION_V2` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `version` | `zircon_runtime_interface::version::ZIRCON_RUNTIME_ABI_VERSION_V3` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |
| `version` | `zircon_runtime_interface::version::ZIRCON_RUNTIME_API_VERSION_V8` | 版本化 ABI 常量、大小上限或枚举值；调用方必须按版本契约使用。 |

## 调用骨架

`ZrRuntimeGetApiFnV8` 是动态库导出的 C 函数指针，参数是可选的 host table 指针，返回值是 runtime-owned（但调用方只借用）的 `ZrRuntimeApiV8` 指针。它没有版本参数，也不返回 Rust `Result`。调用方必须先检查空指针，再把表复制到自己的栈/堆内存，最后验证版本和精确大小；不要在动态库卸载后继续使用表中的函数指针。

```rust
use zircon_runtime_interface::{
    validate_runtime_api_v8_shape, ZrHostApiV1,
    ZrRuntimeApiV8, ZrRuntimeGetApiFnV8, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_API_VERSION_V8,
};
use zircon_runtime_interface::runtime_api::validate_runtime_host_api_v1_pointer;

/// `get_api` 和返回表只在动态库保持加载期间有效。
unsafe fn acquire_api(get_api: ZrRuntimeGetApiFnV8) -> Result<ZrRuntimeApiV8, &'static str> {
    let host = ZrHostApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    unsafe { validate_runtime_host_api_v1_pointer(&host as *const ZrHostApiV1) }
        .map_err(|_| "invalid host table")?;

    let foreign = unsafe { get_api(&host as *const ZrHostApiV1) };
    if foreign.is_null() {
        return Err("runtime returned a null API table");
    }
    if (foreign as usize) % core::mem::align_of::<ZrRuntimeApiV8>() != 0 {
        return Err("runtime returned a misaligned API table");
    }

    // 先复制，再验证；这样后续 shape 检查不依赖 foreign pointer 的生命周期。
    let api = unsafe { core::ptr::read_unaligned(foreign) };
    validate_runtime_api_v8_shape(&api).map_err(|_| "runtime API V8 shape mismatch")?;
    if api.abi_version != ZIRCON_RUNTIME_API_VERSION_V8 {
        return Err("unsupported runtime API version");
    }
    Ok(api)
}
```

上面的示例刻意没有调用任何 slot。生产代码还应检查必需 slot 是否为 `Some`，并把 `ZrStatus` 的 `diagnostics` 复制到宿主内存后再调用 `release_allocation`。`read_unaligned` 只有在动态库保证返回指针可读的调用窗口内才安全；若宿主不能证明该前提，应在进程隔离边界完成复制。

## 文档阅读约定

本页后续章节是字段级契约，不是第二份实现。每个示例都标注为“可复制入口”或“调用形状”：前者只使用公开构造器并可直接放入同一 feature 集合的 crate；后者省略宿主装配、错误映射或平台对象，必须以 rustdoc 为最终签名。所有 `V1`/`V2`/`V3`/`V8` 后缀都代表 wire/ABI 版本，不能通过 `transmute` 或手工改版本号混用。

## 版本、状态和内存模型

### 版本头

| 常量 | 当前值/用途 | 检查时机 |
| --- | --- | --- |
| `ZIRCON_RUNTIME_ABI_VERSION_V1` | 固定布局 DTO（handle、event、frame、surface 等） | 构造或接收每个 `repr(C)` DTO 时 |
| `ZIRCON_RUNTIME_ABI_VERSION_V2` | operation status、部分 ABI 扩展 | 读取对应 V2 结构前 |
| `ZIRCON_RUNTIME_ABI_VERSION_V3` | `ZrRuntimeSessionConfigV3` | `create_session` 前 |
| `ZIRCON_RUNTIME_API_VERSION_V8` | 函数表版本 | `get_api` 后、读取 slot 前 |

`ZrStatusCode` 的已知值为 `Ok`、`Error`、`UnsupportedVersion`、`InvalidArgument`、`NotFound`、`CapabilityDenied`、`Panic`、`BridgeNotEnabled` 和 `LimitExceeded`。未知 code 会折叠到 `Error`，因此宿主不能把未知值当成成功。`ZrStatus { code, diagnostics }` 的 diagnostics 是借用的 `ZrByteSlice`，只在当前调用内有效。

### 句柄与所有权

| 类型 | `repr`/字段 | 语义 | 释放方式 |
| --- | --- | --- | --- |
| `ZrRuntimeSessionHandle` | `#[repr(transparent)] u64` | runtime session 身份；`0` 无效 | `destroy_session(session)` |
| `ZrRuntimeViewportHandle` | `#[repr(transparent)] u64` | session 内 viewport 身份；默认值为 `1` | 随 session 或 surface 生命周期 |
| `ZrRuntimeAllocationId` | `#[repr(transparent)] u64` | `ZrOwnedResultV2` 的分配所有权 | `release_allocation(session, id)` |
| `ZrRuntimePluginHandle` | `#[repr(transparent)] u64` | plugin 注册身份 | 对应 plugin API 注销流程 |
| `ZrByteSlice` | `data: *const u8`, `len: usize` | 调用方借用输入 | 不由 runtime 释放 |
| `ZrOwnedResultV2` | `data: *const u8`, `len: u64`, `allocation` | runtime 拥有的只读输出 | 消费后调用 release slot |
| `ZrOwnedByteBuffer` | `data`, `len`, `capacity`, `owner_token`, `free` | host callback 返回的可释放缓冲 | 调用内嵌 `free` 函数指针 |

`ZrByteSlice::checked_slice(limit)` 会拒绝非零长度空指针、超过 address-space 的长度和超过契约上限的输入；它不会验证指针指向的页是否映射。跨 DLL 释放必须回到分配方，Rust 的 `Vec::from_raw_parts` 或宿主 allocator 不能代替 ABI release。

### 统一 payload 上限

请求/响应按领域使用 `buffer` 中的 `ZrRuntimePayloadLimitV1` 常量。常见上限是：JSON nesting 128、frame 最大边长 16,384、RGBA 最大 256 MiB、world query request/output 1 MiB、world watch request 256 KiB、operation request/result 1 MiB、plugin event page 64 条/256 KiB。先检查字节数，再解析 JSON；被 `LimitExceeded` 拒绝的输入不应重试原文。

## ABI 表与槽位

### `ZrHostApiV1`

```rust
#[repr(C)]
pub struct ZrHostApiV1 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub diagnostics_sink: Option<unsafe extern "C" fn(ZrByteSlice)>,
    pub fetch_resource: Option<ZrRuntimeHostFetchFnV1>,
}
```

`ZrHostApiV1::empty(abi_version)` 生成两个 callback 都为 `None` 的表。null host pointer 表示“没有 host callback”；非 null pointer 必须满足对齐、版本和 `size_bytes == size_of::<ZrHostApiV1>()`。两个 callback 都是可选能力，不能因为 slot 缺失就读取未初始化的函数地址。

### `ZrRuntimeApiV8`

表头永远是 `abi_version: u32`、`size_bytes: usize`，其后字段顺序冻结。当前 required slots 是：

| slot | 函数指针签名 | 必需性 | 生命周期/输出 |
| --- | --- | --- | --- |
| `create_session` | `unsafe extern "C" fn(ZrRuntimeSessionConfigV3, *mut ZrRuntimeSessionHandle) -> ZrStatus` | required | 写入新 session handle |
| `destroy_session` | `unsafe extern "C" fn(ZrRuntimeSessionHandle) -> ZrStatus` | required | 停止 callback、释放 session |
| `release_allocation` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeAllocationId) -> ZrStatus` | required | 释放 owned result |
| `handle_event` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeEventV1) -> ZrStatus` | required | 同步消费借用 payload |
| `capture_frame` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeFrameRequestV1, *mut ZrRuntimeFrameV2) -> ZrStatus` | required | 写入 frame 与 allocation |
| `submit_highlight_set` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeHighlightSetV1) -> ZrStatus` | required | 借用 entity slice 到本次调用结束 |
| `tick_frame` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrRuntimeFrameDemandV1) -> ZrStatus` | required | 写出下一帧 demand |
| `subscribe_plugin_event` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut ZrRuntimePluginEventSubscriptionHandle) -> ZrStatus` | required | 写入 subscription handle |
| `unsubscribe_plugin_event` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimePluginEventSubscriptionHandle) -> ZrStatus` | required | 撤销订阅 |
| `drain_plugin_events` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimePluginEventSubscriptionHandle, *mut ZrOwnedResultV2) -> ZrStatus` | required | 分页 owned JSON |
| `submit_operation` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut ZrRuntimeOperationHandle) -> ZrStatus` | required | 写入 operation handle |
| `poll_operation` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeOperationHandle, *mut ZrRuntimeOperationStatusV2) -> ZrStatus` | required | allocation-free 状态 |
| `harvest_operation` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeOperationHandle, *mut ZrOwnedResultV2) -> ZrStatus` | required | terminal owned output |
| `query_world` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut ZrOwnedResultV2) -> ZrStatus` | required | generation-qualified JSON |
| `watch_world` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrByteSlice, *mut WatchToken) -> ZrStatus` | required | 写入 opaque watch token |
| `unwatch_world` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, WatchToken, *mut u8) -> ZrStatus` | required | 写入是否实际移除 |
| `drain_world_invalidations` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, *mut ZrOwnedResultV2) -> ZrStatus` | required | 一批 invalidation JSON |
| `request_viewport_pick` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportPickRequestV1, *mut ZrRuntimeViewportPickTicket) -> ZrStatus` | required | 写入 pick ticket |
| `poll_viewport_pick` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportPickTicket, *mut ZrRuntimeViewportPickResultV1) -> ZrStatus` | required | fixed-layout completion |
| `cancel_viewport_pick` | `unsafe extern "C" fn(ZrRuntimeSessionHandle, ZrRuntimeViewportPickTicket) -> ZrStatus` | required | 取消 pending ticket |

可选 slots 是 `capture_accessibility_tree`、`bind_viewport_surface`、`unbind_viewport_surface`、`present_viewport`、`profile_control` 和 `drain_host_requests`。V8 不允许通过 `size_bytes` 截断后“猜测”尾部字段；表大小必须精确匹配，未来新增字段应提升表版本。

## `runtime_api::host`：宿主资源与平台请求

源码中的 `host` 是 `runtime_api` 的内部 owner module，未作为 `pub mod host` 开放；公开导入路径是 `zircon_runtime_interface::runtime_api::...`，并且本页列出的 ABI 类型同时由 crate root re-export。不要写成 `zircon_runtime_interface::runtime_api::host::...`。

这一层有两个方向，不能混为同一个 callback。`fetch_resource` 已冻结为可选 ABI 能力，但当前 `zircon_runtime` dynamic session 没有把它当作内置文件系统调用；只有明确接入该 callback 的 host/runtime 组合才会触发资源读取。未提供时应走 runtime 自己的资源 provider 或返回能力缺失，不能假定该 slot 一定被调用。

1. `ZrHostApiV1.fetch_resource` 是 **runtime -> host** 的同步资源读取回调。runtime 传入一个固定布局的资源请求，host 在同一调用中填充可由 `free` 函数指针释放的 `ZrOwnedByteBuffer`。
2. `ZrRuntimeApiV8.drain_host_requests` 是 **host <- runtime** 的异步请求队列。runtime 将 IME、光标、剪贴板、手柄震动和 UI 平台效果排队，host 通过 `ZrOwnedResultV2` 取走一页 JSON；空队列不会分配输出。

### host fetch ABI

`ZrRuntimeHostFetchRequestV1` 是 `#[repr(C)]`，字段顺序和类型固定如下：

```rust
#[repr(C)]
pub struct ZrRuntimeHostFetchRequestV1 {
    pub abi_version: u32,
    pub uri: ZrByteSlice,
    pub flags: u32,
}

pub type ZrRuntimeHostFetchFnV1 =
    unsafe extern "C" fn(ZrRuntimeHostFetchRequestV1, *mut ZrOwnedByteBuffer) -> ZrStatus;
```

构造器是 `ZrRuntimeHostFetchRequestV1::new(abi_version, uri, flags)`。当前唯一定义的 flag 是 `ZR_RUNTIME_FETCH_FLAG_STREAMING_V1`，它是资源提供方与 host 之间的流式意图标记；V1 ABI 本身仍只返回一个完整的 `ZrOwnedByteBuffer`，没有 continuation pointer 或隐式后台读取。若产品需要真正的分块传输，必须在更高层协议另行定义。URI 是调用期借用的 UTF-8 字节，不要把 `String::as_ptr()` 留到 callback 返回之后。

`ZrOwnedByteBuffer` 同样是 `#[repr(C)]`：`data: *mut u8`、`len: usize`、`capacity: usize`、`owner_token: u64`、`free: Option<ZrFreeBytesFn>`。host 返回成功时必须让 `data/len/capacity` 描述同一分配；空结果使用 `ZrOwnedByteBuffer::empty()`。runtime 只在读取 `len` 个字节期间借用数据，消费完成后调用该 buffer 自带的 `free(buffer)`；不能用 runtime allocator、`Vec::from_raw_parts` 或 `owner_token` 猜测释放方式。回调失败应返回 `ZrStatusCode::NotFound`、`InvalidArgument`、`LimitExceeded` 等明确状态，并保持输出为空。

一个 host 实现的调用形状如下（函数指针必须是 `extern "C"`，不能捕获 Rust 闭包）：

```rust
use zircon_runtime_interface::{
    ZrByteSlice, ZrHostApiV1, ZrOwnedByteBuffer, ZrRuntimeHostFetchRequestV1, ZrStatus,
    ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_FETCH_FLAG_STREAMING_V1,
    ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1,
};

unsafe extern "C" fn fetch_asset(
    request: ZrRuntimeHostFetchRequestV1,
    output: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    if output.is_null() {
        return ZrStatus::new(
            ZrStatusCode::InvalidArgument,
            ZrByteSlice::from_static(b"missing fetch output"),
        );
    }
    unsafe { *output = ZrOwnedByteBuffer::empty() };
    if request.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
        return ZrStatus::new(
            ZrStatusCode::UnsupportedVersion,
            ZrByteSlice::from_static(b"unsupported fetch request ABI"),
        );
    }
    let uri = match unsafe {
        request.uri.checked_slice(ZR_RUNTIME_PROJECT_PATH_MAX_ENCODED_BYTES_V1)
    } {
        Ok(uri) => uri,
        Err(_) => {
            return ZrStatus::new(
                ZrStatusCode::InvalidArgument,
                ZrByteSlice::from_static(b"invalid URI bytes"),
            )
        }
    };
    // 将 uri 映射到 host 允许的 res:// 根；禁止把它当作任意 OS 路径。
    let _streaming = request.flags & ZR_RUNTIME_FETCH_FLAG_STREAMING_V1 != 0;
    let _ = (uri, _streaming, ZIRCON_RUNTIME_ABI_VERSION_V1);
    ZrStatus::new(
        ZrStatusCode::NotFound,
        ZrByteSlice::from_static(b"asset provider omitted"),
    )
}

let host = ZrHostApiV1 {
    abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
    size_bytes: core::mem::size_of::<ZrHostApiV1>(),
    diagnostics_sink: None,
    fetch_resource: Some(fetch_asset),
};
```

callback 示例把所有失败分支显式转换为 `ZrStatus::new(...)`；`extern "C"` 函数不能把 Rust `Result` ABI 泄漏给调用方。真实实现应把 `ZrByteSliceError` 映射为 `ZrStatusCode::InvalidArgument` 或 `LimitExceeded`，再用 `validate_runtime_host_api_v1_pointer`/`validate_runtime_host_api_v1_shape` 检查表头，最后把表交给 `get_api`。

### drain host requests

`drain_host_requests` 的真实槽位签名是：

```rust
unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    *mut ZrOwnedResultV2,
) -> ZrStatus;
```

第二个参数必须是非空、预先初始化为 `ZrOwnedResultV2::empty()` 的 out pointer。成功后，`data/len/allocation` 指向 runtime-owned JSON；host 应在当前 session 上用 `release_allocation(session, allocation)` 释放，即使 JSON 解析失败也不能遗忘。runtime 在成功写出 allocation 时就提交本页并从 pending 队列移除，`release_allocation` 只回收字节，不会把请求退回队列；平台动作失败不能依赖再次 drain 自动重放。`InvalidArgument` 表示空 out pointer，`NotFound` 表示未知/已销毁 session；正常空队列返回 `Ok` 且 output 继续保持 empty。单页受 `ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1`（最大条数和编码字节数）约束，超限时 runtime 保留未发出的尾部，下一次 drain 继续取，不应丢弃或重复已确认页面。

页面 JSON 的根类型是 `ZrRuntimeHostRequestBatchV1 { abi_version: u32, requests: Vec<ZrRuntimeHostRequestV1> }`。使用 `new(abi_version, requests)` 或 `empty(abi_version)` 构造；`ZrRuntimeHostRequestV1` 的变体是 `Ime`、`GamepadRumble`、`Cursor`、`Clipboard`、`UiAction`、`UiHost`。这些是 serde DTO，不是 `repr(C)` union；只有外层 `ZrOwnedResultV2` 和 slot 使用 C ABI。host 必须按变体白名单解码，未知 serde variant 或错误 ABI 版本应记录诊断并释放 allocation，不能执行默认平台动作。

#### 请求变体字段

| 变体 | 字段和构造器 | host 行为与约束 |
| --- | --- | --- |
| `Ime` | `ZrRuntimeImeHostRequestV1 { kind, target_viewport: Option<ZrRuntimeViewportHandle>, cursor_area: Option<ZrRuntimeImeCursorAreaV1>, surrounding_text: Option<ZrRuntimeImeSurroundingTextV1> }`；`enable()`、`disable()`、`set_cursor_area(...)`、`set_surrounding_text(...)`、`with_target_viewport(...)` | `kind` 为 Enable/Disable/SetCursorArea/SetSurroundingText。坐标是 window-relative logical pixels；`ZrRuntimeImeTextRangeV1 { start, end }` 是 UTF-8 byte offset。缺少 `target_viewport` 只允许兼容旧序列化输入，当前 producer 总是设置。 |
| `GamepadRumble` | `ZrRuntimeGamepadRumbleRequestV1 { gamepad_id, kind, strong_motor, weak_motor, duration_millis }`；`add(...)` 或 `stop(gamepad_id)` | `kind` 为 Add/Stop；强弱马达应限制在 host backend 支持范围，duration 只在 Add 有意义。无设备时报告能力诊断，不阻塞 session teardown。 |
| `Cursor` | `ZrRuntimeCursorHostRequestV1 { kind, grab_mode, position, value }`；`set_visible(bool)`、`set_grab_mode(...)`、`set_hit_test(bool)`、`set_position(...)` | `kind` 为 SetVisible/SetGrabMode/SetHitTest/SetPosition；grab mode 为 None/Confined/Locked。只读取与 kind 对应的 union-like 字段，避免把 `value` 当作 position。 |
| `Clipboard` | `ZrRuntimeClipboardHostRequestV1 { target_viewport, target_surface, request: UiClipboardRequest }`；`new(viewport, surface, request)` | request 携带 transfer id、intent、expected edit revision、owner 和可选文本。结果用 `ZrRuntimeClipboardResultV1 { target_surface, transfer_id, owner, outcome }` 回送为 clipboard event；文本受 clipboard byte limit 约束。 |
| `UiAction` | `ZrRuntimeUiActionHostRequestV1 { target_viewport, target_surface, input_sequence, action_index, tree_id, target, invocation, secure_value }`；`new(...)` | `secure_value` 只有 opaque reference，Debug/JSON 不包含明文；须在受信任 session 合同内单独解析。用 `input_sequence/action_index/tree_id/target` 做幂等和 stale 检查。 |
| `UiHost` | `ZrRuntimeUiHostRequestV1 { target_viewport, target_surface, input_sequence, request_index, tree_id, effect_index, kind }`；`from_dispatch_request(...) -> Option<Self>` | 通用 effect 为 PointerLock/PointerUnlock/HighPrecisionPointer/Popup/Tooltip/DismissTransientUi/ActivateLink。InputMethod 和 Clipboard 刻意返回 `None`，必须走各自专用队列；`ActivateLink` 的 wire 字段名是 `href`。 |

`ZrRuntimeProjectSceneTransitionRequestV1` 也从 host 请求域导出，但它不是 `ZrRuntimeHostRequestV1` 的 variant，而是项目层 transition DTO：`request_id`、`scene_uri`、`policy`。用 `try_new` 构造时只接受 `res://`、非空、正斜杠、无 `.`/`..`/query/fragment 的项目相对 URI；失败分别是 `MissingResourceScheme`、`EmptyResourcePath` 或 `NonCanonicalResourcePath`。结果含 `request_id`、`scene_uri`、`status`（Succeeded/Failed/Superseded/Rejected）和可选 diagnostic。宿主应把 transition 作为异步 operation 处理，不能在 `handle_event` callback 中同步卸载当前 scene。

host 轮询的最小 Rust 形状如下：

下面的 `apply_*`/`enqueue_*` 是宿主适配器自身的函数名占位，不属于 `zircon_runtime_interface` 导出；示例重点是 slot 调用、owned payload 解码和释放顺序。

```rust
use zircon_runtime_interface::{
    ZrOwnedResultV2, ZrRuntimeApiV8, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1,
    ZrRuntimeSessionHandle,
};

fn drain_once(
    api: &ZrRuntimeApiV8,
    session: ZrRuntimeSessionHandle,
) -> Result<(), &'static str> {
    let drain = api.drain_host_requests.ok_or("host request capability missing")?;
    let release = api.release_allocation.ok_or("allocation release capability missing")?;
    let mut owned = ZrOwnedResultV2::empty();
    let status = unsafe { drain(session, &mut owned) };
    if !status.is_ok() {
        return Err("drain_host_requests failed");
    }
    if !owned.is_empty() {
        let decoded: Result<ZrRuntimeHostRequestBatchV1, &'static str> = match
            usize::try_from(owned.len)
        {
            Err(_) => Err("host request payload is too large"),
            Ok(_) if owned.data.is_null() => {
                Err("non-empty host request payload has a null pointer")
            }
            Ok(len) => {
                let bytes = unsafe { core::slice::from_raw_parts(owned.data, len) };
                serde_json::from_slice::<ZrRuntimeHostRequestBatchV1>(bytes)
                    .map_err(|_| "malformed host request payload")
            }
        };
        let allocation = owned.allocation;
        // Release before propagating a decode error; the runtime owns this allocation.
        let release_status = unsafe { release(session, allocation) };
        if !release_status.is_ok() {
            return Err("host request allocation release failed");
        }
        let batch = decoded?;
        for request in batch.requests {
            match request {
                ZrRuntimeHostRequestV1::Ime(request) => apply_ime(request),
                ZrRuntimeHostRequestV1::Cursor(request) => apply_cursor(request),
                ZrRuntimeHostRequestV1::Clipboard(request) => enqueue_clipboard(request),
                ZrRuntimeHostRequestV1::UiAction(request) => dispatch_ui_action(request),
                ZrRuntimeHostRequestV1::UiHost(request) => apply_ui_effect(request),
                ZrRuntimeHostRequestV1::GamepadRumble(request) => rumble(request),
            }
        }
    }
    Ok(())
}
```

`apply_*` 必须在宿主自己的 UI/GPU 线程调度器中执行；drain callback 本身只负责搬运数据。先把一页 JSON 解码成自有 DTO，再释放 allocation，随后分派平台动作；若平台动作失败，记录 request identity 和平台错误，不要把同一页无限重放。V8 没有独立的 cancel-host-request slot，session destroy 前应停止 admission、drain pending host requests，并由宿主适配器决定未执行请求的丢弃/补偿策略；同时释放所有 owned allocations，之后再卸载动态库。

## `world_sync`：查询、观察与失效

模块路径：`zircon_runtime_interface::world_sync`。这些 DTO 可 serde 为 JSON，也可直接作为 runtime API 的 request/response payload；它们不是 ECS world 的借用引用。

### 请求模型

| 导出 | 真实字段/方法 | 约束 |
| --- | --- | --- |
| `EntityId` | `type EntityId = u64` | `0` 仍可作为协议值，但 runtime 产生的实体通常应非零 |
| `ComponentSelector` | `type_name: String`; `new(type_name)` | 选择一个反射组件类型 |
| `QueryFilter` | `with: Vec<String>`, `without: Vec<String>` | 两个列表均默认空；`deny_unknown_fields` |
| `ComponentWorldQuery` | `filter`, `select`, `generation_hint: Option<u64>` | 组件投影；空 select 表示不取组件值 |
| `WorldHierarchyQuery` | `generation_hint: Option<u64>` | 取层级行 |
| `WorldInspectionFieldsQuery` | `entity`, `generation_hint` | 聚焦 Inspector 字段 |
| `WorldTransformSnapshotQuery` | `entity` | 获取 CAS transform 前的快照 |
| `WorldQuery` | `Components`, `Hierarchy`, `InspectionFields`, `TransformSnapshot` | serde tag 为 `kind`、content 为 `data` |

`WorldQuery` 的便捷构造器是 `hierarchy(hint)`、`inspection_fields(entity, hint)` 和 `transform_snapshot(entity)`；组件查询应直接构造 `ComponentWorldQuery`。`generation_hint()`、`with_generation_hint()` 和 `request_item_count()` 用于缓存/预算决策，不会访问 runtime world。

```rust
use zircon_runtime_interface::world_sync::{
    ComponentSelector, ComponentWorldQuery, QueryFilter, WorldQuery,
};

let query = WorldQuery::Components(ComponentWorldQuery {
    filter: QueryFilter {
        with: vec!["zircon::Transform".into()],
        without: vec!["zircon::Hidden".into()],
    },
    select: vec![ComponentSelector::new("zircon::Transform")],
    generation_hint: Some(last_generation),
});
let request = serde_json::to_vec(&query)?;
// 将 request 作为 ZrByteSlice 传给 api.query_world；不要把 Vec 的指针保存到调用之外。
```

### 结果模型

| 结果 | 字段 | 语义 |
| --- | --- | --- |
| `EntityRow` | `entity`, `components: BTreeMap<String, serde_json::Value>` | 组件投影行；runtime 按 entity 排序 |
| `WorldHierarchyRow` | `entity`, `parent`, `depth`, `display_name`, `kind`, `subtree_hash`, `active_in_hierarchy`, `has_children` | 结构树行 |
| `WorldInspectionFieldRow` | `component_type_path`, `component_display_name`, `field_name`, `field_display_name`, `value_type_path`, `value`, `writable`, `serializable`, `plugin_owned` | Inspector 字段能力 |
| `WorldQueryResult::ComponentRows` | `generation`, `rows` | 新组件投影 |
| `WorldQueryResult::HierarchyRows` | `generation`, `rows` | 新层级投影 |
| `WorldQueryResult::InspectionFields` | `generation`, `entity`, `fields` | 新字段投影 |
| `WorldQueryResult::TransformSnapshot` | `generation`, `world_replacement_epoch`, `entity`, `transform` | CAS 写入依据 |
| `WorldQueryResult::EntityMissing` | `generation`, `entity` | 实体不存在，不能当空字段 |
| `WorldQueryResult::NotModified` | `generation` | hint 与当前 generation 相同 |

`component_result_for_generation` 会稳定排序实体并在 hint 命中时返回 `NotModified`；hierarchy 同理。实现对 `generation == u64::MAX` 特意禁用 short-circuit，因此客户端不得把该值当作可缓存 generation 或依赖它取得 `NotModified`。

### Watch 与 invalidation

`WatchKey` 的四种值是 `Subtree { root }`、`ComponentType { type_name }`、`Asset { resource_id }`、`WorldStructure`；`WatchRegistration { key }` 由 `WatchRegistration::new(key)` 构造。runtime 返回 `WatchToken(u64)`，只能通过 `new/value/is_valid` 观察，token `0` 无效。

```rust
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};
let registration = WatchRegistration::new(WatchKey::WorldStructure);
let bytes = serde_json::to_vec(&registration)?;
// api.watch_world(session, ZrByteSlice { data: bytes.as_ptr(), len: bytes.len() }, &mut token)
// 关闭时调用 api.unwatch_world(session, token, &mut removed); 再 drain_world_invalidations。
```

`InvalidationBatch { generation, dirty, facts }` 的 `dirty` 是 token 列表，`facts` 可为 `Spawned`、`Despawned`、`Reparented`、`SceneLoaded`、`SceneUnloaded`、`WorldReplaced` 或 `AssetReloadApplied`。`has_canonical_dirty_tokens()` 只在 token 严格递增且唯一时返回 true；它不验证事实内容。`AssetReloadFrameApplyReportDto` 的字段是 `applied`、`failed`、`stale` 和 `pending_count`。

### world sync 生命周期

1. session 创建后注册 watch；注册失败不应留下本地 token。
2. 查询时发送当前 generation hint；收到 `NotModified` 只刷新观察时间，不重建树。
3. 每帧或 wake callback 后 drain invalidations，先处理 `WorldReplaced`，再使旧查询/旧 entity selection 失效。
4. `unwatch_world` 是幂等边界；`removed == 0` 表示 token 已被 teardown 或早已撤销。
5. session destroy 前停止新 query，drain 已分配 output，再释放 allocation。

## `project`：项目身份、启动意图与持久引用

模块路径：`zircon_runtime_interface::project`。该模块把“用户请求打开什么”与“预检后允许执行什么”分离，避免把未经验证的路径或 manifest 当成运行时身份。

### 导出分组

| 分组 | 导出 | 作用 |
| --- | --- | --- |
| 激活 ID | `ProjectLaunchInstanceId`, `ProjectActivationOperationSequence`, `ProjectActivationOperationId`, `ProjectActivationOperationIdGenerator` | 并发启动的幂等键；sequence 从 1 开始，generator 的 `allocate()` 返回 `Option` |
| 路径/引用 | `RelPath`, `AssetRef`, `PersistedAssetReference` | project root 下的规范化路径和可迁移 asset identity |
| 身份 | `ProjectGuid`, `ProjectManifestDigest`, `CanonicalDescriptorIdentity`, `ProjectIdentity` | GUID、manifest BLAKE3 摘要和 canonical descriptor |
| 启动 | `ProjectLaunchIntent`, `ProjectLaunchTarget`, `ProjectLaunchProfile`, `ProjectLaunchSource` | Application/Hub/CLI/Welcome/Recent 入口共享的版本化请求 |
| manifest | `ProjectManifestSummary`, `load_project_manifest_value_from_toml_str`, `validate_engine_version_req` 及 `MAX_PROJECT_*` 常量 | 受限 TOML 预检和 v1-v3 迁移 |
| 兼容性 | `ProjectEngineVersion`, `ProjectEngineCompatibility`, `assess_project_engine_compatibility` | 运行 engine 与项目 requirement 的判定 |
| 模板 | `ProjectTemplateId`, `ProjectTemplateDescriptor`, `render_project_template`, `ProjectTemplateReceipt` 等 | 生成项目并留下内容 digest/receipt |
| session lock | `project::session_lock` public module | lock 文件、principal、generation、heartbeat 和生命周期编码 |
| 迁移 | `migrate_retired_*`, `RetiredAssetRefMigrationBudget` | 受预算保护的 retired asset ref 链迁移 |

### 规范化路径和 asset ref

`RelPath::parse` 会先把反斜杠规范化为 `/`，然后拒绝空值、绝对路径、平台前缀和 `.`/`..` 穿越；连续的分隔符会被折叠。`project_assets()` 返回 `assets`。`join_to(root)` 只把已规范化字符串转成 `PathBuf`，不会再次授权 root，因此 root 仍应由调用方校验。`AssetRef::try_new(guid, path_hint, sub)` 将 GUID、可移动 path hint 和可选 subasset 组合；`sub` 必须通过内部 sub-path 校验。读取 identity 时使用 `guid()`、`path_hint()`、`sub()`，不要把 path hint 当主键。

```rust
use zircon_runtime_interface::project::{AssetRef, RelPath};
use zircon_runtime_interface::resource::AssetUuid;

let path = RelPath::parse("characters/hero.model")?;
let reference = AssetRef::try_new(AssetUuid::from_stable_label("hero"), path, None)?;
assert_eq!(reference.sub(), None);
```

`PersistedAssetReference::project(reference)` 只接受 project asset；`try_builtin(locator)` 会检查 scheme 必须为 `builtin`，错误类型是 `PersistedAssetReferenceError`。`builtin(locator)` 在程序员错误时 panic，外部输入必须使用 `try_builtin`。

### 身份、digest 与兼容性

`ProjectGuid::new()` 生成非 nil UUID；`try_from_uuid` 和 `FromStr` 拒绝 nil/非法 UUID。`ProjectManifestDigest::from_bytes` 对**实际被接受的 manifest bytes**做 BLAKE3；`parse` 只接受 64 个小写十六进制字符。`CanonicalDescriptorIdentity::new(path)` 只做 lexical 检查（必须是绝对路径且不能含 `.`/`..`），不执行 filesystem canonicalize；调用方必须先通过平台 filesystem authority 解析。`ProjectIdentity::new(descriptor, guid, digest)` 将三个不可变值绑定，供 admission receipt 使用。

`ProjectEngineVersion::parse` 使用 semver；`assess_project_engine_compatibility(requirement, running)` 返回 `ProjectEngineCompatibility`，调用方通过 `requirement()`、`running_engine()`、`disposition()` 和 `is_compatible()`读取结果。`ProjectEngineCompatibilityDisposition` 的不兼容结果必须阻断 activation，不能仅记录 warning。

### 启动意图

```rust
use std::path::PathBuf;
use uuid::Uuid;
use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationSequence,
    ProjectLaunchInstanceId, ProjectLaunchIntent, ProjectLaunchProfile,
    ProjectLaunchSource,
};

fn make_intent() -> Result<ProjectLaunchIntent, Box<dyn std::error::Error>> {
    let origin = ProjectLaunchInstanceId::new();
    let sequence = ProjectActivationOperationSequence::new(1).expect("non-zero sequence");
    let operation = ProjectActivationOperationId::try_from_parts(origin, sequence, Uuid::new_v4())?;
    let intent = ProjectLaunchIntent::open_existing(
        operation,
        ProjectLaunchSource::Hub,
        ProjectLaunchProfile::Normal,
        PathBuf::from("C:/projects/example"),
    )?;
    assert_eq!(intent.schema_version(), 1);
    Ok(intent)
}
```

`ProjectLaunchTarget::OpenExisting` 携带 `requested_path`；`CreateProject` 携带 `project_name`、`location` 和 `ProjectTemplateId`。构造器会拒绝空/非文本路径和非法项目名。`ProjectLaunchIntent` 是 request，不包含 `ProjectIdentity` 或 activation permit；只有 data-only preflight 成功后才能生成 canonical identity。serde 反序列化会拒绝未知字段和非 `PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1` 版本。

### manifest、模板和迁移

`ProjectManifestSummary` 的字段为 `name`、可选 `engine_version_req`、可选 `template_receipt`、`default_scene`、`format_version` 和可选 `project_guid`。`parse_toml_str`/`parse_toml_bytes` 返回 `Loaded<ProjectManifestSummary>`；bytes 先过 `MAX_PROJECT_MANIFEST_BYTES`（4 MiB）检查。manifest parser 同时限制 roots 4,096、array items 65,536、table entries 16,384、nesting depth 32，并通过 `load_project_manifest_value_from_toml_str` 执行当前 format version 3 的迁移。

模板 API 的稳定顺序是：取得 `ProjectTemplateDescriptor` -> `render_project_template` -> 校验每个 `RenderedProjectTemplateEntry` 的 path/content digest -> 持久化 `ProjectTemplateReceipt`。receipt schema 为 `PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1`，不能把渲染结果当作已激活项目。

retired asset ref 迁移函数都接受明确输入，带 `_with_budget` 的版本使用 `RetiredAssetRefMigrationBudget::new(max_nodes, max_depth, max_references)`；标准预算可由 `standard()`取得。JSON walker 会按 depth、node 和 reference 三项预算拒绝超限，并以 `RetiredAssetRefMigrationError::ResourceLimitExceeded` 报告；resolver 失败则是 `Resolve`，不要无限递归或静默保留部分改写。

### session lock

`project::session_lock` 导出 `PROJECT_SESSION_LOCK_FILE_NAME`、`project_session_lock_path`、Windows mutex 名称函数、`ProjectSessionPrincipalV1`、`ProjectSessionAdmissionLifecycleV1`、`ProjectSessionGenerationV1` 和 `ProjectSessionAdmissionRecordV1`。record 由 `claim(...)` 创建，通过 `transition_to`、`commit_ready` 等方法推进；`encode_project_session_admission_record`/`decode_project_session_admission_record` 是唯一持久化 codec。process id、heartbeat、build-set id、operation id 和 checked epoch 都是诊断/互斥依据，不能从客户端 JSON 直接拼接 lock 文本。

## `serialization`：版本化 envelope、迁移与 canonical 输出

模块路径：`zircon_runtime_interface::serialization`。它统一 text(JSON) 与 binary(bincode) 的 schema/version 头，保证同一 payload 在 runtime、editor、Hub 间可诊断地迁移。

### 公共类型

| 导出 | 字段/方法 | 关键语义 |
| --- | --- | --- |
| `Format` | `Text`, `Binary` | 物理编码选择，不改变 schema |
| `SerializationBudget` | `new(max_output_bytes)`, `max_output_bytes()` | caller-owned 输出上限，无 `Default` |
| `SchemaId` | `new(&'static str)`, `TryFrom<&str/String>`, `as_str()` | 至少两个小写 namespace 段；最大 128 bytes |
| `PayloadHeader` | `schema_id`, `schema_version` | text/binary 共享头，拒绝未知字段 |
| `Loaded<T>` | `value`, `migrated_from: Option<u32>` | 成功加载和迁移来源版本 |
| `VersionedSchema` | `SCHEMA`, `VERSION`, `migrations()` | 类型声明当前 schema 和完整 forward chain |
| `MigrationChain<T>` | `new(&'static [MigrationStep])`, `migrate_value(...)` | 先校验链，再逐版本执行 |
| `MigrationStep` | `new(from_version, migrate)` | 一步只允许 `n -> n+1` |

`SchemaId` 的合法段以小写 ASCII 字母开头，可包含小写字母/数字/连字符但不能以连字符结尾；空段、非 ASCII、缺 namespace 都产生 `SchemaIdError`。schema id 是逻辑身份，不是文件路径。

### 写入与读取

```rust
use serde::{Deserialize, Serialize};
use zircon_runtime_interface::serialization::{
    load_versioned, write_versioned, Format, MigrationChain, VersionedSchema,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Settings { volume: f32 }

impl VersionedSchema for Settings {
    const SCHEMA: zircon_runtime_interface::serialization::SchemaId =
        zircon_runtime_interface::serialization::SchemaId::new("zircon.settings");
    const VERSION: u32 = 1;
    fn migrations() -> &'static MigrationChain<Self> {
        static CHAIN: MigrationChain<Settings> = MigrationChain::new(&[]);
        &CHAIN
    }
}

let bytes = write_versioned(&Settings { volume: 0.8 }, Format::Text)?;
let loaded = load_versioned::<Settings>(&bytes, Format::Text)?;
assert_eq!(loaded.migrated_from, None);
```

Text output是 canonical pretty JSON，并带一个 trailing newline；所有 object key 会排序。`write_versioned_text_to` 直接流式写入 `Write` sink，适合大文件；`write_canonical_text_to` 只写 canonical JSON，不添加 `$zircon` envelope，必须由外层格式另行定义兼容契约。Binary payload 先写固定 wire prefix，再写 `PayloadHeader` 和 value domain；改变字段顺序需要 wire-version bump。

读取顺序是：限制文档大小 -> 检查 text envelope/magic -> schema match -> source version 不得高于 current -> 验证完整 migration chain -> 解码/迁移 -> 构造 `Loaded<T>`。`load_versioned` 对无 envelope 的 text 返回 `MissingTextEnvelope`；只有明确调用 `load_versioned_legacy_schema_zero` 才接受 schema-zero legacy payload。

### 错误与迁移策略

`LoadError` 覆盖 malformed text/binary、invalid envelope、missing envelope、document/payload 超限、magic/wire version、schema mismatch、future version、payload decode 和 `MigrateError`。`WriteError`/`CanonicalTextWriteError` 额外拒绝 NaN/Infinity、RawValue、非有限向量和 sink I/O 失败。错误中保留 schema id/version；上层应按“输入损坏、未来版本、预算超限、暂态 I/O”分类处理。

迁移链必须从 0 连续到 `VERSION - 1`，不允许 duplicate/out-of-order/missing/unexpected step。迁移函数只接收 `serde_json::Value`，不得绕过 `MigrationChain::migrate_value` 直接调用 function pointer；失败会包装为 `StepFailed` 并保留 from_version。

## `reflect`：稳定类型/字段身份与读写

模块路径：`zircon_runtime_interface::reflect`。反射层服务 Inspector、远程 world query、脚本和序列化；字段名称可改名，但 field ID、type path 和 schema fingerprint 负责稳定 identity。

### 统一值模型

`ReflectedValue` 的 variant 是 `Null`、`Bool(bool)`、`Integer(i64)`、`Unsigned(u64)`、`Scalar(f32)`、`String(String)`、`Enum(String)`、`Vec2([f32;2])`、`Vec3([f32;3])`、`Vec4([f32;4])`、`Quaternion([f32;4])`、`Entity(Option<u64>)`、`Resource(String)`、`List(Vec<ReflectedValue>)`、`Map(BTreeMap<String, ReflectedValue>)` 和 `Json(serde_json::Value)`。`type_name()` 返回稳定诊断名称。`ZrReflectValue` 的内置实现覆盖 bool、i8-i64、u8-u64、f32、String、Option<u64>、Vec2/3/4 和 `Vec<T>`；f32/向量必须 finite，整数窄化超范围返回 `ReflectError::TypeMismatch`。

### 类型和字段 schema

| 类型 | 真实字段 | 构造/读取 |
| --- | --- | --- |
| `ReflectTypePath` | `type_path`, `short_type_path`, optional `module_path`, optional `plugin_id` | `new(full, short)`, `with_module_path`, `with_plugin_id`; getter 均为借用 |
| `ReflectFieldId` | transparent UUID | `from_stable_keys(owner_key, field_key)`, `try_from_uuid`, `as_uuid` |
| `ReflectFieldInfo` | `id`, `name`, `display_name`, `aliases`, `value_type_path`, `editable`, `serializable`, `editor_visible`, optional default/range, enum options, `editor_hint`, optional documentation | `new`/`from_stable_keys` 后用 builder 方法补充元数据 |
| `ReflectTypeInfo` | `kind`, `fields` | `struct_with_fields`, `json_with_fields`, `opaque` |
| `ReflectTypeRegistration` | `type_path`, `display_name`, optional docs, `type_info`, `serialization`, `role`, `serializable`, `editor_visible`, `remote_visible`, `script_visibility` | `new`, `as_component`, `as_resource`, `with_*` |
| `ReflectSchemaFilter` | optional `type_path`, include components/resources, editor/remote/plugin flags | `editor_visible`, `remote_visible`, `for_type` |
| `ReflectSchemaRequest` | `filter` | 同名便捷构造器 |
| `ReflectSchemaResponse` | catalog algorithm version, fingerprint, registrations | `new(fingerprint, registrations)` |

`ReflectTypeKind` 为 Struct/TupleStruct/Tuple/Enum/List/Map/Scalar/Opaque/Json；`ReflectSerializationStrategy` 为 None/Value/Json/ResourceHandle/EntityReference；`ReflectTypeRole` 为 Value/Component/Resource。`with_plugin_id` 会验证 plugin id 与 type path 组合，失败返回 `ReflectError`。

### 读写请求

`ReflectObjectAddress` 只有 `Component { entity, type_path }` 与 `Resource { type_path }`，构造器会验证 type path。`ReflectReadRequest { address, field_id }`、`ReflectWriteRequest { address, field_id, value }`、`ReflectFieldsRequest { address }` 都是 serde DTO；响应分别携带 field 或 fields，写响应还包含 `changed`。`field_name` 仅用于当前 schema/diagnostic，identity 永远使用 `ReflectFieldId`。

```rust
use zircon_runtime_interface::reflect::{
    ReflectFieldId, ReflectObjectAddress, ReflectReadRequest, ReflectWriteRequest, ReflectedValue,
};

let address = ReflectObjectAddress::component(42, "zircon::Transform")?;
let field_id = ReflectFieldId::from_stable_keys("zircon::Transform", "translation");
let read = ReflectReadRequest::new(address.clone(), field_id);
let write = ReflectWriteRequest::new(
    address,
    field_id,
    ReflectedValue::Vec3([1.0, 2.0, 3.0]),
);
```

`ZrReflect` derive/implementation同时提供按 name 与按 slot 的读写；按 slot 适合 runtime hot path，按 name 适合 authoring/diagnostics。写入前必须检查 registration 的 `editable`/`serializable`/`remote_visible` 和 value budget；不应把 `Json` variant 当成绕过 schema 的万能通道。

### schema catalog 与 fingerprint

`ReflectSchemaCatalog` 维护 registration、依赖边、short path ambiguity 和 field-id ownership；`try_new`、`try_insert`、`try_replace`、`try_remove`、`clear` 会使 dependency order/fingerprint 缓存失效。删除仍被其他 registration 依赖的 type 会返回 `ReflectError::InvalidRegistration`。`ReflectSchemaCatalogSnapshot` 是只读发布物，`ReflectSchemaFingerprint` 使用 BLAKE3，算法常量为 `REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION`。fingerprint 输入包含 type path、字段 id/name/alias、类型、默认值、范围、enum options、hint、serialization、role 和 visibility；仅改 display name 也会改变 fingerprint，故升级时必须重新协商。

## `resource`：locator、identity、typed handle 与记录

模块路径：`zircon_runtime_interface::resource`。resource ID 与 locator 分离：locator 是可读寻址，ID 是事件/依赖图 identity，typed handle 只在 Rust 类型层附加 marker。

### locator 和 ID

`ResourceScheme` 支持 `Res` (`res://`)、`Library` (`lib://`)、`Package` (`package://`)、`Builtin` (`builtin://`) 和 `Memory` (`mem://`)。`ResourceLocator::parse`/`new` 会规范化分隔符和 `.`，拒绝空路径、root escape、空 label；package locator 必须有 `package_id/path`。公开 getter 是 `scheme()`、`path()`、`label()`、`package_id()`、`package_path()`，`matches_display` 不分配字符串。

```rust
use zircon_runtime_interface::resource::{ResourceLocator, ResourceScheme, ResourceId};

let locator = ResourceLocator::parse("package://com.zircon.ui/layout/main.zui#root")?;
assert_eq!(locator.scheme(), ResourceScheme::Package);
assert_eq!(locator.package_id(), Some("com.zircon.ui"));
let id = ResourceId::from_locator(&locator);
```

`ResourceId::from_locator` 对 memory scheme 生成随机 ID，对其他 scheme 使用稳定 label；`from_stable_label` 和 `from_asset_uuid` 也生成稳定 UUID。`AssetUuid` 可由 `new` 或 `from_stable_label`取得，并实现 `Display`/`FromStr`。

### typed/untyped handle

`ResourceHandle<TMarker>` 是 `Copy` 的 `(ResourceId, PhantomData<TMarker>)`，通过 `new(id)` 和 `id()`访问；`UntypedResourceHandle` 是 `(id, ResourceKind)`，可通过 `typed::<TMarker>()` 在 kind 相等时恢复 typed handle。`ResourceMarker` 要求 `Send + Sync + 'static` 并提供关联常量 `KIND`。当前 marker 覆盖 Data、Model、Mesh、Material/MaterialGraph、Texture、Shader、Scene、Sound、Font、PhysicsMaterial、NavMesh、NavigationSettings、Terrain/TerrainLayerStack、TileSet/TileMap、Prefab、Animation* 和 UiLayout/UiWidget/UiStyle。

### record、诊断和事件

`ResourceRecord` 字段为 `id`、`kind`、`primary_locator`、可选 `artifact_locator`、`revision`、`state`、`dependency_ids`、`diagnostics`、`source_hash`、`importer_id`、`importer_version` 和 `config_hash`。`new` 默认 `Pending`/revision 0；`with_*` builder 只改变 record，不执行导入。`failure_reason()` 在 state 为 `Error` 时优先返回 error severity diagnostic，否则返回第一条诊断。

`ResourceState` 为 `Pending`、`Ready`、`Error`、`Reloading`；`ResourceEvent` 携带 `kind` (`Added`/`Updated`/`Removed`/`Renamed`/`ReloadFailed`)、resource kind、id、current/previous locator 和 revision。资源热 reload 应按 revision 丢弃过期事件；`ReloadFailed` 不应隐式删除 last-good artifact。

## `ui`：公开模块地图与关键数据流

`zircon_runtime_interface::ui` 是一个 public module facade，根 crate 不把所有 UI 类型平铺 re-export。下表按 `ui/*/mod.rs` 的实际 `pub use` 分组；同组内的类型共享相同生命周期与序列化规则。

| 子模块 | 公开导出范围（代表性/完整族） | 责任 |
| --- | --- | --- |
| `accessibility` | `UiA11yRole`, `UiA11yCheckedState`, `UiA11yTextSelection`, `UiA11yState`, `UiAccessibilityAction*`, `UiAccessibilityNode`, `UiAccessibilityTreeSnapshot`, `UiAccessibilityDiagnostic*`, `UiAccessibilityContract` | 无障碍树、动作与诊断 |
| `binding` | `UiBinding*`, `UiEventBinding`, `UiEventKind`, `UiEventPath`, `UiModel*`, `UI_BINDING_*`/`UI_MODEL_*` limits | 模型解析、binding conversion、provider/schema |
| `component` | `UiComponent*`, descriptor/prop/slot/host/render capability、drag/event、`UiValue*` | widget/component descriptor 与事件 |
| `design_tokens` | token/cascade/chrome/density/state-role 类型及常量 | 设计令牌与主题级联 |
| `dispatch` | input/navigation/pointer 的 `Ui*InputEvent`、request/reply/effect/context/result | 输入路由与副作用 |
| `ecs` | projection/diff/compute 类型 | UI ECS 投影和 dirty 计算 |
| `event_ui` | `UiNodeId`, `UiTreeId`, `UiNodePath`, `Ui*Descriptor`, `UiReflection*`, invocation/control/route/subscription | Inspector 与动作反射 |
| `focus` | focus path/state/traversal 类型 | 键盘/指针焦点 |
| `layout` | constraints、geometry、slot、style、scroll、virtualization、engine selection | 测量/布局/像素对齐 |
| `navigation` | navigation graph/route 类型 | UI 导航语义 |
| `picking` | hit/pointer picking 类型 | 命中测试 |
| `pipeline` | `UiPipelineStage`, `UiPipelineStageReport`, `UiPipelineFrameReport`, counters | 十阶段 UI 帧流水线 |
| `skin`/`style` | preset、semantic token、resolved style | 皮肤与样式解析 |
| `surface` | arranged tree、hit grid、persistent sequence、render extract/batches/text、debug timeline | 不可变 frame 发布与绘制提取 |
| `template` | asset/document/prototype/compiled binding/style/resource 类型与 schema limits | `.zui` authoring/compile contract |
| `tree` | `UiTree`, `UiTreeNode(s)`, `UiVisibility`, `UiDirtyFlags`, input/pointer policy | 运行时树状态 |
| `v2` | `UiV2AssetDocument`, `UiV2CompiledDocument`, arena/graph/repeat/style | 新版文档编译路径 |
| `widget` | widget identity/state/value 类型 | 控件行为 |
| `window` | `UiWindowEvent*`, metrics, pump, runtime adapter | 平台窗口到 UI 输入的适配 |

### UI frame 发布契约

`UiSurfaceFrame` 是一次原子发布的 immutable snapshot：`generation`、`domain_generations`、`tree_id`、`window_state`、`arranged_tree: Arc<UiArrangedTree>`、`render_extract`、`hit_grid`、`focus_state`、`focus_path`、`last_rebuild`、`layout_engine_report` 和 `pipeline_report`。domain generations 分别是 layout/render/hit_test/focus/pipeline/window；只更新一个域时不要伪造整个 frame generation。

`UiArrangedTree` 含 `tree_id`、roots、nodes、draw_order 和 canvas_layers；`get(node_id)`/`children_of(node_id)`只返回 snapshot 内引用。`UiArrangedNode` 的字段包括 node/path/parent/children、frame/clip_frame、z/paint order、visibility、input/pointer policy、enabled/clickable/hoverable/focusable、clip_to_bounds、control_id 和 slot summary。`supports_pointer()` 同时检查 enabled、visibility、pointer policy 和交互能力。

### event_ui reflection

`UiNodeId` 与 `UiTreeId`/`UiNodePath` 是 serde-safe identity：`UiNodeId(pub u64)` 是单字段 newtype（源码未声明 `repr(C)`/`repr(transparent)`），后二者包裹 `String`，因此它们属于 serde DTO 而不是跨 DLL 的固定布局 ABI。`UiNodeDescriptor` 字段为 node identity、class/display name、children、state flags、visibility、properties 和 actions；builder 是 `new`、`with_child`、`with_state_flags`、`with_visibility`、`with_property`、`with_action`。`UiPropertyDescriptor` 记录 name/type/readable/writable/reflected_value；`UiActionDescriptor` 记录 action id、event kind、binding symbol、parameter schema、remote callable 和 route id。

`UiReflectionSnapshot::new(tree_id, roots, nodes)` 将 node vector 建成 BTreeMap；`node(id)` 返回借用 descriptor。更丰富的 `UiReflectorSnapshot` 还包含 lifecycle、effective visibility、dirty、property source/invalidation、focus/capture/hover 和 source asset/template path。远程 Inspector 必须读取 `readable`/`writable`，并把 `validation_message` 与 `UiPropertyInvalidationReason` 一起显示。

### binding、dispatch 与窗口

binding 层把 `UiBindingSource`、`UiBindingTarget`、`UiBindingValue`、conversion provider generation 和 execution/mutation receipt 串成一次可追踪更新。集合值有 max depth/nodes/string bytes/entries 限制；解析错误与 provider generation mismatch 都是可诊断终止，不应静默使用旧值。

dispatch 输入按 `UiInputSequence`、`UiInputTimestamp`、`UiInputModifiers` 和 `UiInputRoutePolicy` 归属；结果由 `UiInputDispatchResult`/`UiDispatchReply`/`UiDispatchEffect` 表达。IME 使用 `UiImeInputEvent`、preedit clauses、delete-surrounding 和 surrounding text；clipboard/drag/drop/pointer capture 都有独立 transfer/session identity。

`UiWindowEventKind` 的关键值为 Created、CloseRequested、Closed、Destroyed、CursorMoved、CursorEntered/Left、Focused、ApplicationActivation、Occluded、Resized、ScaleFactorChanged、BackendScaleFactorChanged、Moved、WindowAction 和 RequestRedraw。`UiWindowEvent::impact()` 将事件映射到 layout/input/redraw/close dirty domain；失焦、应用停用和 non-client action 可通过 `transient_dismissal_effect()` 生成 popup dismiss effect。

### layout、pipeline 与 render extract

`AxisConstraint { min, max, preferred, priority, weight, stretch_mode }` 的 `max < 0` 表示无上限，`resolved()` 会 clamp preferred 并把非正 weight 归一化为 1。`BoxConstraints` 包含 width/height；`LayoutBoundary` 的 ContentDriven 才向父级传播 child layout invalidation。`UiFrame`/`UiGeometry`/`UiLayoutTransform` 使用 f32 几何，像素 snapping 需结合 `UiLayoutMetrics` 的 DPI。

runtime schedule 固定十个 `UiPipelineStage`：InputCollect -> Focus -> WidgetBehavior -> TextMeasure -> Layout -> PostLayout -> Picking -> A11yExtract -> RenderExtract -> BatchPrepare。`UiPipelineFrameReport::from_stage_reports` 会重算 totals；`is_complete_ordered()` 检查完整顺序，`missing_required_stages()` 找缺失 stage，`repeated_pointer_move_fast_path_holds(expected)` 用于确认 pointer move 没有触发全布局。六个旧 diagnostic stage 仅为存档反序列化，不属于 runtime schedule。

surface render 导出包括 `UiRenderFrameExtract`、`UiRenderCommand`、`UiBatchPlan`、brush/payload、text shaping、cache/debug/visualizer 类型。render extract 只借用/引用本 frame generation 的资源 key；generation 改变时由 `UiRenderCacheInvalidationReason` 驱动重建，不能跨 frame 保存裸指针。

### template 与 v2

template 模块同时公开 authoring document、compiled artifact、binding、style、prototype、resource dependency 和 schema diagnostic。关键版本常量是 `UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION`、`UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION`、`UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION`、`UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION`。migration outcome 必须保留 `UiAssetMigrationReport`；编译失败时不得发布旧 document 的半成品 surface。

v2 使用 `UiV2AssetDocument` -> `UiV2CompiledDocument` -> arena/graph/surface 的显式阶段；`UiV2NodeArena` 和 `UiV2NodeHandle` 只在对应 compiled document 生命周期内有效。repeat 字段名常量（`UI_V2_REPEAT_*`）是序列化契约，改名需要 schema 升级和迁移。

## `runtime_api::frame`：帧需求、高亮和 viewport picking

本标题按源码 owner module 命名；`runtime_api::frame` 在 crate 中是私有 `mod frame`，不是可直接访问的 public module。外部调用应从 `zircon_runtime_interface::runtime_api::{ZrRuntimeFrameDemandV1, ...}` 或 crate root re-export 导入。

### frame demand

`ZrRuntimeFrameDemandV1` 是 `#[repr(C)] { abi_version: u32, kind: u32, delay_nanoseconds: u64 }`。kind 常量为 idle=0、immediate=1、after=2；构造器是 `idle()`、`immediate()`、`after(delay_nanoseconds)`。`is_valid()` 要求 ABI 版本正确，idle/immediate 的 delay 必须为 0，未知 kind 失败。宿主可把 `After` 映射到定时器，但不能把它当作硬实时保证。

### highlight

`ZrRuntimeEntityIdSliceV1 { data: *const u64, len: usize }` 是借用 slice；`empty`、`from_slice` 和 unsafe `as_slice` 会检查 null、对齐、长度和 address-space。`ZrRuntimeHighlightRenderAttributesV1 { outline_enabled: u32, tint_rgba: [f32;4] }` 只接受 0/1 和 finite tint。`ZrRuntimeHighlightSetV1 { abi_version, viewport, generation, entities, attributes }` 由 `new(viewport, generation, entities, attributes)`构造，unsafe `validate()` 同时验证 header、viewport、attributes 和 entity slice。实体数组必须活到 submit slot 返回。

### viewport pick

`ZrRuntimeViewportPickRequestV1` 字段为 abi/purpose/viewport/viewport_size/pixel/frame_generation/input_sequence/policy_flags/reserved；`new` 使用 `ZrRuntimeViewportPickPurposeV1::{Hover,Press,Selection}`。请求验证要求 pixel 在边界内、frame/input generation 非零、policy 只含 include-translucent/backfaces 两个 flag、reserved=0。

`ZrRuntimeViewportPickResultV1` 重复请求 identity，并追加 world_generation、entity、instance、subobject、depth、world_position、world_normal、applied_policy_flags。disposition 为 Pending/NoHit/Hit/StaleFrame/Unavailable/Rejected/Cancelled；只有 Hit 可以携带非零 entity/depth/world geometry，其他 terminal result 必须把 target 字段清零。`matches_request` 防止跨 viewport/frame/input 混用 completion。

```rust
use zircon_runtime_interface::{
    ZrRuntimeViewportHandle, ZrRuntimeViewportPickPurposeV1,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPixelV1,
    ZrRuntimeViewportSizeV1,
};
let request = ZrRuntimeViewportPickRequestV1::new(
    ZrRuntimeViewportHandle::new(1),
    ZrRuntimeViewportSizeV1::new(1280, 720),
    ZrRuntimeViewportPixelV1::new(640, 360),
    presented_generation,
    input_sequence,
    ZrRuntimeViewportPickPurposeV1::Selection,
    0,
);
assert!(request.validate_viewport_pick());
```

## `runtime_api::session`：session、事件、operation 和 plugin event

本标题按源码 owner module 命名；`runtime_api::session` 在 crate 中是私有 `mod session`。公开路径是 `zircon_runtime_interface::runtime_api::...`（或 crate root re-export），而不是 `zircon_runtime_interface::runtime_api::session::...`。

### session config 与 identity

`ZrRuntimeSessionConfigV3` 是 `#[repr(C)] { abi_version, profile, project_root, play_scene, play_report_pipe, wake_sink }`。所有 string 都是调用期间借用的 `ZrByteSlice`；project_root 是 physical root anchor，play_scene 是 project-relative DynamicScene，play_report_pipe 是可选逻辑 outlet。`empty()` 使用 V3 header 和 disabled wake sink；`is_valid()` 只做版本与 wake sink 形状检查，路径/UTF-8/feature 由 runtime admission 处理。

`ZrRuntimeWakeSinkV1 { abi_version, token, wake }` 要么 `(token=0, wake=None)`，要么 token 非零且 callback Some；callback 必须快速返回，不能在 callback 内同步 destroy 同一 session。`GatewaySessionIdentity` 把 runtime_instance/session、gateway_generation、transport_epoch、project 和 play_instance 绑定；`detached()` 的 session handle 无效。不要只传裸 `ZrRuntimeSessionHandle` 给 editor gateway。

### frame、viewport、camera

`ZrRuntimeFrameRequestV1 { abi_version, viewport, size }` 和 `ZrRuntimeAccessibilityTreeRequestV1 { abi_version, viewport, size, generation_hint }` 都通过 `new` 构造。`ZrRuntimeFrameV2 { abi_version, width, height, generation, rgba: ZrOwnedResultV2 }` 是非 Clone 的 out payload；`empty(abi_version)` 生成清空状态，`is_empty()` 只判断 out-parameter 是否清空。读取 RGBA 前必须调用 `validate_runtime_frame_rgba_shape(width,height,rgba.len)`，期望长度是 `width * height * 4`，并校验最大边长/最大字节数。

`ZrRuntimeViewportSizeV1 { width, height }` 与 `ZrRuntimeViewportMetricsV1 { logical_size, device_scale_factor, physical_size }` 是值 DTO。`ZrRuntimeNativeSurfaceTargetV1` 支持 `none(abi)` 和 `win32(abi, hwnd, hinstance)`；`ZrRuntimeBindViewportSurfaceRequestV1` 组合 abi/viewport/size/target。`ZrRuntimeViewportCameraV1` 字段为 abi、Transform、projection_kind、fov_y_radians、ortho_size、z_near、z_far；projection kind 只能使用 perspective/orthographic 常量。

### 事件

`ZrRuntimeEventV1` 固定字段为 `abi_version`、`kind`、`viewport`、`size`、`metrics`、`x`、`y`、`delta`、`button`、`state`、`pointer_id`、`key_code`、`scan_code` 和 `payload`。构造器覆盖 viewport resize/metrics、pointer/mouse/wheel、camera、editor transform、cursor/file drag、window status、lifecycle、touch、keyboard、IME、clipboard、accessibility 和 gamepad。payload 只在 `handle_event` 同步调用中借用；事件 kind/state/button 必须用 constants 模块的数值，不要写魔数。

常见调用形状：`ZrRuntimeEventV1::pointer_moved(abi, viewport, x, y)`；`keyboard(abi, viewport, action, key_code, scan_code, key_text)`；`ime_preedit(abi, viewport, value, cursor_start, cursor_end)`；`file_dropped(abi, viewport, path)`；`editor_transform_write(abi, viewport, &write)`。UTF-8 文本长度受 event/clipboard limit 约束，坏 UTF-8 应在 host adapter 处报告而不是 panic。

### asynchronous operation

`ZrRuntimeOperationHandle` 是 transparent u64，`new/invalid/raw/is_valid` 与其他 handle 一致。提交 DTO `ZrRuntimeOperationSubmitRequestV1 { abi_version, operation_id, payload }` 通过 `new` 构造，调用 `serde_json::to_vec` 后送入 submit slot。状态 `ZrRuntimeOperationStatusV2` 为 `#[repr(C)] { abi_version, phase, detail_kind, reserved, handle, completed_work, total_work, detail_value }`；phase 为 Queued/Preparing/ReadyToApply/Completed/Failed/Cancelled/Expired/Harvested，detail kind 描述 queue/admission/deadline/panic/TTL 等原因。

结果 `ZrRuntimeOperationResultV1 { abi_version, handle, operation_id, outcome }` 的 outcome 是 `Succeeded { output }` 或 `Failed { error }`；`succeeded_output()`/`failure()` 提供只读访问。submit 只代表 admission，poll 直到 terminal，再 harvest owned result；shutdown 前 cancel/drain pending handles。terminal TTL、already harvested 和 stale session 都应映射到可恢复/不可恢复诊断。

### plugin event mirror

订阅 request `ZrRuntimePluginEventSubscribeRequestV1 { abi_version, event_id, payload_schema }`；handle 是 transparent u64。delivery 记录 play_session_id、subscription、event_id、payload_schema、sequence 和 `Box<RawValue>` payload，避免先构造 JSON Value tree；batch 记录 abi、deliveries、remaining_deliveries、oldest_pending_age_millis。单页最多 64 条/256 KiB；消费顺序按 sequence，发现 backlog 时继续 drain，不能把 remaining=0 当作 unsubscribe。

## 错误、线程与生命周期矩阵

| 场景 | 典型错误/状态 | 调用方动作 |
| --- | --- | --- |
| ABI 版本/大小不符 | `UnsupportedVersion`、shape error | 拒绝加载，报告 provider/build-set，不调用 slot |
| 输入指针/长度错误 | `InvalidArgument`、`ZrByteSliceError` | 丢弃请求；不要读取 pointer |
| 输出超限 | `LimitExceeded`、serialization output error | 记录大小和 schema，缩小请求或分片 |
| 可选能力缺失 | `BridgeNotEnabled`、slot `None` | 退化到 headless/无 surface 路径 |
| stale generation/handle | `NotFound`、pick `StaleFrame`、world `NotModified` | 刷新 identity/generation，不重放旧写入 |
| runtime panic/worker 丢失 | `Panic`、operation detail `WorkerPanic`/`WorkerChannelLost` | 隔离 session，保留诊断，等待 teardown |
| serialization/schema | `SchemaMismatch`、`FutureVersion`、`MigrateError` | 迁移或升级 producer；不能猜测字段 |

ABI callbacks 的线程归属由具体宿主合同决定；`ZrRuntimeWakeSinkV1` 只负责唤醒，不能假设回调线程可操作 UI/GPU。跨线程传递 DTO 时只传 `Copy` 值或自有 bytes；借用 slice、RawValue、owned output 和 `Arc` snapshot 必须遵守各自生命周期。

推荐的 session teardown 顺序：停止 admission -> 取消/harvest operations -> unsubscribe plugin/world watches -> drain host/plugin/world outputs -> unbind viewport surface -> destroy session -> 卸载动态库。任一步失败都应进入 teardown failure ledger，而不是继续释放未知 owner 的 allocation。

## 源码与测试索引

以下链接指向当前仓库 `main` 分支的 owner 文件；它们用于在签名变更后快速回读，不是独立的版本承诺。

| 领域 | owner 源码 | 契约测试 |
| --- | --- | --- |
| crate root / ABI facade | [src/lib.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/lib.rs)、[runtime_api/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/mod.rs) | [abi_safety_contracts.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/abi_safety_contracts.rs) |
| function table | [runtime_api/abi/api_table.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/abi/api_table.rs) | [runtime API shape tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/abi/api_shape_tests.rs)、[runtime dynamic table tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/tests/api_table.rs) |
| host callbacks / request queue | [runtime_api/abi/host_api_shape.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/abi/host_api_shape.rs)、[runtime_api/host/host_requests.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/host/host_requests.rs)、[runtime_api/session/requests.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/session/requests.rs) | [host_requests.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/dynamic_api/tests/host_requests.rs)、[contracts.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/contracts.rs) |
| world sync | [world_sync/query.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/world_sync/query.rs)、[watch.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/world_sync/watch.rs)、[invalidation.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/world_sync/invalidation.rs) | [world_sync_contracts.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/world_sync_contracts.rs) |
| project identity | [project/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/project/mod.rs)、[project_launch_intent.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/project/project_launch_intent.rs) | [project_launch_intent.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/project/tests/project_launch_intent.rs)、[asset_ref.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/project/tests/asset_ref.rs) |
| serialization | [serialization/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/serialization/mod.rs)、[load.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/serialization/load.rs)、[write.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/serialization/write.rs) | [load_contract.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/serialization/tests/load_contract.rs)、[migration_contract.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/serialization/tests/migration_contract.rs)、[write_contract.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/serialization/tests/write_contract.rs) |
| reflection | [reflect/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/reflect/mod.rs)、[reflected_value.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/reflect/reflected_value.rs)、[schema_catalog/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/reflect/schema_catalog/mod.rs) | [reflect_contracts.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/reflect_contracts.rs)、[schema catalog tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/reflect/schema_catalog/tests.rs) |
| resources | [resource/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/resource/mod.rs)、[locator.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/resource/locator.rs)、[resource_record.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/resource/resource_record.rs) | [resource_contracts.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/resource_contracts.rs) |
| frame / picking | [frame/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/frame/mod.rs)、[viewport_pick.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/frame/viewport_pick.rs)、[frame_shape.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/frame/frame_shape.rs) | [surface_frame_contracts.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/surface_frame_contracts.rs) |
| session / events | [session/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/session/mod.rs)、[events.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/session/events.rs)、[operation.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/runtime_api/session/operation.rs) | [runtime_operation.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/runtime_operation.rs)、[runtime_owned_result.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/runtime_owned_result.rs) |
| UI contracts | [ui/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/ui/mod.rs)、[surface/frame.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/ui/surface/frame.rs)、[window/event.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/ui/window/event.rs) | [ui_contract_spine.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/ui_contract_spine.rs)、[IME contracts](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime_interface/src/tests/window_input_contracts/ime.rs) |

## 机制案例

### 案例 A：远程 Inspector 增量刷新

1. host 用 `WorldQuery::Hierarchy(Some(last_generation))` 查询树；收到 `HierarchyRows` 后保存 generation。
2. 注册 `WatchKey::WorldStructure` 与关注组件类型 watch。
3. 每次 `drain_world_invalidations`，先检查 `WorldReplaced`/SceneLoaded，再按 dirty token 重查受影响 projection。
4. focused entity 使用 `WorldInspectionFieldsQuery`；本 crate 只定义 `ReflectWriteRequest` DTO，并未在 `ZrRuntimeApiV8` 中提供通用 reflect-write slot。编辑器必须通过其拥有的 gateway/operation 合同提交写入，并携带当前 world replacement epoch，避免把旧 UI selection 写进新 world。

### 案例 B：编辑器拖拽 transform

`ZrRuntimeEditorTransformWriteV1` 由 `new(entity, interaction_id, sequence, world_replacement_epoch, phase, expected, target)` 创建。Begin/Apply 的 sequence 必须为 1，Preview/Commit/Cancel 必须大于 1；expected/target 的 translation/rotation/scale 必须 finite、rotation 非零、scale 分量非零。把 request 借为 `ZrByteSlice` 后发 `handle_event`；runtime 以 expected 做 compare-and-set，失败时返回 `ZrRuntimeEditorTransformError`/诊断，编辑器应刷新 snapshot 而不是强制覆盖。

### 案例 C：帧捕获与 hit-test 配对

先用 `capture_frame` 得到 generation 和 RGBA；将鼠标物理像素、同一 generation、input sequence 组装为 `ZrRuntimeViewportPickRequestV1`。poll 结果必须 `matches_request(request)`；`StaleFrame` 只触发重新 capture，不得把新 frame 的 entity 当作旧点击结果。

### 案例 D：UI 模板热加载

加载 TOML 后保留 `UiAssetMigrationOutcome.report`，用 `UiDocumentCompiler` 生成 compiled document，再构建 template surface。将 source schema/compiler schema/package schema 写入缓存 key；file watcher 事件只产生新 generation，旧 `UiSurfaceFrame` 继续供当前 render 使用，直到新 frame 完整通过 layout/picking/render extract/pipeline stages。

## 最佳实践与测试映射

- 新增 `repr(C)` 字段必须提升 ABI family/version，并同步更新 `runtime_build_set/interface_spec_v1.json`、slot catalog 和 shape tests；不要在 V8 表尾偷偷追加字段。
- 新增 serde 字段时默认使用 `deny_unknown_fields` 的 DTO 先做兼容评估；版本化 payload 必须补 migration step 和 malformed/future-version 测试。
- 所有 opaque handle 都应在日志中同时输出 session/gateway/transport generation；单独打印整数会掩盖跨 session 复用。
- 任何 borrowed pointer 都在函数签名旁写明调用窗口；所有 owned output 都在同一 owner 中释放，即使解析失败。
- world/UI 查询优先使用 generation hint 和 immutable snapshot；不要在渲染线程持有 ECS/world 可变借用。
- UI、resource、plugin event 的限制常量是安全边界，不是性能建议；达到上限时做分页/分层查询。

对应验证套件：`src/tests/abi_safety_contracts.rs`、`src/tests/contracts.rs`、`src/tests/world_sync_contracts.rs`、`src/serialization/tests/*`、`src/tests/reflect_contracts.rs`、`src/reflect/schema_catalog/tests.rs`、`src/tests/resource_contracts.rs`、`src/tests/surface_frame_contracts.rs`、`src/tests/window_input_contracts/*`、`src/tests/ui_contract_spine.rs`、`src/tests/runtime_owned_result.rs`、`src/tests/runtime_operation.rs`，以及 runtime consumer 的 `zircon_runtime/src/dynamic_api/tests/host_requests.rs`。建议在 Windows 工作区执行：

```powershell
cargo test -p zircon_runtime_interface --all-features
cargo test -p zircon_runtime_interface world_sync --all-features
cargo test -p zircon_runtime_interface serialization --all-features
cargo test -p zircon_runtime_interface reflect --all-features
cargo test -p zircon_runtime_interface abi --all-features
```

## 覆盖边界与缺口

- `plugin_api`、`runtime_api`、`buffer` 已按 crate-root re-export 全量列出；本页补充了 runtime interface 的字段级 owner、ABI 和调用顺序。
- `hub_protocol` 在 Windows 下额外导出 `windows_hub_recent_projects_mutex_name`，非 Windows 构建不可用；其详细业务流程仍由 Hub 页面维护。
- `ui` 的导出族非常大，本页覆盖每个 public 子模块和关键稳定 DTO；widget/component 的每个视觉属性仍应以对应 rustdoc 和 template schema 为准。
- `zircon_runtime_interface` 明确是内部 lockstep boundary，不是独立发布的第三方 semver SDK；跨版本支持必须由 BuildSet、ABI shape 和 schema migration 同时证明。

## 可重复审计

```powershell
rg --no-heading --line-number "^pub use|^pub mod|^pub trait|^pub struct|^pub enum|^pub type|^pub const|^pub fn" zircon_runtime_interface/src
cargo rustdoc -p zircon_runtime_interface --features full -- -Z unstable-options --output-format json
```

`cargo rustdoc` 需要 nightly 或仓库允许的 rustdoc 配置；CI 不应把生成 JSON 当作源码事实，审计结果应回写本页和 coverage-matrix。
