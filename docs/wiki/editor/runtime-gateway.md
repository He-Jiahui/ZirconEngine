---
related_code:
  - zircon_editor/src/core/gateway/mod.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/core/gateway/session/mod.rs
  - zircon_editor/src/core/gateway/operation_route.rs
  - zircon_editor/src/core/gateway/viewport_pick_route.rs
implementation_files:
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/in_process.rs
  - zircon_editor/src/core/gateway/session/gateway.rs
  - zircon_editor/src/core/gateway/session/contract.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/editor-and-tooling/runtime-editor-boundary-cleanup.md
tests:
  - zircon_editor/src/core/gateway
  - zircon_runtime_interface/tests
doc_type: module-detail
---

# Runtime Gateway

## 目的

`EditorRuntimeGateway` 是编辑器访问 runtime 的唯一统一门面。它屏蔽运行时位于同一进程的 `LevelSystem`，还是通过稳定 ABI session 提供服务。上层编辑器代码应基于 capability 编程，而不是判断具体 gateway 类型。

## Gateway 实现

| 实现 | 场景 | World 借用 | Serialized operation |
| --- | --- | --- | --- |
| `DetachedEditorRuntimeGateway` | 未连接项目/runtime | 否 | 否，返回 capability missing |
| `InProcessGateway` | 编辑器与 authoring level 同进程 | `with_world(_mut)` | 默认不提供 runtime operation |
| `SessionGateway` | App 建立的 ABI runtime session | 否 | 是，按 V8 API/capability |
| `EditorRuntimeGatewayHandle` | 可替换的稳定编辑器服务句柄 | 转发 | 转发并管理 generation |

`SharedEditorRuntimeGateway` 是 `Arc<dyn EditorRuntimeGateway>`。

## 能力发现

`RuntimeCapabilities` 包含：

- `SessionProfileKind::{Runtime, Editor, Dev, Minimal, Headless}`。
- 去重排序后的 core capability 名称。
- 插件 id/version 和 `PluginActivationState::{Active, Disabled, Rejected}`。

调用可选 API 前读取 `gateway.capabilities()`。缺少能力时标准错误是 `GatewayError::CapabilityMissing`，UI 应降级/禁用操作，而不是 panic。

## In-process 访问

```rust
use std::sync::Arc;
use zircon_editor::{
    EditorRuntimeGateway, InProcessGateway, SharedEditorRuntimeGateway,
};

let gateway: SharedEditorRuntimeGateway = Arc::new(
    InProcessGateway::for_authoring_level(level_system)
);

let mut inspect = |world: &zircon_runtime::scene::World| {
    // 在回调生命周期内只读访问。
};
gateway.with_world(&mut inspect)?;
# Ok::<(), zircon_editor::GatewayError>(())
```

World 引用只在 callback 内有效，不得保存或跨线程转移。Callback 内再次调用 borrowed-world API 会返回 `ReentrantBorrowedWorldAccess`，防止嵌套借用破坏 `LevelSystem` 访问纪律。

`InProcessGateway::new(core, level)` 保留 core owner；`for_authoring_level(level)` 用于 editor-owned authoring level facade。它支持 world query/watch/invalidation 和 highlight set，但默认 operation submit/poll/harvest 不可用。

## Serialized SessionGateway

`SessionGateway` 包装已验证的 `ZrRuntimeApiV8`、session handle、App-issued identity、capabilities、foreign-output state 与 runtime owner。

其构造为 `unsafe`，原因是调用者必须保证 `runtime_owner` 在 gateway drop 前保持所有函数指针的动态库/provider 存活：

```rust
let gateway = unsafe {
    SessionGateway::new_with_identity(
        runtime_owner,
        api_v8,
        session,
        identity,
        capabilities,
        foreign_output_state,
    )?
};
```

构造器会验证 session、identity 与固定 V8 API table shape，并要求 allocation release 与 viewport pick 函数存在。`with_module_composition_receipt` 附加经过 schema 校验的运行模块收据；`with_viewport_surface_bindings` 与 runtime session 共享 surface 销毁权威。

ABI 返回的 owned output 必须由同一 session 的 releaser 释放。Gateway 会限制输出预算、验证 schema/shape，并将运行调用错误映射为 `GatewayError::Runtime`，协议错误映射为 `Protocol`。

## 可替换 Handle 与身份

`EditorRuntimeGatewayHandle` 允许项目/Play 生命周期替换 endpoint：

```rust
let handle = EditorRuntimeGatewayHandle::detached();
let before = handle.generation();
handle.replace(runtime_gateway)?;
let identity = handle.identity();
assert!(handle.generation() > before);
```

每次 replace 发布新 generation。`GatewaySessionIdentity` 不只是 ABI session handle，还包含 runtime instance、gateway generation 和可选 Play instance；opaque session 值可能被另一 runtime 重用，因此仅比较 handle 不安全。

短操作通过内部 `GatewayLease` 固定一次 generation。长期持有的 ticket/operation 使用 `GatewayOrigin` 派生 route，使清理和完成仍回到创建资源的 endpoint，而不是当前替换后的 endpoint。

## Operation route

`EditorRuntimeOperationRoute::capture_at_identity` 验证当前 identity，然后固定 origin：

```rust
let identity = handle.identity();
let route = EditorRuntimeOperationRoute::capture_at_identity(&handle, &identity)?;
let operation = route.submit_operation(request)?;

loop {
    let status = route.poll_operation(operation)?;
    if status.is_terminal() { break; }
}
let result = route.harvest_operation(operation)?;
```

Submit、poll、harvest 必须用同一个 route。不要 submit 到 handle 后，在项目切换后继续通过 handle poll。

## World 同步

Gateway world-sync API 提供：

- `query_world(WorldQuery) -> WorldQueryResult`
- `watch_world(WatchRegistration) -> WatchToken`
- `unwatch_world(WatchToken)`
- `drain_world_invalidations() -> Vec<InvalidationBatch>`

Serialized gateway 用 query DTO 替代 `&World`。Watch token 属于创建它的 session；项目关闭时应在 origin endpoint unwatch 或由 session teardown 回收。

## 帧与 Viewport

- `tick_frame()` 返回 `EditorRuntimeFrameDemand`。
- `handle_event()` 提交输入/窗口等 runtime event。
- `bind_viewport_surface` / `unbind_viewport_surface` 管理原生 surface。
- `present_viewport` 请求呈现。
- `capture_frame` 返回 `EditorRuntimeFrame`，使用后显式 `release`。
- `submit_highlight_set` 发布 editor selection overlay。

Surface transition 同一 viewport 只能有一个 in-flight 操作，否则返回 `ViewportSurfaceTransitionInFlight`。

## Viewport pick route

`EditorRuntimeViewportPickRoute` 固定 request/poll/cancel 的 endpoint，并验证：

- request ABI/version/viewport/size/pixel identity。
- 返回 ticket 非空。
- poll result 的 ticket 与完整 request identity 一致。
- cancel ticket 有效。

这能拒绝“上一帧/上一项目”的异步 pick 结果。

## Plugin event 与 profiling

支持的 endpoint 可以 subscribe/unsubscribe/drain plugin event。Drain 返回 `EditorRuntimePluginEventPage`，包含 delivery、编码字节数、runtime drain/decode 时间以及 backlog 数量和最老 pending age，宿主据此诊断消费压力。

`profile_control` 用共享 `ProfileControlRequest/Response` 控制 runtime profiling；不支持时按 capability 降级。

## 错误处理

| 错误 | 含义 | 建议处理 |
| --- | --- | --- |
| `StaleGeneration` | identity 与当前 handle 不同 | 丢弃异步结果，重新捕获路由 |
| `SessionLost` | runtime 已退出 | 退回 detached，关闭相关文档/Play |
| `RequiresSerializedAccess` | 对 serialized endpoint 请求借用 World | 改用 query/operation |
| `ReentrantBorrowedWorldAccess` | callback 内嵌套借用 | 重构为一次回调或先收集 DTO |
| `CapabilityMissing` | endpoint 未实现功能 | 禁用 UI 或采用明确降级 |
| `Runtime` | runtime 报告执行失败 | 展示 operation/runtime diagnostics |
| `Protocol` | ABI shape/身份/输出不可信 | 隔离 endpoint，不消费结果 |

## 权威边界

Gateway 传递访问能力，不转移权威。Editor 可以缓存 projection、identity 和 generation；不能缓存借用的 World、renderer 内部资源或假设 session handle 全局唯一。关闭项目/Play 时先阻止新操作，再取消/收割有 origin 的资源，最后 detach gateway。
