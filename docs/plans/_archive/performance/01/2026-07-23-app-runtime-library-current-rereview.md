---
related_code:
  - zircon_app/src/entry/runtime_library
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
tests:
  - zircon_app/src/entry/runtime_library/tests.rs
  - current-source managed Windows Cargo pending
  - owned-output and failed-destroy failure storms pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App runtime library当前源码增量复核（2026-07-23）

## 范围与证据边界

`zircon_app/src/entry/runtime_library/**`当前物理源码 **8/8** 个Rust文件、**1,939** 行已逐文件复读：`library_path.rs`、`loaded_runtime.rs`、`mod.rs`、`runtime_library_error.rs`、`runtime_session.rs`、`runtime_session/operation.rs`、`tests.rs`和`wake_registry.rs`。其中4个tracked文件有其他会话未提交修改，`wake_registry.rs`为未跟踪当前源码；本轮只读审查并保留其内容，不吸收为本计划实现。

这份记录只把`runtime_library`提升为当前源码静态覆盖，不重写2026-07-19入口树144/144的历史证据。入口树当前已增长为146个Rust文件、13,796行，完整current-hash重对账仍待按子目录完成，故`zircon_app/src/entry`继续留在`pending.md`。

## 当前热点

- **PERF-MVP-425 / Runtime10**：非空host request仍由调用线程跨ABI取完整JSON owned buffer后整批解码，且`RuntimeWakeRegistration::wake`和FFI trampoline共享全局`Mutex<HashMap<u64, EventLoopProxy>>`；host已持registration却仍支付全局查表/锁成本，重复wake也没有edge coalescing合同。
- **PERF-MVP-574 / Runtime10**：`capture_frame`、host/plugin drain和operation output都先检查FFI status，后建立owned output的解码/释放责任。当前ABI没有冻结“runtime在错误返回前写出非空owned output”时由谁释放；若producer写出，App错误早退会跳过free。success后的decode失败会释放，不能把它误报为同类泄漏。
- **PERF-MVP-574 / teardown**：`destroy_session`失败时为避免runtime继续回调已注销wake sink造成UAF，Drop会永久`mem::forget` registration。这个fail-safe保证当前回调安全，但没有retry/detach/terminal owner或有界quarantine；重复失败可使全局registry/proxy驻留无上界。现有测试锁定destroy-before-unregister和失败时forget，没有证明最终回收或驻留上限。

## 目标设计与动态验收

Runtime10先冻结out-param合同：调用前必须为空；无论success/error，只要callee写出非空owned output，它就是可释放的有效owner且必须exactly-once free。App在调用前后立即以RAII guard接管raw buffer/frame，status、ABI版本、JSON decode、empty和cleanup error所有分支都走同一释放路径；cleanup失败不得覆盖primary FFI错误，但要进入组合诊断。不得跨Windows CRT直接接管未知allocator内存。

为session teardown定义显式wake detach + destroy retry终态，或可测量且有hard count/bytes/age上限的quarantine；`mem::forget`只能作为进程终止级最后手段。fake FFI覆盖success、error-after-output、invalid JSON、wrong ABI和free failure，输出0/1KiB/64MiB时断言`free calls == owned outputs`、double-free=0、leaked bytes=0。对1/1k/100k次failed destroy记录registry entries、proxy owners、retry/drop/age和callback安全，要求无UAF且驻留为0或明确硬有界。

当前未运行Cargo：受管测试、规模counter、F0动态runtime加载/退出、F2 fallback capture和RenderDoc仍缺失，因此本目录不得进入`review.md`。

## 责任计划

- Runtime10：继续使用既有`runtime/10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md`，统一处理PERF-MVP-425与574，禁止建立重复failure记录。
- Runtime03：只消费frame demand/cadence和fallback capture结果，不拥有owned-output ABI。
- Runtime12：只消费input/gamepad storm预算，不拥有wake registry或session teardown。
