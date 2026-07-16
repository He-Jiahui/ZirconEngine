---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: empty-host-request-batch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
tests:
  - session_host_request_abi_round_trips_and_releases
resolved_at: 2026-07-17
---


# Runtime10：空 host-request batch 每帧执行 JSON/ABI 往返

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F0 每帧 host request drain
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：零长度 owned buffer 的含义是 Runtime10 ABI 契约，必须由 dynamic API owner 固化并迁移所有宿主。

## 失败现象与复现证据

无 host request 的常见帧仍把空 batch JSON 编码到 owned ABI buffer，宿主再解码并释放。根因是 Runtime10 dynamic API 没有把“零长度 owned buffer 表示空 batch”固化为 ABI fast-path 契约。

性能 Session 已在 `ffi.rs` 返回 `ZrOwnedByteBuffer::empty()`，并把第二次 drain 回归改为断言空输出；非空 payload 路径保持原协议。Cargo lane 和模块文档合并仍待完成。

## 最低共享层根因

空批次语义缺失在 dynamic API session owner；只在 app caller 判断空状态无法消除其他宿主的编码、分配与释放成本。

## 架构修复验收

- 聚焦 ABI round-trip 测试通过：首次非空可解码/释放，第二次为空且无 allocator owner。
- Runtime10 文档明确空 batch 契约；所有宿主消费者接受零长度而不尝试 JSON decode。
- 当前源码空闲帧计数证明空批次不再分配/编码。

## 禁止临时方案

- 不得改变非空 JSON schema 或绕过 request ownership/release。
- 不得只在某个 app caller 跳过 drain；契约必须位于 dynamic API owner。

## 修复结果与回传

- 根因：Runtime10 dynamic session lacked an explicit zero-request ABI result and always encoded an empty JSON batch into an allocated owned buffer.
- 架构修复：The Runtime10 FFI owner now returns the canonical ZrOwnedByteBuffer::empty() for zero requests while preserving the non-empty JSON schema, ownership, and release path; the existing zircon_app consumer already accepts an empty buffer before decoding.
- 验证：Managed Windows job b0ea82ad0943466794e3af3c5333816b / run 4b9e4151d39f4cd9b95de28b2c0ee261 executed the exact lib-only host-request drain test once: 1 passed, 0 failed, 8190 filtered, exit 0; independent specification and quality reviews reported 0 critical and 0 important findings.
- 回传：Returned the fixed empty host-request batch ABI contract and managed evidence to Performance01; Runtime10 retains the ABI owner and does not change non-empty payload semantics.
