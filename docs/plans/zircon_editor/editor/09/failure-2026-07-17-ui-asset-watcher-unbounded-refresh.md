---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: ui-asset-watcher-unbounded-refresh
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/host.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/dependency_index/generation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/generation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/traversal.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/queue.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/service.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/job.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/commit.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher/diagnostics.rs
reference_sources:
  - dev/godot/editor/file_system/editor_file_system.cpp
tests:
  - tools/tests/test_editor09_ui_asset_watcher_generation_contract.py
  - 1000/10000 filesystem event bounded-coalescing stress
  - rename/delete/write burst debounce and generation ordering
  - direct/transitive import invalidation parity matrix
---

# Editor09：UI asset watcher 无界队列与全量刷新

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：UI asset workspace watcher/refresh/import 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：filesystem event retention、asset dependency index与import invalidation属于 Editor09 asset management generation。

## 失败现象与复现证据

每个 watched root 的 notify callback 向 `crossbeam_channel::unbounded` 发送每条 path。retained poll 用 `while try_recv` 一次性排空所有积压，再在主线程 BTreeSet coalesce、扫描所有 open UI asset sessions、读盘/hash/rebuild/rehydrate imports并逐 instance同步。保存、git checkout、生成器或目录复制可产生 1k/10k burst，队列内存与单 tick工作均无上限。

依赖刷新只从每个 session 的 direct widget/style refs 现算 matching；没有 project-generation reverse dependency index，transitive change 要靠后续全 hydrate偶然覆盖。Godot EditorFileSystem持有 file cache/modified time/import md5并通过 scan/import state更新，而不是让每个 editor consumer各自无界接收后全表重扫。

## 最低共享层根因

watcher channel是原始 path transport，没有 bounded latest-set/debounce/generation；asset graph没有 canonical reverse dependencies，因此 consumer只能从 open sessions重扫并重建。

## 架构修复验收

- callback写入有界 canonical-path latest set/ring，rename/delete/write burst在时间窗内合并；overflow/drop/coalesced/age可观测且最终状态一致。
- retained tick有 count/time budget与跨tick cursor；I/O/parse在有界worker，generation commit防止旧结果覆盖新文件。
- project asset generation维护direct+transitive reverse dependency index；只刷新受影响 documents，单 physical file每 generation read/parse≤1。
- 1k/10k burst内存/主线程p95有界；rename/delete/recreate、multi-root、conflict/local-dirty、stale-import诊断顺序通过。

## 禁止临时方案

- 不得只把 unbounded channel换成 bounded channel并静默丢最后状态。
- 不得在单 tick继续排空全部积压。
- 不得让 UI asset editor私有 watcher成为第二个 runtime asset inventory authority。

## 产出记录与时间

Open state: `bounded ingress/reconcile、reverse dependency generation、shared physical parse generation、EditorJobSystem worker、generation-checked commit 与 per-asset bounded retry 已完成源码硬切；受管 Cargo、1k/10k 产品 storm/p95、最终独立复审、fixed return 与 managed commit 仍待完成。`

2026-07-22局部止损：性能审查TDD删除了`asset_id_for_path`每事件收集matching roots的临时`Vec`，并让poll直接把changed batch借给refresh归一化，避免返回同一批结果前深clone全部`String`；源码合同2/2与rustfmt通过。该止损不改变本failure的open状态：unbounded ingress、drain-all、主线程同步I/O/parse/hydrate、全session scan及reverse dependency generation仍未解决，且current-source Cargo与1k/10k storm未验收。

2026-07-23基础设施硬切：旧 `crossbeam_channel::unbounded`、`while try_recv` drain-all 与
`Result<Vec<String>, _>` poll surface 已删除。notify callback 只写容量 4,096 的 physical-path latest-set；
retained poll 的 ingress/reconcile 枚举按 256 item/2ms 双预算跨 tick 处理，并发布 pending/cursor-active/
coalesced/overflow/oldest-age 诊断。容量溢出会丢弃不完整 path generation，转为
`{session id, next import index}` cursor borrowed 枚举当前打开 route 与 direct import roots，不预先
clone/物化完整 reconcile set；
现有 traversal 重新展开 transitive imports，不建立第二 asset inventory。静态合同与旧性能守卫合计
5/5 GREEN，exact rustfmt/结构预算通过；独立 review 从 0/3/0 收敛到 0/0/0。该阶段只关闭无界 ingress与
单 tick drain 放大；reverse dependency generation、worker parse/commit、产品 storm、受管 Cargo、
fixed return 与 commit 仍未完成，
所以本 Failure 保持 open。

2026-07-23 reverse-generation/worker 源码阶段：打开文档现在进入唯一
`UiAssetDependencyGeneration`，direct route 与 normalized import edge 均为 generation-owned reverse index；
import edge 在 resolve/read/parse 前登记，initial hydration 即使遇到 missing/invalid import 也保留 last-good
session、stale diagnostic 与恢复边。旧扁平 `imports.rs`/`refresh.rs` 已删除，physical-path parse cache 在一个
worker generation 的全部受影响文档间共享。Watcher 只 enqueue `EditorJobSystem` `Index`/`Background` job；
UI 线程以 project root、dependency generation、route identity、disk baseline 与 source fingerprint 做 commit gate，
resolved imports 与 reverse edges 按 `dependency_generation -> ui_asset_sessions` 同一锁 epoch 提交。
Transient I/O/job/commit/sync failure 使用 per-asset/per-generation retry cohort（50ms base、2s cap、最多6次），
immediate requeue 会先排除 delayed retry，exhausted/superseded/pending/active 状态进入 typed diagnostics；same-project
save/restart 不清空 work，真实 project-root cutover 才 cancel/reset。静态合同 11/11 与精确 rustfmt 通过；独立复审
先后暴露 0/6/2 与 0/5/2，第三轮独立复审已收敛为 0/0/0。Coordinator01 validation-copy failure 与外部
Text01 Cargo 仍阻断 source-bound Rust gate，因此本 Failure 保持 `open/validation_blocked`，不写 accepted/fixed。
