---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-v2-file-cache-fs-poll-and-unbounded-stale-artifacts
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/template/asset/prototype_file_cache.rs
  - zircon_runtime/src/ui/template/asset/schema/migrator.rs
  - zircon_runtime/src/ui/template/asset/schema/flat_nodes.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
tests:
  - stable 1000-load zero-filesystem-call counter
  - single-leaf hot-reload dependency-delta test
  - stale-generation entry and byte-budget test
---

# Runtime UI v2 file cache稳定命中仍轮询文件系统

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/v2` 16/16逐文件审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 交接原因：UI asset identity、watch generation、dependency index与compiled cache生命周期均由EditorUI05拥有。

## 失败现象与复现证据

PERF-MVP-271/310：稳定内存命中仍canonicalize/stat显式及全部传递source，persistent命中重复stat；`UiPrototypeStoreFileCache`命中也重新canonicalize/stat显式与传递sources。mtime进入map key使修改后旧entry残留。asset-id import miss递归read/sort资产树并parse每个`.zui`建立一次性索引；schema migrator同一source还执行Value/header/typed三次TOML parse。

## 最低共享层根因

cache key同时承担asset identity与change detection，却没有canonical current-entry、watch generation、持久asset-id/dependency index或entry/byte回收预算。

## 架构修复验收

- canonical asset identity只对应一个current generation，旧entry按明确resident/byte预算回收。
- watcher维护source fingerprint及正反向依赖；stable load filesystem calls=0，单叶修改只访问changed+dependents。
- asset-id index随catalog generation驻留，不在cache miss递归扫描全资产树。
- 1/1k/100k files记录stat/canonicalize/read_dir/parse、entry/bytes、caller I/O与hot-reload p95；删除/重命名/alias/import通过。

## 禁止临时方案

- 不得以缩短poll周期或扩大HashMap掩盖重复I/O和无界陈旧项。
- 不得为v2另建脱离runtime asset watcher的第二套全库扫描器。

## 修复结果与回传

Open state: `等待EditorUI05回传event-driven fingerprint cache、持久asset-id/dependency index及规模证据`。
