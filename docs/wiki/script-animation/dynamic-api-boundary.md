---
related_code:
  - zircon_runtime/src/dynamic_api/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/buffer.rs
implementation_files:
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/buffer.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/engine-architecture/runtime-interface-cdylib-loader.md
  - docs/zircon_runtime_interface/runtime_api.md
tests:
  - zircon_runtime_interface/src/runtime_api/abi/api_shape_tests.rs
  - zircon_runtime_interface/src/runtime_api/session/session_identity_tests.rs
  - zircon_runtime/src/dynamic_api/tests
doc_type: module-detail
---

# 动态 API 交界

## 边界定位

`zircon_runtime::dynamic_api` 将 runtime 实现导出为 `zircon_runtime_interface` 定义的 C ABI。当前公开函数表是 `ZrRuntimeApiV8`，而 `ZrRuntimeSessionConfigV3` 是 session 配置 DTO；二者版本号不能混淆。入口符号为 `zircon_runtime_get_api_v8`。

```text
host process
  -> libloading / get_api_v8(host table)
  -> validate ABI version + size + pointer shape
  -> create_session(config_v3)
  -> tick/capture/query/operation/watch
  -> release allocations + destroy_session
```

## Panic 和状态收口

runtime 导出函数在 FFI wrapper 中使用 `catch_unwind`。panic 不得穿越边界，而是转换为 `ZrStatusCode::Panic`；host 仍必须把它视为失败并记录 diagnostics。`ZrStatus` 携带 code 和受限 byte slice，具体结果通过 `ZrOwnedResultV2`/allocation id 交付。

## 会话与函数族

`ZrRuntimeApiV8` 当前函数族可分为：

| 族 | 典型槽 |
| --- | --- |
| 生命周期/内存 | `create_session`、`destroy_session`、`release_allocation` |
| 帧/表面 | `handle_event`、`tick_frame`、`capture_frame`、`bind_viewport_surface`、`present_viewport` |
| 输入/宿主请求 | `drain_host_requests`、IME/clipboard/cursor/gamepad DTO |
| 场景查询 | `query_world`、`watch_world`、`drain_world_invalidations`、`unwatch_world` |
| 操作 | `submit_operation`、`poll_operation`、`harvest_operation` |
| 插件事件 | `subscribe_plugin_event`、`drain_plugin_events`、`unsubscribe_plugin_event` |
| 作者态辅助 | highlight set、viewport pick、accessibility tree、profile control |

所有函数第一参数是 `ZrRuntimeSessionHandle` 或 host table；session identity/generation 在实现中再次验证。句柄不能跨 session、DLL build set 或 ABI generation 复用。

## 安全调用顺序

1. 验证 `ZrHostApiV1` 指针和 `ZrRuntimeApiV8` shape。
2. 创建 session，保存返回 handle 和 build/profile identity。
3. 逐帧 `tick_frame`，按 `ZrRuntimeFrameDemandV1` 决定是否请求 capture/present。
4. 对 query/operation/watch 返回的 allocation、ticket、token 做有界 drain/harvest。
5. 处理 `ZrStatus` code；成功不代表 payload 可永久保留。
6. 先取消/解绑/取消 watch，再 `destroy_session`。

## Rust 宿主侧

在 Rust 宿主中优先使用 `zircon_runtime_host` 的 ownership/decoding 类型和 `zircon_runtime_interface` 的 DTO；不要手写 `extern "C"` 声明或复制函数表。动态 loader、WGPU 对象和 Rust trait 不应穿过 interface crate。

## 限制

- API V8 是当前内部 lockstep 边界，不是对外长期稳定 ABI；版本演进只能新增版本结构并保留 shape 校验。
- 每种 request/response 有编码字节、JSON nesting、frame dimension、world query、event page 等上限；超限返回 status/error。
- `ZrOwnedByteBuffer` 必须由 runtime 提供的 free/release 路径回收，不能跨 allocator 释放。
