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
  - zircon_editor/src/ui/host/asset_editor_sessions/watcher.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports.rs
reference_sources:
  - dev/godot/editor/file_system/editor_file_system.cpp
tests:
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

## 修复结果与回传

Open state: `待 Editor09 建立 bounded coalescing watcher、reverse dependency generation 与预算化异步 refresh`。
