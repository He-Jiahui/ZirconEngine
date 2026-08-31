---
related_code:
  - zircon_editor/src/core/logging
canonical_review:
  - docs/plans/performance/01/2026-08-23-editor-core-logging-currentness-revalidation.md
protected_targets:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: protected-plan-routing
status: update_requested_not_applied
---

# Editor core logging当前性保护计划路由（2026-08-23）

## 请求Performance01接纳

将`zircon_editor/src/core/logging/**`记录为13/13 Rust文件、1,889 physical lines、59,980 bytes、
16 tests，ordered path + NUL + raw bytes + NUL SHA256为
`405b501147771b555850f1885dc8bcdd036f8922da72d57b13eb2c8bf7af2aca`。状态为
`static_current_revalidated / dynamic_and_structural_pending`。

将旧路由中尚未进入Performance01主表的`PERF-MVP-644`接纳为P0：所有producer仍在全局锁内执行
逐row目录检查、metadata/open/write/flush；抢到dispatcher的producer还同步drain sink；宽UI snapshot
重复全量scan/clone/format/join，而逐条JSON bus projection没有production subscriber。

要求cutover为`LogIngressRange -> LogStoreGeneration -> DiagnosticsWriterBatch ->
LogPersistenceReceipt -> FilteredLogWindowGeneration -> RetainedConsoleRowDelta`。writer必须保留active
segment并在独立recursion-safe lane按bytes/age/fatal/shutdown显式flush；UI只materialize visible加
overscan rows；store提供cursor/range/O(1) sequence lookup；逐条JSON duplicate authority必须hard-cut。

## 其他模块计划的必需更新

| owner plan | 必需接纳的责任 |
|---|---|
| Editor17 | memory generation为即时authority；定义range receipt、RSS预算、batch ingress、persistence/fatal/shutdown合同和open-segment生命周期 |
| Editor14 + Runtime11 | 唯一recursion-safe diagnostics blocking-I/O lane；count/bytes/age/deadline bounds；不得从会再次记录日志的失败路径递归提交 |
| Editor02 + EditorUI08 | 一个typed generation/range invalidation；宽snapshot引用cached generation；filter与visible-window owner只生成visible加overscan rows；删除unused JSON bus path |
| Editor13 | compiler diagnostics改为bounded page/batch ingress；一次diagnostic batch不得产生N个producer locks、file opens或flush requests |

计划owner应同时记录producer wait、store lock hold、admitted/dropped rows/bytes、writer queue age/RSS、
segment open/metadata/write/flush、UI scan/clone/format和visible-row计数。RenderDoc不是本切片CPU/I/O验收
工具，只能补console绘制parity。

## 受保护索引状态请求

- `pending.md`：加入上述冻结、canonical review、`PERF-MVP-644`和
  `static_current_revalidated / dynamic_and_structural_pending`。
- `review.md`：在结构cutover、approved-root managed tests、scale/RSS gates以及至少31次F0/F4 WPR
  CPU/wait/file-I/O/power矩阵全部通过前，不得加入。

本会话不修改这些受保护文件或owner计划。当前无current-source可执行文件且managed validator已归档，
所以没有动态里程碑、git commit或企微通知。
