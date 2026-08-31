---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: dynamic-status-diagnostic-ownership
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/buffer.rs
tests:
  - dynamic error ownership and reload stress
  - V3/V4 ABI negotiation parity
---

# Runtime10：dynamic status diagnostics所有权

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-429 dynamic status diagnostic ownership audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：dynamic status 的诊断内存所有权、释放协议与 ABI 协商由 Runtime10 所有。

## 失败现象与复现证据

`error_status`把每个动态错误message转为boxed bytes再`Box::leak`成`'static`，`ZrStatus`却只携带borrowed slice且无free或有效期合同。任意重复失败会永久增长RSS；TLS覆盖会引入跨调用/跨线程寿命陷阱，不能作为无合同修补。

## 最低共享层根因

动态错误 payload 缺少跨 ABI 的显式所有权、有效期与释放协议，当前 borrowed slice 合同无法表达可释放的动态诊断。

## 架构修复验收

- 冻结新版本错误所有权：caller-owned output或带显式free的owned diagnostics；稳定小消息可static，动态详情不得泄漏。
- V3兼容、V4协商、panic边界、并发与runtime reload/unload语义明确。
- 1/1k/1M错误后leaked bytes=0、owned buffers全部释放且RSS有界；结果回传PERF-MVP-429。

## 禁止临时方案

不得继续使用 `Box::leak`、TLS 借用寿命补丁、隐式 ABI fallback 或弱化 reload/concurrency/ownership 测试。

## 修复结果与回传

Open state: `待 Runtime10 冻结动态诊断所有权并回传ABI与长时内存证据`。

### 2026-08-26 current-source architecture pre-review

- 历史 `Box::leak` 已不在 current source；Runtime 现在把动态错误写入每线程 4 KiB 有界缓冲，并以 borrowed `ZrByteSlice` 返回。该实现限制了单线程常驻容量，但 `ZrStatus` 仍为 `Copy`，下一次同线程错误可覆盖旧 status 指向的字节，跨线程保留和 DLL unload 也没有 ABI 生命周期保证，因此 failure 不能关闭。
- session 创建前即可产生 status，现有 session-scoped `release_allocation` 不能单独承接所有错误。App/Editor 当前立即读取 diagnostics 只是调用习惯，不是可验证的 wire contract；TLS 多槽、扩大缓冲或“立即读取”文档均被拒绝。
- 下一步必须在新 API table 中原子切换到 non-`Copy` 显式 owned diagnostics（优先复用带 owner token/free callback 的 buffer contract）或同等 caller-output 方案，并同批迁移 negotiation、create/destroy、普通调用、panic、plugin-event nested status 及 App/Editor RAII decoder，随后删除 borrowed/TLS 路径，不保留逐调用 V7/V8 fallback。
- 当前 App、Editor 与 API table 调用链正在被其它 owner 修改；仅改 interface/Runtime 会生成不可用的并行 ABI，所以本轮状态为 `design_review_complete_atomic_cutover_deferred_due_shared_owner_changes`，未写任何兼容层或局部寿命补丁。
- 完整设计、迁移顺序和 1/1k/1M error、并发、pre-session、reload/unload、panic 的 allocation/free/RSS 验收矩阵记录在 `docs/plans/optimize/zircon_runtime/10/2026-08-26-status-diagnostics-abi-hard-cut-pre-review.md`。本轮没有性能、功耗或泄漏消失声明。
