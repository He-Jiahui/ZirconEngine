---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-template-hot-reload-resolver-full-scan
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/template/asset/dependency_index.rs
  - zircon_runtime/src/ui/template/asset/watch_invalidation.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolver.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resolution_report.rs
tests:
  - 10k-change dependency edge single-visit counter
  - 100k-entry resolver invalidation scan counter
  - repeated placeholder diagnostic residency budget test
  - single-leaf resource exact-node dirty test
---

# Runtime UI template hot reload与resolver全量扫描

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：template dependency/watch/surface/resource resolver审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md`
- 联动责任：EditorUI02/04消费精确layout/style dirty domain。
- 交接原因：watch generation、dependency authority、compiled cache与asset-to-surface ownership必须统一。

## 失败现象与复现证据

PERF-MVP-309：watch batch逐change重复cascade和String set工作；resolver逐invalid URI retain全cache。cached placeholder反扫全部历史diagnostics，diagnostics跨批次无界保留。template变化缺少精确node ownership时标记surface roots全dirty。

## 最低共享层根因

dependency graph、resolver cache、diagnostic lifetime与surface/node asset ownership是四套独立数据，缺少共同generation和反向索引，批量change无法单次传播也不能精确回收/dirty。

## 架构修复验收

- watch batch先规范化去重，再在generation DAG中让每edge最多访问一次。
- resolver维护URI/runtime locator反向索引，批量invalidate单次遍历或O(changed entries)，diagnostic归generation/artifact并有界。
- compiled artifact保留asset→surface/node ownership，resource/theme变化只dirty受影响node/domain；template rebuild明确替换generation。
- 1/100/10k changes/dependencies/cache/surfaces记录edge/cache/diagnostic scans、resident bytes、dirty nodes与reload p95。

## 禁止临时方案

- 不得靠定期`clear_cache`或清空全部diagnostics掩盖生命周期错误。
- 不得把全部root dirty作为长期精确热重载实现。

## 修复结果与回传

Open state: `等待EditorUI05回传generation DAG、resolver reverse index、diagnostic budget与exact-node dirty证据`。
