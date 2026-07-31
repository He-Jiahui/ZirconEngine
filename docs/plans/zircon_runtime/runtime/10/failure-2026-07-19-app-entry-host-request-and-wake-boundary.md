---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: app-entry-host-request-and-wake-boundary
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_entry_app/host_requests
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_library/runtime_session/operation.rs
  - zircon_app/src/entry/runtime_library/wake_registry.rs
  - zircon_runtime/src/dynamic_api
tests:
  - bounded host-request storm contract
  - typed batch ABI parity
  - direct wake no-registry-lock regression
  - owned output exactly-once free failures
  - failed destroy registry reclamation storm
---

# Runtime10：app entry host-request与wake边界

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-425 / PERF-MVP-574 host request, wake and owned-output boundary
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：dynamic ABI batch、wake registration 与 owned output/teardown 合同由 Runtime10 所有。

## 失败现象与复现证据

空host-request fast path已消除空JSON往返；非空批次仍在runtime侧全量drain并收集Vec、序列化JSON、跨owned ABI buffer、app侧再反序列化并在主线程无预算应用。IME/cursor/gamepad请求缺少count/time/backpressure/coalesce合同。host持有wake registration，但立即需求仍经全局`Mutex<HashMap<u64, EventLoopProxy>>`查询；仅FFI callback需要registry。

2026-07-23 current-source复核新增PERF-MVP-574：`capture_frame`、host/plugin drain和operation output先检查status，后解码/释放owned output；ABI尚未冻结callee在error前写出非空owner时的释放方，App错误早退会跳过free。`destroy_session`失败时App为避免已注销callback owner造成UAF而永久forget wake registration，当前安全优先级正确，但重复失败没有detach/retry终态或驻留硬上限。

## 最低共享层根因

非空 host request 仍承担 JSON 双解析和无预算整批应用，host wake 仍经过全局 registry；owned output 的 error-path 释放方与 failed-destroy 的有界终态也未冻结。

## 架构修复验收

- 冻结typed/binary batch ABI或等价零重复解析合同；稳定非空批次JSON encode/decode次数为0，错误与顺序语义保持。
- request按lossless edge、latest-value state和bounded command分类；每帧drain有count/time预算、queue peak/age/drop/coalesce可观测。
- host-owned wake走registration直接proxy，不取全局registry锁；callback保留registry且生命周期/销毁失败安全。
- 冻结out-param合同：调用前为空；callee无论success/error写出的非空owned output都必须由App exactly-once free。App在status/ABI/decode前建立RAII guard，empty、invalid JSON、wrong ABI和cleanup failure不泄漏/不double-free；cleanup诊断不得覆盖primary FFI error。
- session teardown必须有显式wake detach+destroy retry终态，或有count/bytes/age硬上限并可观测drop的quarantine；`mem::forget`不得成为可重复的常规终态。1/1k/100k failed destroy证明callback无UAF且registry/proxy为0或硬有界。
- 1/1k/10k混合请求记录bytes、alloc、manager resolve、lock wait、main-thread p95；结果回传PERF-MVP-425。

## 禁止临时方案

不得仅增大队列或在JSON外再包一层JSON，不得让borrow跨动态runtime卸载，不得牺牲IME/close等lossless边沿。

## 修复结果与回传

Open state: `待 Runtime10 建立typed bounded host-request batch、direct wake owner、owned-output exactly-once free与failed-destroy有界回收，并回传ABI/压力证据`。
